// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing accounts.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, Txo},
        transaction,
        txo::TxoModel,
        WalletDbError,
    },
    json_rpc::{json_rpc_request::JsonRPCRequest, v2::api::request::JsonCommandRequest},
    service::{
        ledger::{LedgerService, LedgerServiceError},
        WalletService,
    },
    util::{
        constants::MNEMONIC_KEY_DERIVATION_VERSION,
        encoding_helpers::{
            hex_to_ristretto, hex_to_ristretto_public, ristretto_public_to_hex, ristretto_to_hex,
        },
    },
};
use base64;
use bip39::{Language, Mnemonic, MnemonicType};
use displaydoc::Display;
use mc_account_keys::{AccountKey, RootEntropy};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::ring_signature::KeyImage;

#[derive(Display, Debug)]
pub enum AccountServiceError {
    /// Error interacting& with the database: {0}
    Database(WalletDbError),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Diesel error: {0}
    Diesel(diesel::result::Error),

    /// Error with the Ledger Service: {0}
    LedgerService(LedgerServiceError),

    /// Unknown key version version: {0}
    UnknownKeyDerivation(u8),

    /// Invalid BIP39 english mnemonic: {0}
    InvalidMnemonic(String),

    /// Error decoding base64: {0}
    Base64DecodeError(String),

    /// Error decoding private view key: {0}
    DecodePrivateKeyError(String),

    /// Account is a view only account and shouldn't be
    AccountIsViewOnly(AccountID),

    /// Account is not a view only account and should be
    AccountIsNotViewOnly(AccountID),

    /// JSON Rpc Request was formatted incorrectly
    InvalidJsonRPCRequest,
}

impl From<WalletDbError> for AccountServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<mc_ledger_db::Error> for AccountServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<hex::FromHexError> for AccountServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<diesel::result::Error> for AccountServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<LedgerServiceError> for AccountServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

impl From<base64::DecodeError> for AccountServiceError {
    fn from(src: base64::DecodeError) -> Self {
        Self::Base64DecodeError(src.to_string())
    }
}

impl From<mc_account_keys::Error> for AccountServiceError {
    fn from(src: mc_account_keys::Error) -> Self {
        Self::Base64DecodeError(src.to_string())
    }
}

