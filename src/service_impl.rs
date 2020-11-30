// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods.

use crate::db::WalletDb;
use crate::error::WalletServiceError;
use crate::service_decorated_types::{
    JsonAccount, JsonAddress, JsonBalanceResponse, JsonCreateAccountResponse,
    JsonImportAccountResponse, JsonListTxosResponse, JsonSubmitResponse, JsonTransactionResponse,
    JsonTxo,
};
use crate::sync::SyncThread;
use crate::transaction_builder::WalletTransactionBuilder;
use mc_account_keys::{
    AccountKey, PublicAddress, RootEntropy, RootIdentity, DEFAULT_SUBADDRESS_INDEX,
};
use mc_common::logger::{log, Logger};
use mc_connection::{ConnectionManager, RetryableUserTxConnection, UserTxConnection};
use mc_crypto_rand::rand_core::RngCore;
use mc_fog_report_connection::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_mobilecoind::payments::TxProposal;
use mc_mobilecoind_json::data_types::JsonTxProposal;
use mc_util_from_random::FromRandom;
use std::convert::TryFrom;
use std::iter::empty;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
pub struct WalletService<
    T: UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
    peer_manager: ConnectionManager<T>,
    fog_pubkey_resolver: Option<Arc<FPR>>,
    _sync_thread: SyncThread,
    /// Monotonically increasing counter. This is used for node round-robin selection.
    submit_node_offset: Arc<AtomicUsize>,
    logger: Logger,
}

