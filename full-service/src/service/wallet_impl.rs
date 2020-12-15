// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods.

use crate::db::b58_decode;
use crate::{
    db::models::{
        Account, AssignedSubaddress, TransactionLog, Txo, TXO_ORPHANED, TXO_PENDING, TXO_SECRETED,
        TXO_SPENT, TXO_UNSPENT,
    },
    db::WalletDb,
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        transaction_log::TransactionLogModel,
        txo::TxoModel,
    },
    error::WalletServiceError,
    service::{
        decorated_types::{
            JsonAccount, JsonAddress, JsonBalanceResponse, JsonBlock, JsonBlockContents,
            JsonCreateAccountResponse, JsonListTxosResponse, JsonSubmitResponse,
            JsonTransactionLog, JsonTxo, JsonWalletStatus,
        },
        sync::SyncThread,
        transaction_builder::WalletTransactionBuilder,
    },
};
use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
use mc_common::logger::{log, Logger};
use mc_connection::{
    BlockchainConnection, ConnectionManager as McConnectionManager, RetryableUserTxConnection,
    UserTxConnection,
};
use mc_crypto_rand::rand_core::RngCore;
use mc_fog_report_connection::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::{NetworkState, PollingNetworkState};
use mc_mobilecoind::payments::TxProposal;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxProposal};
use mc_transaction_core::tx::{Tx, TxOut, TxOutConfirmationNumber};
use mc_util_from_random::FromRandom;
use serde_json::Map;
use std::{
    convert::TryFrom,
    iter::empty,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

/// Service for interacting with the wallet
pub struct WalletService<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
    peer_manager: McConnectionManager<T>,
    network_state: Arc<RwLock<PollingNetworkState<T>>>,
    fog_pubkey_resolver: Option<Arc<FPR>>,
    _sync_thread: SyncThread,
    /// Monotonically increasing counter. This is used for node round-robin selection.
    submit_node_offset: Arc<AtomicUsize>,
    logger: Logger,
}