impl From<mc_util_serial::DecodeError> for AccountServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::DecodePrivateKeyError(src.to_string())
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// accounts.
pub trait AccountService {
    /// Creates a new account with default values.
    fn create_account(
        &self,
        name: Option<String>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the mnemonic.
    #[allow(clippy::too_many_arguments)]
    fn import_account(
        &self,
        mnemonic_phrase: String,
        key_derivation_version: u8,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the entropy.
    #[allow(clippy::too_many_arguments)]
    fn import_account_from_legacy_root_entropy(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the mnemonic.
    fn import_view_only_account(
        &self,
        view_private_key: String,
        spend_public_key: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
    ) -> Result<Account, AccountServiceError>;

    fn reimport_account(&self, account_id: &AccountID) -> Result<(), AccountServiceError>;

    fn get_view_only_account_import_request(
        &self,
        account_id: &AccountID,
    ) -> Result<JsonRPCRequest, AccountServiceError>;

    /// List accounts in the wallet.
    fn list_accounts(
        &self,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Account>, AccountServiceError>;

    /// Get an account in the wallet.
    fn get_account(&self, account_id: &AccountID) -> Result<Account, AccountServiceError>;

    fn get_next_subaddress_index_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<u64, AccountServiceError>;

    /// Update the name for an account.
    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, AccountServiceError>;

    /// complete a sync request for a view only account
    fn sync_account(
        &self,
        account_id: &AccountID,
        txo_ids_and_key_images: Vec<(String, String)>,
        next_subaddress_index: u64,
    ) -> Result<(), AccountServiceError>;

    /// Remove an account from the wallet.
    fn remove_account(&self, account_id: &AccountID) -> Result<bool, AccountServiceError>;
}

impl<T, FPR> AccountService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn create_account(
        &self,
        name: Option<String>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
    ) -> Result<Account, AccountServiceError> {
        log::info!(self.logger, "Creating account {:?}", name,);

        // Generate entropy for the account
        let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);

        // Determine bounds for account syncing. If we are offline, assume the network
        // has at least as many blocks as our local ledger.
        let local_block_height = self.ledger_db.num_blocks()?;
        let network_block_height = if self.offline {
            local_block_height
        } else {
            self.get_network_block_height()?
        };

        // Since we are creating the account from randomness, it is astronomically
        // improbable that it would have collided with another account that
        // already received funds. For this reason, start scanning after the
        // current network block index. Only perform account scanning on blocks that are
        // newer than the account.
        // The index of the previously published block is one less than the ledger
        // height, and the next block after that has an index of one more.
        let first_block_index = network_block_height; // -1 +1
        let import_block_index = local_block_height; // -1 +1

        let conn = self.get_conn()?;
        transaction(&conn, || {
            let (account_id, _public_address_b58) = Account::create_from_mnemonic(
                &mnemonic,
                Some(first_block_index),
                Some(import_block_index),
                None,
                &name.unwrap_or_else(|| "".to_string()),
                fog_report_url,
                fog_report_id,
                fog_authority_spki,
                &conn,
            )?;
            let account = Account::get(&account_id, &conn)?;
            Ok(account)
        })
    }

    fn import_account(
        &self,
        mnemonic_phrase: String,
        key_derivation_version: u8,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
    ) -> Result<Account, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

        if key_derivation_version != MNEMONIC_KEY_DERIVATION_VERSION {
            return Err(AccountServiceError::UnknownKeyDerivation(
                key_derivation_version,
            ));
        }

        // Get mnemonic from phrase
        let mnemonic = match Mnemonic::from_phrase(&mnemonic_phrase, Language::English) {
            Ok(m) => m,
            Err(_) => {
                return Err(AccountServiceError::InvalidMnemonic(
                    mnemonic_phrase.to_string(),
                ))
            }
        };

        // We record the local highest block index because that is the earliest we could
        // start scanning.
        let import_block = self.ledger_db.num_blocks()? - 1;

        let conn = self.get_conn()?;
        transaction(&conn, || {
            Ok(Account::import(
                &mnemonic,
                name,
                import_block,
                first_block_index,
                next_subaddress_index,
                fog_report_url,
                fog_report_id,
                fog_authority_spki,
                &conn,
            )?)
        })
    }

    fn import_account_from_legacy_root_entropy(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
    ) -> Result<Account, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing account {:?} with first block: {:?}",
            name,
            first_block_index,
        );
        // Get account key from entropy
        let mut entropy_bytes = [0u8; 32];
        hex::decode_to_slice(entropy, &mut entropy_bytes)?;

        // We record the local highest block index because that is the earliest we could
        // start scanning.
        let import_block = self.ledger_db.num_blocks()? - 1;

