// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing accounts.

use std::ops::DerefMut;

use crate::{
    db::{
        account::{AccountID, AccountModel},
        exclusive_transaction,
        models::{Account, Txo},
        txo::TxoModel,
        WalletDbError,
    },
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        v2::{api::request::JsonCommandRequest, models::account_key::FogInfo},
    },
    service::{
        hardware_wallet::{
            get_view_only_account_keys, get_view_only_subaddress_keys, HardwareWalletServiceError,
        },
        ledger::{LedgerService, LedgerServiceError},
        WalletService,
    },
};

use base64::{engine::general_purpose, Engine};
use bip39::{Language, Mnemonic, MnemonicType};
use displaydoc::Display;

use mc_account_keys::{
    AccountKey, PublicAddress, RootEntropy, ViewAccountKey, DEFAULT_SUBADDRESS_INDEX,
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_core::{
    account::{RingCtAddress, ViewSubaddress},
    keys::{RootSpendPublic, RootViewPrivate},
};
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_fog_sig_authority::Signer;
use mc_ledger_db::Ledger;
use mc_transaction_signer::types::TxoSynced;

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

    /// Key Error: {0}
    Key(mc_crypto_keys::KeyError),

    /// Error with the HardwareWalletService: {0}
    HardwareWalletService(HardwareWalletServiceError),
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

impl From<mc_crypto_keys::KeyError> for AccountServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::Key(src)
    }
}

impl From<HardwareWalletServiceError> for AccountServiceError {
    fn from(src: HardwareWalletServiceError) -> Self {
        Self::HardwareWalletService(src)
    }
}