impl<
        T: BlockchainConnection + UserTxConnection + 'static,
        FPR: FogPubkeyResolver + Send + Sync + 'static,
    > WalletService<T, FPR>
{
    pub fn new(
        wallet_db: WalletDb,
        ledger_db: LedgerDB,
        peer_manager: McConnectionManager<T>,
        network_state: Arc<RwLock<PollingNetworkState<T>>>,
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
            network_state,
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
        let account_key = AccountKey::from(&root_id);
        let entropy_str = hex::encode(root_id.root_entropy);

        let conn = self.wallet_db.get_conn()?;
        let (account_id, _public_address_b58) = Account::create(
            &account_key,
            first_block,
            None,
            &name.unwrap_or_else(|| "".to_string()),
            &conn,
        )?;

        let local_height = self.ledger_db.num_blocks()?;
        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state.highest_block_index_on_network().unwrap_or(0) + 1;
        let decorated_account =
            Account::get_decorated(&account_id, local_height, network_height, &conn)?;

        Ok(JsonCreateAccountResponse {
            entropy: entropy_str,
            account: decorated_account,
        })
    }

    pub fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<JsonAccount, WalletServiceError> {
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

        let account_id = {
            let conn = self.wallet_db.get_conn()?;
            let (account_id, _public_address_b58) = Account::create(
                &account_key,
                first_block,
                Some(self.ledger_db.num_blocks()?),
                &name.unwrap_or_else(|| "".to_string()),
                &conn,
            )?;
            account_id
        };
        // Use a new conn or else the DB won't submit the previous transaction.
        let account = {
            let local_height = self.ledger_db.num_blocks()?;

            let network_state = self.network_state.read().expect("lock poisoned");
            // network_height = network_block_index + 1
            let network_height = network_state.highest_block_index_on_network().unwrap_or(0) + 1;

            let conn = self.wallet_db.get_conn()?;
            let account = Account::get_decorated(&account_id, local_height, network_height, &conn)?;
            account
        };
        Ok(account)
    }

    pub fn list_accounts(&self) -> Result<Vec<JsonAccount>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let accounts = Account::list_all(&conn)?;
        let local_height = self.ledger_db.num_blocks()?;
        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state.highest_block_index_on_network().unwrap_or(0) + 1;

        accounts
            .iter()
            .map(|a| {
                Account::get_decorated(
                    &AccountID(a.account_id_hex.clone()),
                    local_height,
                    network_height,
                    &conn,
                )
                .map_err(|e| e.into())
            })
            .collect::<Result<Vec<JsonAccount>, WalletServiceError>>()
    }

    pub fn update_account_name(
        &self,
        account_id_hex: &str,
        name: String,
    ) -> Result<(), WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Account::get(&AccountID(account_id_hex.to_string()), &conn)?.update_name(name, &conn)?;
        Ok(())
    }

    pub fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Account::get(&AccountID(account_id_hex.to_string()), &conn)?.delete(&conn)?;
        Ok(())
    }

    pub fn get_account(
        &self,
        account_id_hex: &AccountID,
    ) -> Result<JsonAccount, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let local_height = self.ledger_db.num_blocks()?;
        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state.highest_block_index_on_network().unwrap_or(0) + 1;
        Ok(Account::get_decorated(
            &account_id_hex,
            local_height,
            network_height,
            &conn,
        )?)
    }

    pub fn list_txos(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonListTxosResponse>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let txos = Txo::list_for_account(account_id_hex, &conn)?;
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

        let (txo, account_txo_status, assigned_subaddress) =
            Txo::get(&AccountID(account_id_hex.to_string()), txo_id_hex, &conn)?;
        Ok(JsonTxo::new(
            &txo,
            &account_txo_status,
            assigned_subaddress.as_ref(),
        ))
    }

    // Wallet Status is an overview of the wallet's status
    pub fn get_wallet_status(&self) -> Result<JsonWalletStatus, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let local_height = self.ledger_db.num_blocks()?;

        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state.highest_block_index_on_network().unwrap_or(0) + 1;

        let accounts = Account::list_all(&conn)?;
        let mut account_map = Map::new();

        let mut total_available_pmob = 0;
        let mut total_pending_pmob = 0;
        let mut is_synced_all = true;
        let mut account_ids = Vec::new();
        for account in accounts {
            let decorated = Account::get_decorated(
                &AccountID(account.account_id_hex.clone()),
                local_height,
                network_height,
                &conn,
            )?;
            account_map.insert(
                account.account_id_hex.clone(),
                serde_json::to_value(decorated.clone())?,
            );
            total_available_pmob += decorated.available_pmob.parse::<u64>()?;
            total_pending_pmob += decorated.pending_pmob.parse::<u64>()?;
            is_synced_all = is_synced_all && decorated.is_synced.clone();
            account_ids.push(account.account_id_hex.to_string());
        }

        Ok(JsonWalletStatus {
            object: "wallet_status".to_string(),
            network_height: network_height.to_string(),
            local_height: local_height.to_string(),
            is_synced_all,
            total_available_pmob: total_available_pmob.to_string(),
            total_pending_pmob: total_pending_pmob.to_string(),
            account_ids,
            account_map,
        })
    }

    // Balance consists of the sums of the various txo states in our wallet
    pub fn get_balance(
        &self,
        account_id_hex: &str,
    ) -> Result<JsonBalanceResponse, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let unspent = Txo::list_by_status(account_id_hex, TXO_UNSPENT, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let spent = Txo::list_by_status(account_id_hex, TXO_SPENT, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let secreted = Txo::list_by_status(account_id_hex, TXO_SECRETED, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let orphaned = Txo::list_by_status(account_id_hex, TXO_ORPHANED, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let pending = Txo::list_by_status(account_id_hex, TXO_PENDING, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();

        let local_block_count = self.ledger_db.num_blocks()?;
        let account = Account::get(&AccountID(account_id_hex.to_string()), &conn)?;

        Ok(JsonBalanceResponse {
            unspent: unspent.to_string(),
            pending: pending.to_string(),
            spent: spent.to_string(),
            secreted: secreted.to_string(),
            orphaned: orphaned.to_string(),
            local_block_count: local_block_count.to_string(),
            synced_blocks: account.next_block.to_string(),
        })
    }

    pub fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: Option<&str>,
        // FIXME: WS-32 - add "sync from block"
    ) -> Result<JsonAddress, WalletServiceError> {
        let conn = &self.wallet_db.get_conn()?;
        let (public_address_b58, _subaddress_index) = AssignedSubaddress::create_next_for_account(
            account_id_hex,
            comment.unwrap_or(""),
            &conn,
        )?;

        Ok(JsonAddress::new(&AssignedSubaddress::get(
            &public_address_b58,
            &conn,
        )?))
    }

    pub fn list_assigned_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonAddress>, WalletServiceError> {
        Ok(
            AssignedSubaddress::list_all(account_id_hex, &self.wallet_db.get_conn()?)?
                .iter()
                .map(|a| JsonAddress::new(a))
                .collect::<Vec<JsonAddress>>(),
        )
    }

    #[allow(clippy::too_many_arguments)]
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
        let recipient = b58_decode(recipient_public_address)?;
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
        // FIXME: WS-34 - Would rather not have to convert it to proto first
        let proto_tx_proposal = mc_mobilecoind_api::TxProposal::from(&tx_proposal);

        // FIXME: WS-32 - Might be nice to have a tx_proposal table so that you don't have to
        //        write these out to local files. That's V2, though.
        Ok(JsonTxProposal::from(&proto_tx_proposal))
    }

    pub fn submit_transaction(
        &self,
        tx_proposal: JsonTxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<JsonSubmitResponse, WalletServiceError> {
        // Pick a peer to submit to.
        let responder_ids = self.peer_manager.responder_ids();
        if responder_ids.is_empty() {
            return Err(WalletServiceError::NoPeersConfigured);
        }

        let idx = self.submit_node_offset.fetch_add(1, Ordering::SeqCst);
        let responder_id = &responder_ids[idx % responder_ids.len()];

        // FIXME: WS-34 - would prefer not to convert to proto as intermediary
        let tx_proposal_proto = mc_mobilecoind_api::TxProposal::try_from(&tx_proposal)
            .map_err(WalletServiceError::JsonConversion)?;

        // Try and submit.
        let tx = mc_transaction_core::tx::Tx::try_from(tx_proposal_proto.get_tx())
            .map_err(|_| WalletServiceError::ProtoConversionInfallible)?;

        let block_count = self
            .peer_manager
            .conn(responder_id)
            .ok_or(WalletServiceError::NodeNotFound)?
            .propose_tx(&tx, empty())
            .map_err(WalletServiceError::from)?;

        log::info!(
            self.logger,
            "Tx {:?} submitted at block height {}",
            tx,
            block_count
        );
        let converted_proposal = TxProposal::try_from(&tx_proposal_proto)?;
        let transaction_id = TransactionLog::log_submitted(
            converted_proposal,
            block_count,
            comment.unwrap_or_else(|| "".to_string()),
            account_id_hex.as_deref(),
            &self.wallet_db.get_conn()?,
        )?;

        // Successfully submitted.
        Ok(JsonSubmitResponse { transaction_id })
    }

    /// Convenience method that builds and submits in one go.
    #[allow(clippy::too_many_arguments)]
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
        Ok(self.submit_transaction(tx_proposal, comment, Some(account_id_hex.to_string()))?)
    }

    pub fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonTransactionLog>, WalletServiceError> {
        let transactions = TransactionLog::list_all(account_id_hex, &self.wallet_db.get_conn()?)?;

        let mut results: Vec<JsonTransactionLog> = Vec::new();
        for (transaction, associated_txos) in transactions.iter() {
            results.push(JsonTransactionLog::new(&transaction, &associated_txos));
        }
        Ok(results)
    }

    pub fn get_transaction(
        &self,
        transaction_id_hex: &str,
    ) -> Result<JsonTransactionLog, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction = TransactionLog::get(transaction_id_hex, &conn)?;

        let associated = transaction.get_associated_txos(&conn)?;

        Ok(JsonTransactionLog::new(&transaction, &associated))
    }

    pub fn get_transaction_object(
        &self,
        transaction_id_hex: &str,
    ) -> Result<JsonTx, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction = TransactionLog::get(transaction_id_hex, &conn)?;

        if let Some(tx_bytes) = transaction.tx {
            let tx: Tx = mc_util_serial::decode(&tx_bytes)?;
            // Convert to proto
            let proto_tx = mc_api::external::Tx::from(&tx);
            Ok(JsonTx::from(&proto_tx))
        } else {
            Err(WalletServiceError::NoTxInTransaction)
        }
    }

    pub fn get_txo_object(
        &self,
        account_id_hex: &str,
        txo_id_hex: &str,
    ) -> Result<JsonTxOut, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let (txo, _account_txo_status, _assigned_subaddress) =
            Txo::get(&AccountID(account_id_hex.to_string()), txo_id_hex, &conn)?;

        let txo: TxOut = mc_util_serial::decode(&txo.txo)?;
        // Convert to proto
        let proto_txo = mc_api::external::TxOut::from(&txo);
        Ok(JsonTxOut::from(&proto_txo))
    }

    pub fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(JsonBlock, JsonBlockContents), WalletServiceError> {
        let block = self.ledger_db.get_block(block_index)?;
        let block_contents = self.ledger_db.get_block_contents(block_index)?;
        Ok((
            JsonBlock::new(&block),
            JsonBlockContents::new(&block_contents),
        ))
    }

    pub fn verify_proof(
        &self,
        account_id_hex: &str,
        txo_id_hex: &str,
        proof_hex: &str,
    ) -> Result<bool, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let proof: TxOutConfirmationNumber = mc_util_serial::decode(&hex::decode(proof_hex)?)?;
        Ok(Txo::verify_proof(
            &AccountID(account_id_hex.to_string()),
            &txo_id_hex,
            &proof,
            &conn,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::models::{TXO_MINTED, TXO_RECEIVED},
        test_utils::{
            add_block_to_ledger_db, get_test_ledger, setup_peer_manager_and_network_state,
            WalletDbTestContext,
        },
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
        let (peer_manager, network_state) =
            setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

        WalletService::new(
            wallet_db,
            ledger_db,
            peer_manager,
            network_state,
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
        let alice_public_address = b58_decode(&alice.account.main_address).unwrap();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(5));

        // Verify balance for Alice
        let balance = service.get_balance(&alice.account.account_id).unwrap();

        assert_eq!(balance.unspent, "100000000000000");

        // Verify that we have 1 txo
        let txos = service.list_txos(&alice.account.account_id).unwrap();
        assert_eq!(txos.len(), 1);
        assert_eq!(txos[0].txo_status, TXO_UNSPENT);

        // Add another account
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();

        // Construct a new transaction to Bob
        let tx_proposal = service
            .build_transaction(
                &alice.account.account_id,
                &bob.account.main_address,
                "42000000000000".to_string(),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let _submitted = service
            .submit_transaction(tx_proposal, None, Some(alice.account.account_id.clone()))
            .unwrap();

        // We should now have 3 txos - one pending, two minted (one of which will be change)
        let txos = service.list_txos(&alice.account.account_id).unwrap();
        assert_eq!(txos.len(), 3);
        // The Pending Tx
        let pending: Vec<JsonListTxosResponse> = txos
            .iter()
            .cloned()
            .filter(|t| t.txo_status == TXO_PENDING)
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].txo_type, TXO_RECEIVED);
        assert_eq!(pending[0].value, "100000000000000");
        // The Minted have Status "secreted"
        let minted: Vec<JsonListTxosResponse> = txos
            .iter()
            .cloned()
            .filter(|t| t.txo_status == TXO_SECRETED)
            .collect();
        assert_eq!(minted.len(), 2);
        assert_eq!(minted[0].txo_type, TXO_MINTED);
        assert_eq!(minted[1].txo_type, TXO_MINTED);
        let minted_value_set = HashSet::from_iter(minted.iter().map(|m| m.value.clone()));
        assert!(minted_value_set.contains("57990000000000"));
        assert!(minted_value_set.contains("42000000000000"));

        // Our balance should reflect the various statuses of our txos
        let balance = service.get_balance(&alice.account.account_id).unwrap();
        assert_eq!(balance.unspent, "0");
        assert_eq!(balance.pending, "100000000000000");
        assert_eq!(balance.spent, "0");
        assert_eq!(balance.secreted, "99990000000000");
        assert_eq!(balance.orphaned, "0");

        // FIXME: How to make the transaction actually hit the test ledger?
    }

    // FIXME: Test with 0 change transactions
    // FIXME: Test with balance > u64::max
    // FIXME: sending a transaction with value > u64::max
}