        let conn = self.get_conn()?;
        transaction(&conn, || {
            Ok(Account::import_legacy(
                &RootEntropy::from(&entropy_bytes),
                name,
                import_block,
                first_block_index,
                next_subaddress_index,
                fog_report_url,
                fog_report_id,
                fog_authority_spki,
                &conn,
            )?)
        })
    }

    fn import_view_only_account(
        &self,
        view_private_key: String,
        spend_public_key: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
    ) -> Result<Account, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing view only account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

        let view_private_key =
            hex_to_ristretto(&view_private_key).map_err(AccountServiceError::Base64DecodeError)?;
        let spend_public_key = hex_to_ristretto_public(&spend_public_key)
            .map_err(AccountServiceError::Base64DecodeError)?;

        let import_block_index = self.ledger_db.num_blocks()? - 1;

        let conn = self.get_conn()?;
        transaction(&conn, || {
            Ok(Account::import_view_only(
                &view_private_key,
                &spend_public_key,
                name,
                import_block_index,
                first_block_index,
                next_subaddress_index,
                &conn,
            )?)
        })
    }

    fn reimport_account(&self, account_id: &AccountID) -> Result<(), AccountServiceError> {
        let conn = self.get_conn()?;
        let account = Account::get(account_id, &conn)?;
        account.update_next_block_index(account.first_block_index as u64, &conn)?;
        Ok(())
    }

    fn get_view_only_account_import_request(
        &self,
        account_id: &AccountID,
    ) -> Result<JsonRPCRequest, AccountServiceError> {
        let conn = self.get_conn()?;
        let account = Account::get(account_id, &conn)?;

        if account.view_only {
            return Err(AccountServiceError::AccountIsViewOnly(account_id.clone()));
        }

        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        let view_private_key = account_key.view_private_key();
        let spend_public_key = RistrettoPublic::from(account_key.spend_private_key());

        let json_command_request = JsonCommandRequest::import_view_only_account {
            view_private_key: ristretto_to_hex(view_private_key),
            spend_public_key: ristretto_public_to_hex(&spend_public_key),
            name: Some(account.name.clone()),
            first_block_index: Some(account.first_block_index.to_string()),
            next_subaddress_index: Some(account.next_subaddress_index(&conn)?.to_string()),
        };

        let src_json: serde_json::Value = serde_json::json!(json_command_request);
        let method = src_json
            .get("method")
            .ok_or(AccountServiceError::InvalidJsonRPCRequest)?
            .as_str()
            .ok_or(AccountServiceError::InvalidJsonRPCRequest)?;
        let params = src_json
            .get("params")
            .ok_or(AccountServiceError::InvalidJsonRPCRequest)?;

        Ok(JsonRPCRequest {
            method: method.to_string(),
            params: Some(params.clone()),
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(1)),
        })
    }

    fn list_accounts(
        &self,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Account>, AccountServiceError> {
        let conn = self.get_conn()?;
        Ok(Account::list_all(&conn, offset, limit)?)
    }

    fn get_account(&self, account_id: &AccountID) -> Result<Account, AccountServiceError> {
        let conn = self.get_conn()?;
        Ok(Account::get(account_id, &conn)?)
    }

    fn get_next_subaddress_index_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<u64, AccountServiceError> {
        let conn = self.get_conn()?;
        let account = Account::get(account_id, &conn)?;
        Ok(account.next_subaddress_index(&conn)?)
    }

    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, AccountServiceError> {
        let conn = self.get_conn()?;
        Account::get(account_id, &conn)?.update_name(name, &conn)?;
        Ok(Account::get(account_id, &conn)?)
    }

    fn sync_account(
        &self,
        account_id: &AccountID,
        txo_ids_and_key_images: Vec<(String, String)>,
        next_subaddress_index: u64,
    ) -> Result<(), AccountServiceError> {
        let conn = self.get_conn()?;
        let account = Account::get(account_id, &conn)?;

        if !account.view_only {
            return Err(AccountServiceError::AccountIsNotViewOnly(
                account_id.clone(),
            ));
        }

        for (txo_id_hex, key_image_encoded) in txo_ids_and_key_images {
            let key_image: KeyImage = mc_util_serial::decode(&hex::decode(key_image_encoded)?)?;
            let spent_block_index = self.ledger_db.check_key_image(&key_image)?;
            Txo::update_key_image(&txo_id_hex, &key_image, spent_block_index, &conn)?;
        }

        for _ in account.next_subaddress_index(&conn)?..next_subaddress_index {
            AssignedSubaddress::create_next_for_account(
                &account_id.to_string(),
                "Recovered In Account Sync",
                &self.ledger_db,
                &conn,
            )?;
        }

        Ok(())
    }

    fn remove_account(&self, account_id: &AccountID) -> Result<bool, AccountServiceError> {
        log::info!(self.logger, "Deleting account {}", account_id,);
        let conn = self.get_conn()?;
        transaction(&conn, || {
            let account = Account::get(account_id, &conn)?;
            account.delete(&conn)?;
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{models::Txo, txo::TxoModel},
        test_utils::{
            add_block_to_ledger_db, create_test_received_txo, generate_n_blocks_on_ledger,
            get_empty_test_ledger, get_test_ledger, manually_sync_account, setup_wallet_service,
            setup_wallet_service_offline, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, ViewAccountKey};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPrivate;
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::{tokens::Mob, Amount, Token};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::convert::TryInto;

    #[test_with_logger]
    fn test_applesauce(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let entropy = RootEntropy::from_random(&mut rng);
        let account_key = AccountKey::from(&RootIdentity::from(&entropy));

        let block_count: i64 = 100;

        let known_recipients = vec![account_key.subaddress(0)];
        let mut ledger_db = get_test_ledger(
            5,
            &known_recipients,
            block_count.try_into().unwrap(),
            &mut rng,
        );

        let service = setup_wallet_service(ledger_db.clone(), logger);
        let wallet_db = &service.wallet_db.as_ref().unwrap();

        assert_eq!(ledger_db.num_blocks().unwrap(), block_count as u64);

        // Create an account that exists from the beginning of the ledger
        let account = service
            .import_account_from_legacy_root_entropy(
                hex::encode(entropy.bytes),
                None,
                None,
                None,
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();
        let account_id = AccountID(account.id);

        let account = service.get_account(&account_id).unwrap();
        assert_eq!(account.first_block_index, 0);
        assert_eq!(account.next_block_index, account.first_block_index);

        manually_sync_account(&ledger_db, &wallet_db, &account_id, &service.logger);
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(
            account.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );

        service.reimport_account(&account_id).unwrap();
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(account.next_block_index, account.first_block_index);
        manually_sync_account(&ledger_db, &wallet_db, &account_id, &service.logger);
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(
            account.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );

        // create an account that has its first_block_index set to later in the ledger
        let account2 = service
            .create_account(None, "".to_string(), "".to_string(), "".to_string())
            .unwrap();
        assert_eq!(
            account2.first_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );
        generate_n_blocks_on_ledger(
            5,
            &known_recipients,
            block_count.try_into().unwrap(),
            &mut rng,
            &mut ledger_db,
        );
        assert_eq!(account2.next_block_index, account2.first_block_index);

        manually_sync_account(&ledger_db, &wallet_db, &account_id, &service.logger);
        let account2 = service.get_account(&account_id).unwrap();
        assert_eq!(
            account2.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );

        service.reimport_account(&account_id).unwrap();
        let account2 = service.get_account(&account_id).unwrap();
        assert_eq!(account2.next_block_index, account2.first_block_index);

        manually_sync_account(&ledger_db, &wallet_db, &account_id, &service.logger);
        let account2 = service.get_account(&account_id).unwrap();
        assert_eq!(
            account2.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );
    }

    #[test_with_logger]
    fn test_remove_account_from_txo(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let wallet_db = &service.wallet_db.as_ref().unwrap();

        // Create an account.
        let account = service
            .create_account(
                Some("A".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Add a transaction, with transaction status.
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key).unwrap();

        create_test_received_txo(
            &account_key,
            0,
            Amount::new((100 * MOB) as u64, Mob::ID),
            13 as u64,
            &mut rng,
            &wallet_db,
        );

        let txos = Txo::list_for_account(
            &account.id,
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        // Delete the account. The transaction status referring to it is also cleared.
        let account_id = AccountID(account.id.clone().to_string());
        let result = service.remove_account(&account_id);
        assert!(result.is_ok());

        let txos = Txo::list_for_account(
            &account.id,
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 0);
    }

    #[test_with_logger]
    fn test_create_account_offline(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let service = setup_wallet_service_offline(ledger_db.clone(), logger.clone());

        // Create an account.
        let account = service
            .create_account(
                Some("A".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Even though we don't have a network connection, it sets the block indices
        // based on the ledger height.
        assert_eq!(service.ledger_db.num_blocks().unwrap(), 12);
        assert_eq!(account.first_block_index, 12);
        assert_eq!(account.next_block_index, 12);
        assert_eq!(account.import_block_index, Some(12));

        // Syncing the account does nothing to the block indices since there are no new
        // blocks.
        let account_id = AccountID(account.id);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &account_id,
            &logger,
        );
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(account.first_block_index, 12);
        assert_eq!(account.next_block_index, 12);
        assert_eq!(account.import_block_index, Some(12));
    }

    #[test_with_logger]
    fn test_create_account_offline_no_ledger(logger: Logger) {
        let ledger_db = get_empty_test_ledger();
        let service = setup_wallet_service_offline(ledger_db.clone(), logger.clone());

        // Create an account.
        let account = service
            .create_account(
                Some("A".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // The block indices are set to zero because we have no ledger information
        // whatsoever.
        assert_eq!(service.ledger_db.num_blocks().unwrap(), 0);
        assert_eq!(account.first_block_index, 0);
        assert_eq!(account.next_block_index, 0);
        assert_eq!(account.import_block_index, Some(0));

        // Syncing the account does nothing to the block indices since there are no
        // blocks in the ledger.
        let account_id = AccountID(account.id);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &account_id,
            &logger,
        );
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(account.first_block_index, 0);
        assert_eq!(account.next_block_index, 0);
        assert_eq!(account.import_block_index, Some(0));
    }

    #[test_with_logger]
    fn test_sync_view_only_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let wallet_db = &service.wallet_db.as_ref().unwrap();
        let conn = wallet_db.get_conn().unwrap();

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let spend_private_key = RistrettoPrivate::from_random(&mut rng);

        let account_key = AccountKey::new(&spend_private_key, &view_private_key);
        let view_account_key = ViewAccountKey::from(&account_key);

        let view_only_account = service
            .import_view_only_account(
                ristretto_to_hex(&view_account_key.view_private_key()),
                ristretto_public_to_hex(&view_account_key.spend_public_key()),
                None,
                None,
                None,
            )
            .unwrap();

        let account_id = AccountID(view_only_account.id.clone());

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![
                view_account_key.default_subaddress(),
                view_account_key.subaddress(2),
            ],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(&ledger_db, wallet_db, &account_id, &logger);

        let unverified_txos = Txo::list_unverified(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(unverified_txos.len(), 1);
        assert_eq!(unverified_txos[0].subaddress_index, Some(0));
        assert_eq!(unverified_txos[0].key_image, None);

        let orphaned_txos = Txo::list_orphaned(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(orphaned_txos.len(), 1);
        assert_eq!(orphaned_txos[0].subaddress_index, None);
        assert_eq!(orphaned_txos[0].key_image, None);

        let view_only_account = service.get_account(&account_id).unwrap();
        assert_eq!(view_only_account.next_subaddress_index(&conn).unwrap(), 2);

        let key_image_1 = KeyImage::from(rng.next_u64());
        let key_image_2 = KeyImage::from(rng.next_u64());

        let key_image_1_hex = hex::encode(mc_util_serial::encode(&key_image_1));
        let key_image_2_hex = hex::encode(mc_util_serial::encode(&key_image_2));

        let txo_id_hex_1 = unverified_txos[0].id.clone();
        let txo_id_hex_2 = orphaned_txos[0].id.clone();

        service
            .sync_account(
                &account_id,
                vec![
                    (txo_id_hex_1, key_image_1_hex),
                    (txo_id_hex_2, key_image_2_hex),
                ],
                3,
            )
            .unwrap();

        let view_only_account = service.get_account(&account_id).unwrap();
        assert_eq!(view_only_account.next_subaddress_index(&conn).unwrap(), 3);

        let unverified_txos = Txo::list_unverified(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(unverified_txos.len(), 0);

        let orphaned_txos = Txo::list_orphaned(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(orphaned_txos.len(), 0);

        let unspent_txos = Txo::list_unspent(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(unspent_txos.len(), 2);
    }
}
