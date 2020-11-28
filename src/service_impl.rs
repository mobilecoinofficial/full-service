// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods.

use crate::db::WalletDb;
use crate::error::WalletServiceError;
use crate::service_decorated_types::{
    JsonCreateAccountResponse, JsonImportAccountResponse, JsonListTxosResponse,
};
use crate::sync::SyncThread;
use crate::transaction_builder::WalletTransactionBuilder;
use mc_account_keys::{
    AccountKey, PublicAddress, RootEntropy, RootIdentity, DEFAULT_SUBADDRESS_INDEX,
};
use mc_common::logger::{log, Logger};
use mc_mobilecoind_json::data_types::JsonTxProposal;
// use mc_connection::{ConnectionManager, RetryableUserTxConnection, UserTxConnection};
use mc_ledger_db::LedgerDB;
use mc_util_from_random::FromRandom;
// use std::sync::Arc;
use std::convert::TryFrom;

pub const DEFAULT_CHANGE_SUBADDRESS_INDEX: u64 = 1;
pub const DEFAULT_NEXT_SUBADDRESS_INDEX: u64 = 2;
pub const DEFAULT_FIRST_BLOCK: u64 = 0;

// FIXME: this will probably live in db or service_impl once we're decoding public addresses
pub fn b58_decode(b58_public_address: &str) -> PublicAddress {
    let wrapper =
        mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(b58_public_address.to_string())
            .unwrap();
    let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
        let payment_request = wrapper.get_payment_request();
        payment_request.get_public_address()
    } else if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        panic!("No public address in wrapper");
    };
    PublicAddress::try_from(pubaddr_proto).unwrap()
}

/// Service for interacting with the wallet
pub struct WalletService {
    //<
    //     T: UserTxConnection + 'static,
    //     FPR: FogPubkeyResolver + Send + Sync + 'static,
    // > {
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
    // peer_manager: ConnectionManager<T>,
    // fog_pubkey_resolver: Option<Arc<FPR>>,
    _sync_thread: SyncThread,
    logger: Logger,
}