/// AccountService trait defining the ways in which the wallet can interact with and manage
#[rustfmt::skip]
#[async_trait]
pub trait AccountService {
    /// Creates a new account with default values.
    ///
    /// # Arguments
    ///
    ///| Name                 | Purpose                                | Notes                                                            |
    ///|----------------------|----------------------------------------|------------------------------------------------------------------|
    ///| `name`               | A label for this account.              | A label can have duplicates, but it is not recommended.          |
    ///| `fog_report_url`     | Fog Report server url.                 | Applicable only if user has Fog service, empty string otherwise. |
    ///| `fog_authority_spki` | Fog Authority Subject Public Key Info. | Applicable only if user has Fog service, empty string otherwise. |
    ///| `require_spend_subaddress` | Spend only from subaddress.    | Only allow the account to spend from give subaddresses.          |
    ///
    fn create_account(
        &self,
        name: Option<String>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the mnemonic.
    ///
    /// # Arguments
    ///
    ///| Name                     | Purpose                                                                                    | Notes                                                            |
    ///|--------------------------|--------------------------------------------------------------------------------------------|------------------------------------------------------------------|
    ///| `mnemonic_phrase`        | The secret mnemonic to recover the account.                                                | A label can have duplicates, but it is not recommended.          |
    ///| `name`                   | A Optional label for this account.                                                         |                                                                  |
    ///| `first_block_index`      | The block from which to start scanning the ledger.                                         | All subaddresses below this index will be created.               |
    ///| `next_subaddress_index`  | The next known unused subaddress index for the account.                                    |                                                                  |
    ///| `fog_report_url`         | Fog Report server url.                                                                     | Applicable only if user has Fog service, empty string otherwise. |
    ///| `fog_authority_spki`     | Fog Authority Subject Public Key Info.                                                     | Applicable only if user has Fog service, empty string otherwise. |
    ///| `require_spend_subaddress` | Spend only from subaddress.    | Only allow the account to spend from give subaddresses.          |
    ///
    #[allow(clippy::too_many_arguments)]
    fn import_account(
        &self,
        mnemonic_phrase: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the entropy.
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                 | Notes                                                            |
    ///|-------------------------|---------------------------------------------------------|------------------------------------------------------------------|
    ///| `entropy`               | The secret root entropy.                                | 32 bytes of randomness, hex-encoded.                             |
    ///| `name`                  | A label for this account.                               | A label can have duplicates, but it is not recommended.          |
    ///| `first_block_index`     | The block from which to start scanning the ledger.      | All subaddresses below this index will be created.               |
    ///| `next_subaddress_index` | The next known unused subaddress index for the account. |                                                                  |
    ///| `fog_report_url`        | Fog Report server url.                                  | Applicable only if user has Fog service, empty string otherwise. |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                  | Applicable only if user has Fog service, empty string otherwise. |
    ///| `require_spend_subaddress` | Spend only from subaddress.    | Only allow the account to spend from give subaddresses.          |
    ///
    #[allow(clippy::too_many_arguments)]
    fn import_account_from_legacy_root_entropy(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the mnemonic.
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                 | Notes                                                   |
    ///|-------------------------|---------------------------------------------------------|---------------------------------------------------------|
    ///| `view_private_key`      | The view private key of this account                    |                                                         |
    ///| `spend_public_key`      | The spend public key of this account                    |                                                         |
    ///| `name`                  | A label for this account.                               | A label can have duplicates, but it is not recommended. |
    ///| `first_block_index`     | The block from which to start scanning the ledger.      | All subaddresses below this index will be created.      |
    ///| `next_subaddress_index` | The next known unused subaddress index for the account. |                                                         |
    ///
    fn import_view_only_account(
        &self,
        view_private_key: &RootViewPrivate,
        spend_public_key: &RootSpendPublic,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError>;

    async fn import_view_only_account_from_hardware_wallet(
        &self,
        name: Option<String>,
        first_block_index: Option<u64>,
        fog_info: Option<FogInfo>,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError>;

    /// Re-create sync request for a view only account
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                                                    |
    ///|--------------|----------------------------------------------|----------------------------------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet as a view only account. |
    ///
    fn resync_account(
        &self, 
        account_id: &AccountID
    ) -> Result<(), AccountServiceError>;

    /// Create an import request for a view only account
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///
    fn get_view_only_account_import_request(
        &self,
        account_id: &AccountID,
    ) -> Result<JsonRPCRequest, AccountServiceError>;

    /// List details of all accounts in a given wallet.
    ///
    /// # Arguments
    ///
    ///| Name     | Purpose                                                    | Notes                      |
    ///|----------|------------------------------------------------------------|----------------------------|
    ///| `offset` | The pagination offset. Results start at the offset index.  | Optional, defaults to 0.   |
    ///| `limit`  | Limit for the number of results.                           | Optional                   |
    ///
    fn list_accounts(
        &self,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Account>, AccountServiceError>;

    /// Get the current status of a given account. The account status includes both the account object and the balance object.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///
    fn get_account(
        &self, 
        account_id: &AccountID
    ) -> Result<Account, AccountServiceError>;

    /// Get the next subaddress index for an account
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///
    fn get_next_subaddress_index_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<u64, AccountServiceError>;

    /// Update the name for an account.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///| `name`       | The new name for this account.               |                                   |
    ///
    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, AccountServiceError>;

    /// Update the require_spend_subaddress field for an account.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///| `require_spend_subaddress` | Whether to enable require_spend_subaddress mode |                  |
    ///
    fn update_require_spend_subaddress(
        &self,
        account_id: &AccountID,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError>;

    /// complete a sync request for a view only account
    ///
    /// # Arguments
    ///
    ///| Name                     | Purpose                                                      | Notes                                                    |
    ///|--------------------------|--------------------------------------------------------------|----------------------------------------------------------|
    ///| `account_id`             | The account on which to perform this action.                 | Account must exist in the wallet as a view only account. |
    ///| `synced_txos` | An array of TxoSynced objects (TxOutPublic, KeyImage)             |                                                          |
    ///
    fn sync_account(
        &self,
        account_id: &AccountID,
        synced_txos: Vec<TxoSynced>,
    ) -> Result<(), AccountServiceError>;

    /// Remove an account from the wallet.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///| `name`       | The new name for this account.               |                                   |
    ///
    fn remove_account(
        &self, 
        account_id: &AccountID
    ) -> Result<bool, AccountServiceError>;

    fn resync_in_progress(&self) -> Result<bool, AccountServiceError>;
}

#[async_trait]
impl<T, FPR> AccountService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn create_account(
        &self,
        name: Option<String>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
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

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();

        exclusive_transaction(conn, |conn| {
            let (account_id, _public_address_b58) = Account::create_from_mnemonic(
                &mnemonic,
                Some(first_block_index),
                Some(import_block_index),
                None,
                &name.unwrap_or_default(),
                fog_report_url,
                fog_authority_spki,
                require_spend_subaddress,
                conn,
            )?;
            let account = Account::get(&account_id, conn)?;
            Ok(account)
        })
    }

    fn import_account(
        &self,
        mnemonic_phrase: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

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

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        exclusive_transaction(conn, |conn| {
            Ok(Account::import(
                &mnemonic,
                name,
                import_block,
                first_block_index,
                next_subaddress_index,
                fog_report_url,
                fog_authority_spki,
                require_spend_subaddress,
                conn,
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
        fog_authority_spki: String,
        require_spend_subaddress: bool,
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

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        exclusive_transaction(conn, |conn| {
            Ok(Account::import_legacy(
                &RootEntropy::from(&entropy_bytes),
                name,
                import_block,
                first_block_index,
                next_subaddress_index,
                fog_report_url,
                fog_authority_spki,
                require_spend_subaddress,
                conn,
            )?)
        })
    }

    fn import_view_only_account(
        &self,
        view_private_key: &RootViewPrivate,
        spend_public_key: &RootSpendPublic,
        name: Option<String>,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing view only account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let import_block_index = self.ledger_db.num_blocks()? - 1;

        let view_account_key =
            ViewAccountKey::new(*view_private_key.as_ref(), *spend_public_key.as_ref());

        exclusive_transaction(conn, |conn| {
            Ok(Account::import_view_only(
                &view_account_key,
                name,
                import_block_index,
                first_block_index,
                next_subaddress_index,
                false,
                require_spend_subaddress,
                conn,
            )?)
        })
    }

    async fn import_view_only_account_from_hardware_wallet(
        &self,
        name: Option<String>,
        first_block_index: Option<u64>,
        fog_info: Option<FogInfo>,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError> {
        let view_account = get_view_only_account_keys().await?;

        let view_account_keys = ViewAccountKey::new(
            *view_account.view_private_key().as_ref(),
            *view_account.spend_public_key().as_ref(),
        );

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let import_block_index = self.ledger_db.num_blocks()? - 1;

        match fog_info {
            Some(fog_info) => {
                let fog_authority_spki =
                    general_purpose::STANDARD.decode(fog_info.authority_spki)?;
                let default_subaddress_keys =
                    get_view_only_subaddress_keys(DEFAULT_SUBADDRESS_INDEX).await?;

                let default_public_address = get_public_fog_address(
                    &default_subaddress_keys,
                    fog_info.report_url,
                    &fog_authority_spki,
                );
                exclusive_transaction(conn, |conn| {
                    Ok(Account::import_view_only_from_hardware_wallet_with_fog(
                        &view_account_keys,
                        name,
                        import_block_index,
                        first_block_index,
                        &default_public_address,
                        require_spend_subaddress,
                        conn,
                    )?)
                })
            }
            None => exclusive_transaction(conn, |conn| {
                Ok(Account::import_view_only(
                    &view_account_keys,
                    name,
                    import_block_index,
                    first_block_index,
                    None,
                    true,
                    false,
                    conn,
                )?)
            }),
        }
    }

    fn resync_account(&self, account_id: &AccountID) -> Result<(), AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let account = Account::get(account_id, conn)?;
        account.update_next_block_index(account.first_block_index as u64, conn)?;
        Ok(())
    }

    fn get_view_only_account_import_request(
        &self,
        account_id: &AccountID,
    ) -> Result<JsonRPCRequest, AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let account = Account::get(account_id, conn)?;

        if account.view_only {
            return Err(AccountServiceError::AccountIsViewOnly(account_id.clone()));
        }

        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        let view_private_key = account_key.view_private_key();
        let spend_public_key = RistrettoPublic::from(account_key.spend_private_key());

        let json_command_request = JsonCommandRequest::import_view_only_account {
            view_private_key: hex::encode(view_private_key.to_bytes()),
            spend_public_key: hex::encode(spend_public_key.to_bytes()),
            name: Some(account.name.clone()),
            first_block_index: Some(account.first_block_index.to_string()),
            next_subaddress_index: Some(account.next_subaddress_index(conn)?.to_string()),
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
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        Ok(Account::list_all(conn, offset, limit)?)
    }

    fn get_account(&self, account_id: &AccountID) -> Result<Account, AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        Ok(Account::get(account_id, conn)?)
    }

    fn get_next_subaddress_index_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<u64, AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let account = Account::get(account_id, conn)?;
        Ok(account.next_subaddress_index(conn)?)
    }

    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        Account::get(account_id, conn)?.update_name(name, conn)?;
        Ok(Account::get(account_id, conn)?)
    }

    fn update_require_spend_subaddress(
        &self,
        account_id: &AccountID,
        require_spend_subaddress: bool,
    ) -> Result<Account, AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        Account::get(account_id, conn)?
            .update_require_spend_subaddress(require_spend_subaddress, conn)?;
        Ok(Account::get(account_id, conn)?)
    }

    fn sync_account(
        &self,
        account_id: &AccountID,
        synced_txos: Vec<TxoSynced>,
    ) -> Result<(), AccountServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let account = Account::get(account_id, conn)?;

        if !account.view_only {
            return Err(AccountServiceError::AccountIsNotViewOnly(
                account_id.clone(),
            ));
        }

        for synced_txo in synced_txos {
            let spent_block_index = self.ledger_db.check_key_image(&synced_txo.key_image)?;
            let ristretto_public: &RistrettoPublic = synced_txo.tx_out_public_key.as_ref();
            Txo::update_key_image_by_pubkey(
                &ristretto_public.into(),
                &synced_txo.key_image,
                spent_block_index,
                conn,
            )?;
        }

        Ok(())
    }

    fn remove_account(&self, account_id: &AccountID) -> Result<bool, AccountServiceError> {
        log::info!(self.logger, "Deleting account {}", account_id,);
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();

        exclusive_transaction(conn, |conn| {
            let account = Account::get(account_id, conn)?;
            account.delete(conn)?;
            Ok(true)
        })
    }

    fn resync_in_progress(&self) -> Result<bool, AccountServiceError> {
        let mut pooled_conn = match self.get_pooled_conn() {
            Ok(pooled_conn) => Ok(pooled_conn),
            Err(WalletDbError::WalletFunctionsDisabled) => return Ok(false),
            Err(err) => Err(err),
        }?;

        let conn = pooled_conn.deref_mut();
        Ok(Account::resync_in_progress(conn)?)
    }
}

fn get_public_fog_address(
    subaddress_keys: &ViewSubaddress,
    fog_report_url: String,
    fog_authority_spki_bytes: &[u8],
) -> PublicAddress {
    let fog_authority_sig = {
        let sig = subaddress_keys
            .view_private
            .as_ref()
            .sign_authority(fog_authority_spki_bytes)
            .unwrap();
        let sig_bytes: &[u8] = sig.as_ref();
        sig_bytes.to_vec()
    };

    let subaddress_view_public = subaddress_keys.view_public_key();
    let subaddress_spend_public = subaddress_keys.spend_public_key();

    PublicAddress::new_with_fog(
        subaddress_spend_public.as_ref(),
        subaddress_view_public.as_ref(),
        fog_report_url,
        "".to_string(),
        fog_authority_sig,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{models::Txo, txo::TxoModel},
        service::address::AddressService,
        test_utils::{
            add_block_to_ledger_db, create_test_received_txo, generate_n_blocks_on_ledger,
            get_empty_test_ledger, get_test_ledger, manually_sync_account, setup_wallet_service,
            setup_wallet_service_offline, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, ViewAccountKey};
    use mc_common::logger::{async_test_with_logger, test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPrivate;
    use mc_rand::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Amount, Token};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::convert::{TryFrom, TryInto};

    #[test]
    fn test_get_public_fog_address() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let spend_private_key = RistrettoPrivate::from_random(&mut rng);
        let fog_report_url = "fog://fog.test.mobilecoin.com".to_string();
        let fog_authority_spki = "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ==".as_bytes().to_vec();

        let account_key = AccountKey::new_with_fog(
            &spend_private_key,
            &view_private_key,
            fog_report_url.clone(),
            "".to_string(),
            fog_authority_spki.clone(),
        );

        let public_address_from_account_key = account_key.default_subaddress();
        let default_subaddress_view_private = account_key.default_subaddress_view_private();
        let default_subaddress_spend_private = account_key.default_subaddress_spend_private();
        let default_subaddress_spend_public: RistrettoPublic =
            (&default_subaddress_spend_private).into();

        let default_view_subaddress = ViewSubaddress {
            view_private: default_subaddress_view_private.into(),
            spend_public: default_subaddress_spend_public.into(),
        };

        let public_address_from_view_subaddress = get_public_fog_address(
            &default_view_subaddress,
            fog_report_url,
            fog_authority_spki.as_ref(),
        );

        assert_eq!(
            public_address_from_account_key,
            public_address_from_view_subaddress
        );
    }

    #[test_with_logger]
    fn test_resync_account(logger: Logger) {
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
                false,
            )
            .unwrap();
        let account_id = AccountID(account.id);

        let account = service.get_account(&account_id).unwrap();
        assert_eq!(account.first_block_index, 0);
        assert_eq!(account.next_block_index, account.first_block_index);

        manually_sync_account(&ledger_db, wallet_db, &account_id, &service.logger);
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(
            account.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );

        service.resync_account(&account_id).unwrap();
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(account.next_block_index, account.first_block_index);
        manually_sync_account(&ledger_db, wallet_db, &account_id, &service.logger);
        let account = service.get_account(&account_id).unwrap();
        assert_eq!(
            account.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );

        // create an account that has its first_block_index set to later in the ledger
        let account2 = service
            .create_account(None, "".to_string(), "".to_string(), false)
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

        manually_sync_account(&ledger_db, wallet_db, &account_id, &service.logger);
        let account2 = service.get_account(&account_id).unwrap();
        assert_eq!(
            account2.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );

        service.resync_account(&account_id).unwrap();
        let account2 = service.get_account(&account_id).unwrap();
        assert_eq!(account2.next_block_index, account2.first_block_index);

        manually_sync_account(&ledger_db, wallet_db, &account_id, &service.logger);
        let account2 = service.get_account(&account_id).unwrap();
        assert_eq!(
            account2.next_block_index as u64,
            ledger_db.num_blocks().unwrap()
        );
    }

    #[async_test_with_logger]
    async fn test_resync_account_badly_stored_txo(logger: Logger) {
        use crate::{
            db::{
                account::AccountID,
                models::TransactionLog,
                schema::txos,
                transaction_log::{TransactionId, TransactionLogModel},
            },
            test_utils::{
                add_block_with_tx_outs, create_test_minted_and_change_txos,
                create_test_txo_for_recipient,
            },
        };
        use diesel::prelude::*;
        use rand::{seq::SliceRandom, thread_rng};

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let entropy_a = RootEntropy::from_random(&mut rng);
        let account_a_key = AccountKey::from(&RootIdentity::from(&entropy_a));
        let entropy_b = RootEntropy::from_random(&mut rng);
        let account_b_key = AccountKey::from(&RootIdentity::from(&entropy_a));

        let initial_block_count = 12;
        let mut ledger_db = get_test_ledger(5, &[], initial_block_count, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let wallet_db = service.wallet_db.as_ref().unwrap();

        let account_a = service
            .import_account_from_legacy_root_entropy(
                hex::encode(entropy_a.bytes),
                None,
                None,
                None,
                "".to_string(),
                "".to_string(),
                false,
            )
            .unwrap();
        let account_a_id = AccountID(account_a.id.clone());

        let account_b = service
            .import_account_from_legacy_root_entropy(
                hex::encode(entropy_b.bytes),
                None,
                None,
                None,
                "".to_string(),
                "".to_string(),
                false,
            )
            .unwrap();
        let account_b_id = AccountID(account_b.id.clone());

        // Create TXO for Alice
        let (for_acc_a_txo, _) = create_test_txo_for_recipient(
            &account_a_key,
            0,
            mc_transaction_core::Amount::new(1000 * MOB, Mob::ID),
            &mut rng,
        );

        // Let's add this txo to the ledger
        add_block_with_tx_outs(
            &mut ledger_db,
            &[for_acc_a_txo.clone()],
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        manually_sync_account(&ledger_db, wallet_db, &account_a_id, &logger);

        /* Send the transaction after corrupting the txo entry */
        // build and sign a transaction
        //  - this should build and store the txos in the db
        //  - should also create the transaction logs

        let wallet_db = service.wallet_db.as_ref().unwrap();
        let (transaction_log, tx_proposal) = create_test_minted_and_change_txos(
            account_a_key.clone(),
            account_b_key.subaddress(0),
            72 * MOB,
            wallet_db.clone(),
            ledger_db.clone(),
        )
        .await;

        // Store the real values for the txo's amount and target_key (arbitrary fields
        // we want to corrupt and sync back)
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let associated_txos = transaction_log.get_associated_txos(conn).unwrap();
        let expected_txo_amount = associated_txos.outputs[0].0.value;
        let expected_target_key = associated_txos.outputs[0].0.target_key.clone();

        let for_b_key_image: KeyImage =
            mc_util_serial::decode(&associated_txos.inputs[0].key_image.clone().unwrap()).unwrap();

        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                tx_proposal.change_txos[0].tx_out.clone(),
                tx_proposal.payload_txos[0].tx_out.clone(),
            ],
            &[for_b_key_image],
            &mut rng,
        );
        // Submit the transaction
        TransactionLog::log_submitted(
            &tx_proposal,
            (initial_block_count + 1).try_into().unwrap(),
            "".to_string(),
            &AccountID::from(&account_a_key).to_string(),
            conn,
        )
        .unwrap();

        // manually sync the account
        manually_sync_account(&ledger_db, wallet_db, &account_a_id, &logger);
        manually_sync_account(&ledger_db, wallet_db, &account_b_id, &logger);

        // manually overwrite the amount and target_key of the output txo to
        // something bogus
        let corrupted_txo_amount = expected_txo_amount << 4;
        let mut corrupted_target_key = expected_target_key.clone();
        corrupted_target_key.shuffle(&mut thread_rng());
        assert_ne!(expected_txo_amount, corrupted_txo_amount);
        assert_ne!(expected_target_key, corrupted_target_key);
        diesel::update(&associated_txos.outputs[0].0)
            .set((
                txos::value.eq(corrupted_txo_amount),
                txos::target_key.eq(&corrupted_target_key),
            ))
            .execute(conn)
            .unwrap();

        let associated_txos = transaction_log.get_associated_txos(conn).unwrap();
        assert_ne!(expected_txo_amount, associated_txos.outputs[0].0.value);
        assert_ne!(expected_target_key, associated_txos.outputs[0].0.target_key);

        // resync the account
        service.resync_account(&account_a_id).unwrap();
        service.resync_account(&account_b_id).unwrap();
        manually_sync_account(&ledger_db, wallet_db, &account_a_id, &logger);
        manually_sync_account(&ledger_db, wallet_db, &account_b_id, &logger);

        //  - check that the txo we futzed with is now stored correctly
        //  - check that the transaction log entries are exactly the same as before
        let associated_txos = transaction_log.get_associated_txos(conn).unwrap();
        assert_eq!(expected_txo_amount, associated_txos.outputs[0].0.value);
        assert_eq!(expected_target_key, associated_txos.outputs[0].0.target_key);
        let transaction_log_new =
            TransactionLog::get(&TransactionId(transaction_log.id.clone()), conn).unwrap();
        // We check every field in the struct except for the finalized index field
        // because we expect it to be different
        assert_eq!(transaction_log_new.id, transaction_log.id);
        assert_eq!(transaction_log_new.account_id, transaction_log.account_id);
        assert_eq!(transaction_log_new.fee_value, transaction_log.fee_value);
        assert_eq!(
            transaction_log_new.fee_token_id,
            transaction_log.fee_token_id
        );
        assert_eq!(
            transaction_log_new.submitted_block_index,
            transaction_log.submitted_block_index
        );
        assert_eq!(
            transaction_log_new.tombstone_block_index,
            transaction_log.tombstone_block_index
        );
        assert_eq!(transaction_log_new.comment, transaction_log.comment);
        assert_eq!(transaction_log_new.tx, transaction_log.tx);
        assert_eq!(transaction_log_new.failed, transaction_log.failed);
    }

    #[test_with_logger]
    fn test_remove_account_from_txo(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db, logger);
        let wallet_db = &service.wallet_db.as_ref().unwrap();

        // Create an account.
        let account = service
            .create_account(Some("A".to_string()), "".to_string(), "".to_string(), false)
            .unwrap();

        // Add a transaction, with transaction status.
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key).unwrap();

        create_test_received_txo(
            &account_key,
            0,
            Amount::new(100 * MOB, Mob::ID),
            13_u64,
            &mut rng,
            wallet_db,
        );

        let txos = Txo::list_for_account(
            &account.id,
            None,
            None,
            None,
            None,
            None,
            Some(0),
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        // Delete the account. The transaction status referring to it is also cleared.
        let account_id = AccountID(account.id.clone());
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
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
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
            .create_account(Some("A".to_string()), "".to_string(), "".to_string(), false)
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
            service.wallet_db.as_ref().unwrap(),
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
            .create_account(Some("A".to_string()), "".to_string(), "".to_string(), false)
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
            service.wallet_db.as_ref().unwrap(),
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
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let spend_private_key = RistrettoPrivate::from_random(&mut rng);

        let account_key = AccountKey::new(&spend_private_key, &view_private_key);
        let view_account_key = ViewAccountKey::from(&account_key);

        let view_only_account = service
            .import_view_only_account(
                &(*view_account_key.view_private_key()).into(),
                &(*view_account_key.spend_public_key()).into(),
                None,
                None,
                None,
                false,
            )
            .unwrap();

        let account_id = AccountID(view_only_account.id);

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![
                view_account_key.default_subaddress(),
                view_account_key.subaddress(2),
            ],
            100 * MOB,
            &[KeyImage::from(rng.next_u64())],
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
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
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
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(orphaned_txos.len(), 1);
        assert_eq!(orphaned_txos[0].subaddress_index, None);
        assert_eq!(orphaned_txos[0].key_image, None);

        let view_only_account = service.get_account(&account_id).unwrap();
        assert_eq!(view_only_account.next_subaddress_index(conn).unwrap(), 2);

        let txo_synced_1 = TxoSynced {
            tx_out_public_key: RistrettoPublic::try_from(&unverified_txos[0].public_key().unwrap())
                .unwrap()
                .into(),
            key_image: KeyImage::from(rng.next_u64()),
        };

        service
            .sync_account(&account_id, vec![txo_synced_1])
            .unwrap();

        let view_only_account = service.get_account(&account_id).unwrap();
        assert_eq!(view_only_account.next_subaddress_index(conn).unwrap(), 2);

        let unverified_txos = Txo::list_unverified(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
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
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(orphaned_txos.len(), 1);

        let unspent_txos = Txo::list_unspent(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(unspent_txos.len(), 1);

        service
            .assign_address_for_account(&account_id, None)
            .unwrap();

        let unverified_txos = Txo::list_unverified(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(unverified_txos.len(), 1);

        let txo_synced_2 = TxoSynced {
            tx_out_public_key: RistrettoPublic::try_from(&orphaned_txos[0].public_key().unwrap())
                .unwrap()
                .into(),
            key_image: KeyImage::from(rng.next_u64()),
        };

        service
            .sync_account(&account_id, vec![txo_synced_2])
            .unwrap();

        let view_only_account = service.get_account(&account_id).unwrap();
        assert_eq!(view_only_account.next_subaddress_index(conn).unwrap(), 3);

        let unverified_txos = Txo::list_unverified(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(unverified_txos.len(), 0);

        let unspent_txos = Txo::list_unspent(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(unspent_txos.len(), 2);
    }
}