impl<T: UserTxConnection + 'static, FPR: FogPubkeyResolver + Send + Sync + 'static>
    WalletService<T, FPR>
{
    pub fn new(
        wallet_db: WalletDb,
        ledger_db: LedgerDB,
        peer_manager: ConnectionManager<T>,
        fog_pubkey_resolver: Option<Arc<FPR>>,
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
        let mut rng = rand::thread_rng();
        WalletService {
            wallet_db,
            ledger_db,
            peer_manager,
            fog_pubkey_resolver,
            _sync_thread: sync_thread,
            submit_node_offset: Arc::new(AtomicUsize::new(rng.next_u64() as usize)),
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

    pub fn list_accounts(&self) -> Result<Vec<JsonAccount>, WalletServiceError> {
        Ok(self
            .wallet_db
            .list_accounts()?
            .iter()
            .map(|a| JsonAccount {
                account_id: a.account_id_hex.clone(),
                name: a.name.clone(),
                synced_blocks: a.next_block.to_string(),
            })
            .collect())
    }

    pub fn get_account(&self, account_id_hex: &str) -> Result<JsonAccount, WalletServiceError> {
        let account = self.wallet_db.get_account(account_id_hex)?;
        Ok(JsonAccount {
            account_id: account.account_id_hex.clone(),
            name: account.name.clone(),
            synced_blocks: account.next_block.to_string(),
        })
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

    // FIXME: Would rather return JsonTxo - need to join with AssignedSubaddresses
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

    pub fn get_txo(
        &self,
        account_id_hex: &str,
        txo_id_hex: &str,
    ) -> Result<JsonTxo, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        // FIXME: also add transaction IDs in which this txo was involved
        let (txo, account_txo_status, assigned_subaddress) =
            self.wallet_db.get_txo(account_id_hex, txo_id_hex, &conn)?;
        Ok(JsonTxo::new(
            &txo,
            &account_txo_status,
            &assigned_subaddress,
        ))
    }

    // Balance consists of the sums of the various txo states in our wallet
    // FIXME: We can do more interesting logic here, especially once we have proper change accounting
    // FIXME: Balance for a specific subaddress? Does that need to be exposed? Balance is
    //        somewhat meaningless per subaddress because funds can be moved around arbitrarily.
    pub fn get_balance(
        &self,
        account_id_hex: &str,
    ) -> Result<JsonBalanceResponse, WalletServiceError> {
        let status_map = self.wallet_db.list_txos_by_status(account_id_hex)?;
        let unspent: u64 = status_map["unspent"].iter().map(|t| t.value as u64).sum();
        let pending: u64 = status_map["pending"].iter().map(|t| t.value as u64).sum();
        let spent: u64 = status_map["spent"].iter().map(|t| t.value as u64).sum();
        let unknown: u64 = status_map["unknown"].iter().map(|t| t.value as u64).sum();

        let local_block_height = self.ledger_db.num_blocks()?;

        let account = self.wallet_db.get_account(account_id_hex)?;

        // FIXME: probably also want to compare with network height

        // FIXME: add block height info (see also BEAM wallet-status)
        Ok(JsonBalanceResponse {
            unspent: unspent.to_string(),
            pending: pending.to_string(),
            spent: spent.to_string(),
            unknown: unknown.to_string(),
            local_block_height: local_block_height.to_string(),
            synced_blocks: account.next_block.to_string(),
        })
    }

    pub fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: Option<&str>,
    ) -> Result<JsonAddress, WalletServiceError> {
        let (public_address_b58, subaddress_index) = self
            .wallet_db
            .create_assigned_subaddress(account_id_hex, comment.unwrap_or(""))?;

        // FIXME: have create_assigned_subaddress return the full object
        Ok(JsonAddress {
            public_address_b58,
            subaddress_index: subaddress_index.to_string(),
            address_book_entry_id: None,
            comment: comment.unwrap_or("").to_string(),
        })
    }

    pub fn list_assigned_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonAddress>, WalletServiceError> {
        Ok(self
            .wallet_db
            .list_subaddresses(account_id_hex)?
            .iter()
            .map(|a| JsonAddress::new(a))
            .collect::<Vec<JsonAddress>>())
    }

    pub fn build_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<JsonTxProposal, WalletServiceError> {
        let mut builder = WalletTransactionBuilder::new(
            account_id_hex.to_string(),
            self.wallet_db.clone(),
            self.ledger_db.clone(),
            self.fog_pubkey_resolver.clone(),
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
        } else {
            builder.set_tombstone(0)?;
        }
        if let Some(f) = fee {
            builder.set_fee(f.parse::<u64>()?)?;
        }
        let tx_proposal = builder.build()?;
        // FIXME: Would rather not have to convert it to proto first
        let proto_tx_proposal = mc_mobilecoind_api::TxProposal::from(&tx_proposal);

        // FIXME: Might be nice to have a tx_proposal table so that you don't have to
        //        write these out to local files. That's V2, though.
        Ok(JsonTxProposal::from(&proto_tx_proposal))
    }

    pub fn submit_transaction(
        &self,
        tx_proposal: JsonTxProposal,
        comment: Option<String>,
    ) -> Result<JsonSubmitResponse, WalletServiceError> {
        // Pick a peer to submit to.
        let responder_ids = self.peer_manager.responder_ids();
        if responder_ids.is_empty() {
            return Err(WalletServiceError::NoPeersConfigured);
        }

        let idx = self.submit_node_offset.fetch_add(1, Ordering::SeqCst);
        let responder_id = &responder_ids[idx % responder_ids.len()];

        // FIXME: would prefer not to convert to proto as intermediary
        let tx_proposal_proto = mc_mobilecoind_api::TxProposal::try_from(&tx_proposal)
            .map_err(|e| WalletServiceError::JsonConversion(e))?;

        // Try and submit.
        let tx = mc_transaction_core::tx::Tx::try_from(tx_proposal_proto.get_tx())
            .map_err(|_| WalletServiceError::ProtoConversionInfallible)?;

        let block_height = self
            .peer_manager
            .conn(responder_id)
            .ok_or(WalletServiceError::NodeNotFound)?
            .propose_tx(&tx, empty())
            .map_err(WalletServiceError::from)?;

        log::info!(
            self.logger,
            "Tx {:?} submitted at block height {}",
            tx,
            block_height
        );
        let converted_proposal = TxProposal::try_from(&tx_proposal_proto)?;
        let transaction_id = self.wallet_db.log_submitted_transaction(
            converted_proposal,
            block_height,
            comment.unwrap_or("".to_string()),
        )?;

        // Successfully submitted.
        Ok(JsonSubmitResponse { transaction_id })
    }

    /// Convenience method that builds and submits in one go.
    pub fn send_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    ) -> Result<JsonSubmitResponse, WalletServiceError> {
        let tx_proposal = self.build_transaction(
            account_id_hex,
            recipient_public_address,
            value,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
        )?;
        Ok(self.submit_transaction(tx_proposal, comment)?)
    }

    pub fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonTransactionResponse>, WalletServiceError> {
        let transactions = self.wallet_db.list_transactions(account_id_hex)?;

        let mut results: Vec<JsonTransactionResponse> = Vec::new();
        for (transaction, inputs, outputs, change) in transactions.iter() {
            results.push(JsonTransactionResponse::new(
                &transaction,
                inputs,
                outputs,
                change,
            ));
        }
        Ok(results)
    }

    pub fn get_transaction(
        &self,
        transaction_id_hex: &str,
    ) -> Result<JsonTransactionResponse, WalletServiceError> {
        // FIXME: hack to get around current db access design
        let conn = self.wallet_db.get_conn()?;
        let (transaction, inputs, outputs, change) =
            self.wallet_db.get_transaction(transaction_id_hex, &conn)?;

        Ok(JsonTransactionResponse::new(
            &transaction,
            &inputs,
            &outputs,
            &change,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        add_block_to_ledger_db, get_test_ledger, setup_peer_manager, WalletDbTestContext,
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_common::HashSet;
    use mc_connection_test_utils::MockBlockchainConnection;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use std::iter::FromIterator;
    use std::time::Duration;

    fn setup_service(
        ledger_db: LedgerDB,
        logger: Logger,
    ) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let peer_manager = setup_peer_manager(ledger_db.clone(), logger.clone());
        WalletService::new(
            wallet_db,
            ledger_db,
            peer_manager,
            Some(Arc::new(MockFogPubkeyResolver::new())),
            None,
            logger,
        )
    }

    #[test_with_logger]
    fn test_txo_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db.clone(), logger);
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Add a block with a transaction for this recipient
        // Add a block with a txo for this address (note that value is smaller than MINIMUM_FEE)
        let alice_public_address = b58_decode(&alice.public_address_b58);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(2));

        // Verify balance for Alice
        let balance = service.get_balance(&alice.account_id).unwrap();

        assert_eq!(balance.unspent, "100000000000000");

        // Verify that we have 1 txo
        let txos = service.list_txos(&alice.account_id).unwrap();
        assert_eq!(txos.len(), 1);
        assert_eq!(txos[0].txo_status, "unspent");

        // Add another account
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();

        // Construct a new transaction to Bob
        let tx_proposal = service
            .build_transaction(
                &alice.account_id,
                &bob.public_address_b58,
                "42000000000000".to_string(),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let _submitted = service.submit_transaction(tx_proposal, None).unwrap();

        // We should now have 3 txos - one pending, two minted (one of which will be change)
        let txos = service.list_txos(&alice.account_id).unwrap();
        assert_eq!(txos.len(), 3);
        // The Pending Tx
        let pending: Vec<JsonListTxosResponse> = txos
            .iter()
            .cloned()
            .filter(|t| t.txo_status == "pending")
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].txo_type, "received");
        assert_eq!(pending[0].value, "100000000000000");
        // The Minted have Status Unknown
        let minted: Vec<JsonListTxosResponse> = txos
            .iter()
            .cloned()
            .filter(|t| t.txo_status == "unknown")
            .collect();
        assert_eq!(minted.len(), 2);
        assert_eq!(minted[0].txo_type, "minted");
        assert_eq!(minted[1].txo_type, "minted");
        let minted_value_set = HashSet::from_iter(minted.iter().map(|m| m.value.clone()));
        assert!(minted_value_set.contains("0"));
        assert!(minted_value_set.contains("42000000000000"));

        // Our balance should reflect the various statuses of our txos
        let balance = service.get_balance(&alice.account_id).unwrap();
        assert_eq!(balance.unspent, "0");
        assert_eq!(balance.pending, "100000000000000");
        assert_eq!(balance.spent, "0");
        // In this case, unknown is sent
        assert_eq!(balance.unknown, "42000000000000");

        // FIXME: How to make the transaction actually hit the test ledger?
    }
}