impl WalletService
//<T: UserTxConnection + 'static, FPR: FogPubkeyResolver + Send + Sync + 'static>
//     WalletService<T, FPR>
{
    pub fn new(
        wallet_db: WalletDb,
        ledger_db: LedgerDB,
        // peer_manager: Connectionmanager<T>,
        // fog_pubkey_resolver: Option<Arc<FPR>>,
        num_workers: Option<usize>,
        logger: Logger,
    ) -> Self {
        log::info!(logger, "Starting Wallet TXO Sync Task Thread");
        let sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            num_workers,
            logger.clone(),
        );
        WalletService {
            wallet_db,
            ledger_db,
            // peer_manager,
            // fog_pubkey_resolver,
            _sync_thread: sync_thread,
            logger,
        }
    }

    /// Creates a new account with defaults
    pub fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<JsonCreateAccountResponse, WalletServiceError> {
        log::info!(
            self.logger,
            "Creating account {:?} with first_block: {:?}",
            name,
            first_block,
        );
        // Generate entropy for the account
        let mut rng = rand::thread_rng();
        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id.clone());
        let entropy_str = hex::encode(root_id.root_entropy);

        let fb = first_block.unwrap_or(DEFAULT_FIRST_BLOCK);
        let (account_id, public_address_b58) = self.wallet_db.create_account(
            &account_key,
            DEFAULT_SUBADDRESS_INDEX,
            DEFAULT_CHANGE_SUBADDRESS_INDEX,
            DEFAULT_NEXT_SUBADDRESS_INDEX,
            fb,
            fb + 1,
            &name.unwrap_or("".to_string()),
        )?;

        Ok(JsonCreateAccountResponse {
            entropy: entropy_str.to_string(),
            public_address_b58,
            account_id,
        })
    }

    pub fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<JsonImportAccountResponse, WalletServiceError> {
        log::info!(
            self.logger,
            "Importing account {:?} with first_block: {:?}",
            name,
            first_block,
        );
        // Get account key from entropy
        let mut entropy_bytes = [0u8; 32];
        hex::decode_to_slice(entropy, &mut entropy_bytes)?;
        let account_key = AccountKey::from(&RootIdentity::from(&RootEntropy::from(&entropy_bytes)));

        let fb = first_block.unwrap_or(DEFAULT_FIRST_BLOCK);
        let (account_id, public_address_b58) = self.wallet_db.create_account(
            &account_key,
            DEFAULT_SUBADDRESS_INDEX,
            DEFAULT_CHANGE_SUBADDRESS_INDEX,
            DEFAULT_NEXT_SUBADDRESS_INDEX,
            fb,
            fb + 1,
            &name.unwrap_or("".to_string()),
        )?;
        Ok(JsonImportAccountResponse {
            public_address_b58,
            account_id,
        })
    }

    pub fn list_accounts(&self) -> Result<Vec<String>, WalletServiceError> {
        Ok(self
            .wallet_db
            .list_accounts()?
            .iter()
            .map(|a| a.account_id_hex.clone())
            .collect())
    }

    pub fn get_account(&self, account_id_hex: &str) -> Result<String, WalletServiceError> {
        let account = self.wallet_db.get_account(account_id_hex)?;
        Ok(account.name)
    }

    pub fn update_account_name(
        &self,
        account_id_hex: &str,
        name: String,
    ) -> Result<(), WalletServiceError> {
        self.wallet_db.update_account_name(account_id_hex, name)?;
        Ok(())
    }

    pub fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletServiceError> {
        self.wallet_db.delete_account(account_id_hex)?;
        Ok(())
    }

    pub fn list_txos(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonListTxosResponse>, WalletServiceError> {
        let txos = self.wallet_db.list_txos(account_id_hex)?;
        Ok(txos
            .iter()
            .map(|(t, s)| JsonListTxosResponse::new(t, s))
            .collect())
    }

    pub fn get_balance(&self, account_id_hex: &str) -> Result<u64, WalletServiceError> {
        let txos = self.wallet_db.list_unspent_txos(account_id_hex)?;
        Ok(txos.iter().map(|t| t.value as u64).sum())
    }

    pub fn build_transaction(
        &self,
        account_id_hex: &str,
        input_txo_ids: Option<&Vec<String>>,
        recipient_public_address: &str,
        value: String,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<JsonTxProposal, WalletServiceError> {
        let mut builder = WalletTransactionBuilder::new(
            account_id_hex.to_string(),
            self.wallet_db.clone(),
            self.ledger_db.clone(),
            self.logger.clone(),
        );
        let recipient = b58_decode(recipient_public_address);
        builder.add_recipient(recipient, value.parse::<u64>()?)?;
        if let Some(inputs) = input_txo_ids {
            builder.set_txos(inputs)?;
        } else {
            let max_spendable = if let Some(msv) = max_spendable_value {
                Some(msv.parse::<u64>()?)
            } else {
                None
            };
            builder.select_txos(max_spendable)?;
        }
        if let Some(tombstone) = tombstone_block {
            builder.set_tombstone(tombstone.parse::<u64>()?)?;
        }
        if let Some(f) = fee {
            builder.set_fee(f.parse::<u64>()?)?;
        }
        let tx_proposal = builder.build()?;
        // FIXME: Would rather not have to convert it to proto first
        let proto_tx_proposal = mc_mobilecoind_api::TxProposal::from(&tx_proposal);
        Ok(JsonTxProposal::from(&proto_tx_proposal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_test_ledger, WalletDbTestContext};
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    fn setup_service(ledger_db: LedgerDB, logger: Logger) -> WalletService {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());

        WalletService::new(wallet_db, ledger_db, None, logger)
    }

    #[test_with_logger]
    fn test_create_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db, logger);
        let _account_details = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // FIXME: TODO - assert other things that should be true with the service state
        //        after an account has been created, such as the balance, etc
    }
}
