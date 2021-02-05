// Copyright (c) 2020-2021 MobileCoin Inc.

//! Application-layer Wallet service.

use crate::db::{
    self, b58_decode, Account, AccountModel, AssignedSubaddress, TransactionLog,
    TransactionLogModel, Txo, TxoModel, TXO_ORPHANED, TXO_PENDING, TXO_SECRETED, TXO_SPENT,
    TXO_UNSPENT,
};
use crate::json_rpc::{BalanceResponse, MembershipProof};
use crate::service::wallet_trait::Wallet;
use crate::service::WalletServiceError;
use crate::{
    db::WalletDb,
    json_rpc,
    service::{sync::SyncThread, transaction_builder::WalletTransactionBuilder},
};
use diesel::prelude::*;
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

use crate::db::assigned_subaddress::AssignedSubaddressModel;
use mc_transaction_core::tx::{Tx, TxOut, TxOutConfirmationNumber};

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
}

impl<
        T: BlockchainConnection + UserTxConnection + 'static,
        FPR: FogPubkeyResolver + Send + Sync + 'static,
    > Wallet for WalletService<T, FPR>
{
    // Wallet Status is an overview of the wallet's status
    fn get_wallet_status(&self) -> Result<json_rpc::WalletStatus, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let local_height = self.ledger_db.num_blocks()?;

        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state
            .highest_block_index_on_network()
            .map(|v| v + 1)
            .unwrap_or(0);

        Ok(
            conn.transaction::<json_rpc::WalletStatus, WalletServiceError, _>(|| {
                let accounts = db::Account::list_all(&conn)?;
                let mut account_map = Map::new();

                let mut total_available_pmob = 0;
                let mut total_pending_pmob = 0;
                let mut is_synced_all = true;
                let mut account_ids = Vec::new();
                for account in accounts {
                    let decorated = db::Account::get_decorated(
                        &db::AccountID(account.account_id_hex.clone()),
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
                    is_synced_all = is_synced_all && decorated.is_synced;
                    account_ids.push(account.account_id_hex.to_string());
                }

                Ok(json_rpc::WalletStatus {
                    object: "wallet_status".to_string(),
                    network_height: network_height.to_string(),
                    local_height: local_height.to_string(),
                    is_synced_all,
                    total_available_pmob: total_available_pmob.to_string(),
                    total_pending_pmob: total_pending_pmob.to_string(),
                    account_ids,
                    account_map,
                })
            })?,
        )
    }

    /// Creates a new account with defaults
    fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<json_rpc::CreateAccountResponse, WalletServiceError> {
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
        let (account_id, _public_address_b58) = db::Account::create(
            &account_key,
            first_block,
            None,
            &name.unwrap_or_else(|| "".to_string()),
            &conn,
        )?;

        let local_height = self.ledger_db.num_blocks()?;
        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state
            .highest_block_index_on_network()
            .map(|v| v + 1)
            .unwrap_or(0);
        let decorated_account: json_rpc::Account =
            db::Account::get_decorated(&account_id, local_height, network_height, &conn)?;

        Ok(json_rpc::CreateAccountResponse {
            entropy: entropy_str,
            account: decorated_account,
        })
    }

    fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<json_rpc::Account, WalletServiceError> {
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
        let local_height = self.ledger_db.num_blocks()?;
        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state
            .highest_block_index_on_network()
            .map(|v| v + 1)
            .unwrap_or(0);
        let conn = self.wallet_db.get_conn()?;
        Ok(Account::import(
            &account_key,
            name,
            first_block,
            local_height,
            network_height,
            &conn,
        )?)
    }

    fn get_account(
        &self,
        account_id_hex: &db::AccountID,
    ) -> Result<json_rpc::Account, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let local_height = self.ledger_db.num_blocks()?;
        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state
            .highest_block_index_on_network()
            .map(|v| v + 1)
            .unwrap_or(0);
        Ok(db::Account::get_decorated(
            &account_id_hex,
            local_height,
            network_height,
            &conn,
        )?)
    }

    fn list_accounts(&self) -> Result<Vec<json_rpc::Account>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(
            conn.transaction::<Vec<json_rpc::Account>, WalletServiceError, _>(|| {
                let accounts = Account::list_all(&conn)?;
                let local_height = self.ledger_db.num_blocks()?;
                let network_state = self.network_state.read().expect("lock poisoned");
                // network_height = network_block_index + 1
                let network_height = network_state
                    .highest_block_index_on_network()
                    .map(|v| v + 1)
                    .unwrap_or(0);
                accounts
                    .iter()
                    .map(|a| {
                        db::Account::get_decorated(
                            &db::AccountID(a.account_id_hex.clone()),
                            local_height,
                            network_height,
                            &conn,
                        )
                        .map_err(|e| e.into())
                    })
                    .collect::<Result<Vec<_>, WalletServiceError>>()
            })?,
        )
    }

    fn update_account_name(
        &self,
        account_id_hex: &str,
        name: String,
    ) -> Result<json_rpc::Account, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<json_rpc::Account, WalletServiceError, _>(|| {
                Account::get(&db::AccountID(account_id_hex.to_string()), &conn)?
                    .update_name(name, &conn)?;

                let local_height = self.ledger_db.num_blocks()?;
                let network_state = self.network_state.read().expect("lock poisoned");
                // network_height = network_block_index + 1
                let network_height = network_state
                    .highest_block_index_on_network()
                    .map(|v| v + 1)
                    .unwrap_or(0);
                let decorated_account = Account::get_decorated(
                    &db::AccountID(account_id_hex.to_string()),
                    local_height,
                    network_height,
                    &conn,
                )?;
                Ok(decorated_account)
            })?,
        )
    }

    fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        db::Account::get(&db::AccountID(account_id_hex.to_string()), &conn)
            .and_then(|account| account.delete(&conn))?;
        Ok(())
    }

    fn list_txos(&self, account_id_hex: &str) -> Result<Vec<json_rpc::Txo>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let txos = Txo::list_for_account(account_id_hex, &conn)?;
        Ok(txos.iter().map(|t| json_rpc::Txo::new(t)).collect())
    }

    fn get_txo(&self, txo_id_hex: &str) -> Result<json_rpc::Txo, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let txo_details = Txo::get(txo_id_hex, &conn)?;
        Ok(json_rpc::Txo::new(&txo_details))
    }

    // Balance consists of the sums of the various txo states in our wallet
    fn get_balance(&self, account_id_hex: &str) -> Result<BalanceResponse, WalletServiceError> {
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
        let account = Account::get(&db::AccountID(account_id_hex.to_string()), &conn)?;

        Ok(BalanceResponse {
            unspent: unspent.to_string(),
            pending: pending.to_string(),
            spent: spent.to_string(),
            secreted: secreted.to_string(),
            orphaned: orphaned.to_string(),
            local_block_count: local_block_count.to_string(),
            synced_blocks: account.next_block.to_string(),
        })
    }

    fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: Option<String>,
        // FIXME: WS-32 - add "sync from block"
    ) -> Result<json_rpc::Address, WalletServiceError> {
        let conn = &self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<json_rpc::Address, WalletServiceError, _>(|| {
                let (public_address_b58, _subaddress_index) =
                    AssignedSubaddress::create_next_for_account(
                        account_id_hex,
                        comment.unwrap_or_else(|| String::from("")).as_str(),
                        &conn,
                    )?;

                Ok(json_rpc::Address::new(&AssignedSubaddress::get(
                    &public_address_b58,
                    &conn,
                )?))
            })?,
        )
    }

    fn list_assigned_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<json_rpc::Address>, WalletServiceError> {
        Ok(
            AssignedSubaddress::list_all(account_id_hex, &self.wallet_db.get_conn()?)?
                .iter()
                .map(|a| json_rpc::Address::new(a))
                .collect::<Vec<json_rpc::Address>>(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTxProposal, WalletServiceError> {
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
            builder.set_txos(&inputs)?;
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
        Ok(mc_mobilecoind_json::data_types::JsonTxProposal::from(
            &proto_tx_proposal,
        ))
    }

    fn submit_transaction(
        &self,
        tx_proposal: mc_mobilecoind_json::data_types::JsonTxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<json_rpc::SubmitResponse, WalletServiceError> {
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
        let converted_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&tx_proposal_proto)?; // Look here
        let transaction_id = db::TransactionLog::log_submitted(
            converted_proposal,
            block_count,
            comment.unwrap_or_else(|| "".to_string()),
            account_id_hex.as_deref(),
            &self.wallet_db.get_conn()?,
        )?;

        // Successfully submitted.
        Ok(json_rpc::SubmitResponse { transaction_id })
    }

    /// Convenience method that builds and submits in one go.
    #[allow(clippy::too_many_arguments)]
    fn send_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    ) -> Result<json_rpc::SubmitResponse, WalletServiceError> {
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

    fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<json_rpc::TransactionLog>, WalletServiceError> {
        let transactions = TransactionLog::list_all(account_id_hex, &self.wallet_db.get_conn()?)?;

        let mut results: Vec<json_rpc::TransactionLog> = Vec::new();
        for (transaction, associated_txos) in transactions.iter() {
            results.push(json_rpc::TransactionLog::new(
                &transaction,
                &associated_txos,
            ));
        }
        Ok(results)
    }

    fn get_transaction(
        &self,
        transaction_id_hex: &str,
    ) -> Result<json_rpc::TransactionLog, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<json_rpc::TransactionLog, WalletServiceError, _>(|| {
                let transaction = TransactionLog::get(transaction_id_hex, &conn)?;
                let associated = transaction.get_associated_txos(&conn)?;

                Ok(json_rpc::TransactionLog::new(&transaction, &associated))
            })?,
        )
    }

    fn get_transaction_object(
        &self,
        transaction_id_hex: &str,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTx, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction = TransactionLog::get(transaction_id_hex, &conn)?;

        if let Some(tx_bytes) = transaction.tx {
            let tx: Tx = mc_util_serial::decode(&tx_bytes)?;
            // Convert to proto
            let proto_tx = mc_api::external::Tx::from(&tx); // TODO: ???
            Ok(mc_mobilecoind_json::data_types::JsonTx::from(&proto_tx))
        } else {
            Err(WalletServiceError::NoTxInTransaction)
        }
    }

    /*

    pub fn get_txo_object(&self, txo_id_hex: &str) -> Result<JsonTxOut, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let txo_details = Txo::get(txo_id_hex, &conn)?;

        let txo: TxOut = mc_util_serial::decode(&txo_details.txo.txo)?;
        // Convert to proto
        let proto_txo = mc_api::external::TxOut::from(&txo);
        Ok(JsonTxOut::from(&proto_txo))
    }
     */

    fn get_txo_object(
        &self,
        txo_id_hex: &str,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTxOut, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let txo_details = Txo::get(txo_id_hex, &conn)?;

        let tx_out: TxOut = mc_util_serial::decode(&txo_details.txo.txo)?;
        // Convert to proto
        let proto_txo = mc_api::external::TxOut::from(&tx_out);
        Ok(mc_mobilecoind_json::data_types::JsonTxOut::from(&proto_txo))
    }

    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(json_rpc::Block, json_rpc::BlockContents), WalletServiceError> {
        let block = self.ledger_db.get_block(block_index)?;
        let block_contents = self.ledger_db.get_block_contents(block_index)?;
        Ok((
            json_rpc::Block::new(&block),
            json_rpc::BlockContents::new(&block_contents),
        ))
    }

    fn get_proofs(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<MembershipProof>, WalletServiceError> {
        let transaction_log = self.get_transaction(&transaction_log_id)?;
        let proofs: Vec<MembershipProof> = transaction_log
            .output_txo_ids
            .iter()
            .map(|txo_id| {
                self.get_txo(txo_id).and_then(|txo| {
                    txo.proof
                        .map(|proof| MembershipProof {
                            object: "proof".to_string(),
                            txo_id: txo_id.clone(),
                            proof,
                        })
                        .ok_or_else(|| WalletServiceError::MissingProof(txo_id.to_string()))
                })
            })
            .collect::<Result<Vec<MembershipProof>, WalletServiceError>>()?;
        Ok(proofs)
    }

    fn verify_proof(
        &self,
        account_id_hex: &str,
        txo_id_hex: &str,
        proof_hex: &str,
    ) -> Result<bool, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let proof: TxOutConfirmationNumber = mc_util_serial::decode(&hex::decode(proof_hex)?)?;
        Ok(Txo::verify_proof(
            &db::AccountID(account_id_hex.to_string()),
            &txo_id_hex,
            &proof,
            &conn,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{TXO_MINTED, TXO_RECEIVED};
    use crate::test_utils::{
        add_block_to_ledger_db, get_test_ledger, setup_peer_manager_and_network_state,
        WalletDbTestContext,
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
        assert_eq!(
            txos[0].account_status_map[&alice.account.account_id]
                .get("txo_status")
                .unwrap(),
            TXO_UNSPENT
        );

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
        let pending: Vec<json_rpc::Txo> = txos
            .iter()
            .cloned()
            .filter(|t| {
                t.account_status_map[&alice.account.account_id]["txo_status"] == TXO_PENDING
            })
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].account_status_map[&alice.account.account_id]
                .get("txo_type")
                .unwrap(),
            TXO_RECEIVED
        );
        assert_eq!(pending[0].value_pmob, "100000000000000");
        let minted: Vec<json_rpc::Txo> = txos
            .iter()
            .cloned()
            .filter(|t| t.minted_account_id.is_some())
            .collect();
        assert_eq!(minted.len(), 2);
        assert_eq!(
            minted[0].account_status_map[&alice.account.account_id]
                .get("txo_type")
                .unwrap(),
            TXO_MINTED
        );
        assert_eq!(
            minted[1].account_status_map[&alice.account.account_id]
                .get("txo_type")
                .unwrap(),
            TXO_MINTED
        );
        let minted_value_set = HashSet::from_iter(minted.iter().map(|m| m.value_pmob.clone()));
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

    // TODO: test create_account
    // TODO: test import_account
    // TODO: test get_all_accounts
    // TODO: test get_account
    // TODO: test update_account_name
    // TODO: test delete_account
    // TODO: test get_all_txos_by_account
    // TODO: test get_txo
    // TODO: test get_wallet_status
    // TODO: test get_balance
    // TODO: test create_address
    // TODO: test get_all_addresses_by_account
    // TODO: test send_transaction
    // TODO: test build_transaction
    // TODO: test submit_transaction
    // TODO: test get_all_transactions_by_account
    // TODO: test get_transaction
    // TODO: test get_transaction_object
    // TODO: test get_txo_object
    // TODO: test get_block_object
    // TODO: test get_proofs
    // TODO: test verify_proof
}
