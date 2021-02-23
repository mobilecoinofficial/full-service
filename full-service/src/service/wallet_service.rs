// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet Service for interacting with the wallet.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        b58_decode,
        models::{
            Account, AssignedSubaddress, TransactionLog, Txo, TXO_STATUS_ORPHANED,
            TXO_STATUS_PENDING, TXO_STATUS_SECRETED, TXO_STATUS_SPENT, TXO_STATUS_UNSPENT,
        },
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb,
    },
    error::WalletServiceError,
    json_rpc::api_v1::decorated_types::{
        JsonAddress, JsonBalanceResponse, JsonBlock, JsonBlockContents, JsonProof,
        JsonSubmitResponse, JsonTransactionLog, JsonTxo, JsonWalletStatus,
    },
    service::{sync::SyncThread, transaction_builder::WalletTransactionBuilder},
};
use mc_common::logger::{log, Logger};
use mc_connection::{
    BlockchainConnection, ConnectionManager as McConnectionManager, RetryableUserTxConnection,
    UserTxConnection,
};
use mc_crypto_rand::rand_core::RngCore;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::{NetworkState, PollingNetworkState};
use mc_mobilecoind::payments::TxProposal;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxProposal};
use mc_transaction_core::tx::{Tx, TxOut, TxOutConfirmationNumber};
use mc_util_uri::FogUri;

use diesel::prelude::*;
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
///
/// Note that some fields need to be pub in order to be used in trait
/// implementations, as Rust sees those as separate modules when they are in
/// separate files.
pub struct WalletService<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    /// Wallet database handle.
    pub wallet_db: WalletDb,

    /// Ledger database.
    pub ledger_db: LedgerDB,

    /// Peer manager for consensus validators to query for network height.
    peer_manager: McConnectionManager<T>,

    /// Representation of the current network state.
    pub network_state: Arc<RwLock<PollingNetworkState<T>>>,

    /// Fog resolver factory to obtain the public key of the ingest enclave from
    /// a fog address.
    fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Background ledger sync thread.
    _sync_thread: SyncThread,

    /// Monotonically increasing counter. This is used for node round-robin
    /// selection.
    submit_node_offset: Arc<AtomicUsize>,

    /// Whether the service should run in offline mode.
    offline: bool,

    /// Logger.
    pub logger: Logger,
}

impl<
        T: BlockchainConnection + UserTxConnection + 'static,
        FPR: FogPubkeyResolver + Send + Sync + 'static,
    > WalletService<T, FPR>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_db: WalletDb,
        ledger_db: LedgerDB,
        peer_manager: McConnectionManager<T>,
        network_state: Arc<RwLock<PollingNetworkState<T>>>,
        fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,
        num_workers: Option<usize>,
        offline: bool,
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
            fog_resolver_factory,
            _sync_thread: sync_thread,
            submit_node_offset: Arc::new(AtomicUsize::new(rng.next_u64() as usize)),
            offline,
            logger,
        }
    }

    pub fn list_txos(&self, account_id_hex: &str) -> Result<Vec<JsonTxo>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let txos = Txo::list_for_account(account_id_hex, &conn)?;
        Ok(txos.iter().map(|t| JsonTxo::new(t)).collect())
    }

    pub fn get_txo(&self, txo_id_hex: &str) -> Result<JsonTxo, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let txo_details = Txo::get(txo_id_hex, &conn)?;
        Ok(JsonTxo::new(&txo_details))
    }

    pub fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: Option<&str>,
        // FIXME: WS-32 - add "sync from block"
    ) -> Result<JsonAddress, WalletServiceError> {
        let conn = &self.wallet_db.get_conn()?;

        Ok(conn.transaction::<JsonAddress, WalletServiceError, _>(|| {
            let (public_address_b58, _subaddress_index) =
                AssignedSubaddress::create_next_for_account(
                    account_id_hex,
                    comment.unwrap_or(""),
                    &conn,
                )?;

            Ok(JsonAddress::new(&AssignedSubaddress::get(
                &public_address_b58,
                &conn,
            )?))
        })?)
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
            self.fog_resolver_factory.clone(),
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

        // FIXME: WS-32 - Might be nice to have a tx_proposal table so that you don't
        // have to        write these out to local files. That's V2, though.
        Ok(JsonTxProposal::from(&proto_tx_proposal))
    }

    pub fn submit_transaction(
        &self,
        tx_proposal: JsonTxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<JsonSubmitResponse, WalletServiceError> {
        if self.offline {
            return Err(WalletServiceError::Offline);
        }

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

        let transaction_id = account_id_hex
            .map(|a| {
                TransactionLog::log_submitted(
                    converted_proposal,
                    block_count,
                    comment.unwrap_or_else(|| "".to_string()),
                    Some(&a),
                    &self.wallet_db.get_conn()?,
                )
            })
            .map_or(Ok(None), |v| v.map(Some))?;

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

        Ok(
            conn.transaction::<JsonTransactionLog, WalletServiceError, _>(|| {
                let transaction = TransactionLog::get(transaction_id_hex, &conn)?;
                let associated = transaction.get_associated_txos(&conn)?;

                Ok(JsonTransactionLog::new(&transaction, &associated))
            })?,
        )
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

    pub fn get_txo_object(&self, txo_id_hex: &str) -> Result<JsonTxOut, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let txo_details = Txo::get(txo_id_hex, &conn)?;

        let txo: TxOut = mc_util_serial::decode(&txo_details.txo.txo)?;
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

    pub fn get_proofs(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<JsonProof>, WalletServiceError> {
        let transaction_log = self.get_transaction(&transaction_log_id)?;
        let proofs: Vec<JsonProof> = transaction_log
            .output_txo_ids
            .iter()
            .map(|txo_id| {
                self.get_txo(txo_id).and_then(|txo| {
                    txo.proof
                        .map(|proof| JsonProof {
                            object: "proof".to_string(),
                            txo_id: txo_id.clone(),
                            proof,
                        })
                        .ok_or_else(|| WalletServiceError::MissingProof(txo_id.to_string()))
                })
            })
            .collect::<Result<Vec<JsonProof>, WalletServiceError>>()?;
        Ok(proofs)
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
        db::{
            models::{TXO_TYPE_MINTED, TXO_TYPE_RECEIVED},
            txo::TxoDetails,
        },
        service::{account::AccountService, balance::BalanceService},
        test_utils::{
            add_block_to_ledger_db, get_resolver_factory, get_test_ledger,
            setup_peer_manager_and_network_state, WalletDbTestContext, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::{
        logger::{test_with_logger, Logger},
        HashSet,
    };
    use mc_connection_test_utils::MockBlockchainConnection;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{iter::FromIterator, time::Duration};

    fn setup_service(
        ledger_db: LedgerDB,
        logger: Logger,
    ) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let (peer_manager, network_state) =
            setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

        WalletService::new(
            wallet_db,
            ledger_db,
            peer_manager,
            network_state,
            get_resolver_factory(&mut rng).unwrap(),
            None,
            false,
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
        // Add a block with a txo for this address (note that value is smaller than
        // MINIMUM_FEE)
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(5));

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex.clone()))
            .unwrap();

        assert_eq!(balance.unspent, 100 * MOB as u64);

        // Verify that we have 1 txo
        let txos = service.list_txos(&alice.account_id_hex).unwrap();
        assert_eq!(txos.len(), 1);
        assert_eq!(
            txos[0].account_status_map[&alice.account_id_hex]
                .get("txo_status")
                .unwrap(),
            TXO_STATUS_UNSPENT
        );

        // Add another account
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();

        // Construct a new transaction to Bob
        let bob_account_key: AccountKey = mc_util_serial::decode(&bob.account_key).unwrap();
        let tx_proposal = service
            .build_transaction(
                &alice.account_id_hex,
                &b58_encode(&bob_account_key.subaddress(bob.main_subaddress_index as u64)).unwrap(),
                "42000000000000".to_string(),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let _submitted = service
            .submit_transaction(tx_proposal, None, Some(alice.account_id_hex.clone()))
            .unwrap();

        // We should now have 3 txos - one pending, two minted (one of which will be
        // change)
        let txos = service.list_txos(&alice.account_id_hex).unwrap();
        assert_eq!(txos.len(), 3);
        // The Pending Tx
        let pending: Vec<JsonTxo> = txos
            .iter()
            .cloned()
            .filter(|t| t.account_status_map[&alice.account_id_hex]["txo_status"] == TXO_PENDING)
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].account_status_map[&alice.account_id_hex]
                .get("txo_type")
                .unwrap(),
            TXO_TYPE_RECEIVED
        );
        assert_eq!(pending[0].value_pmob, "100000000000000");
        let minted: Vec<JsonTxo> = txos
            .iter()
            .cloned()
            .filter(|t| t.minted_account_id.is_some())
            .collect();
        assert_eq!(minted.len(), 2);
        assert_eq!(
            minted[0].account_status_map[&alice.account_id_hex]
                .get("txo_type")
                .unwrap(),
            TXO_TYPE_MINTED
        );
        assert_eq!(
            minted[1].account_status_map[&alice.account_id_hex]
                .get("txo_type")
                .unwrap(),
            TXO_TYPE_MINTED
        );
        let minted_value_set = HashSet::from_iter(minted.iter().map(|m| m.value_pmob.clone()));
        assert!(minted_value_set.contains("57990000000000"));
        assert!(minted_value_set.contains("42000000000000"));

        // Our balance should reflect the various statuses of our txos
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex))
            .unwrap();
        assert_eq!(balance.unspent, 0);
        assert_eq!(balance.pending, 100000000000000);
        assert_eq!(balance.spent, 0);
        assert_eq!(balance.secreted, 99990000000000);
        assert_eq!(balance.orphaned, 0);

        // FIXME: How to make the transaction actually hit the test ledger?
    }

    // Test sending a transaction from Alice -> Bob, and then from Bob -> Alice
    #[test_with_logger]
    fn test_send_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_public_address = b58_decode(&alice.account.main_address).unwrap();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo - FIXME poll instead of sleep
        std::thread::sleep(Duration::from_secs(8));

        // Verify balance for Alice
        let balance = service.get_balance(&alice.account.account_id).unwrap();
        assert_eq!(balance.unspent.parse::<i64>().unwrap(), 100 * MOB);

        // Add an account for Bob
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();

        // Create an assigned subaddress for Bob
        let bob_address_from_alice = service
            .create_assigned_subaddress(&bob.account.account_id, Some("From Alice"))
            .unwrap();

        // Send a transaction from Alice to Bob
        let submit_response = service
            .send_transaction(
                &alice.account.account_id,
                &bob_address_from_alice.public_address,
                (42 * MOB).to_string(),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        log::info!(logger, "Built and submitted transaction from Alice");

        let json_transaction_log = service
            .get_transaction(&submit_response.transaction_id.unwrap())
            .unwrap();

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.
        let transaction_log = {
            let conn = service.wallet_db.get_conn().unwrap();

            TransactionLog::get(&json_transaction_log.transaction_log_id, &conn).unwrap()
        };

        {
            log::info!(logger, "Adding block from transaction log");
            let conn = service.wallet_db.get_conn().unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn, &transaction_log);
        }

        std::thread::sleep(Duration::from_secs(8));

        // Get the Txos from the transaction log
        let transaction_txos = transaction_log
            .get_associated_txos(&service.wallet_db.get_conn().unwrap())
            .unwrap();
        let secreted = transaction_txos
            .outputs
            .iter()
            .map(|t| Txo::get(t, &service.wallet_db.get_conn().unwrap()).unwrap())
            .collect::<Vec<TxoDetails>>();
        assert_eq!(secreted.len(), 1);
        assert_eq!(secreted[0].txo.value, 42 * MOB);

        let change = transaction_txos
            .change
            .iter()
            .map(|t| Txo::get(t, &service.wallet_db.get_conn().unwrap()).unwrap())
            .collect::<Vec<TxoDetails>>();
        assert_eq!(change.len(), 1);
        assert_eq!(change[0].txo.value, (57.99 * MOB as f64) as i64);

        let inputs = transaction_txos
            .inputs
            .iter()
            .map(|t| Txo::get(t, &service.wallet_db.get_conn().unwrap()).unwrap())
            .collect::<Vec<TxoDetails>>();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].txo.value, 100 * MOB);

        // Verify balance for Alice = original balance - fee - txo_value
        let balance = service.get_balance(&alice.account.account_id).unwrap();
        assert_eq!(balance.unspent, "57990000000000");

        // Bob's balance should be = output_txo_value
        let bob_balance = service.get_balance(&bob.account.account_id).unwrap();
        assert_eq!(bob_balance.unspent, "42000000000000");

        // Bob should now be able to send to Alice
        let submit_response = service
            .send_transaction(
                &bob.account.account_id,
                &alice.account.main_address,
                (8 * MOB).to_string(),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let json_transaction_log = service
            .get_transaction(&submit_response.transaction_id.unwrap())
            .unwrap();

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.
        let transaction_log = {
            let conn = service.wallet_db.get_conn().unwrap();

            TransactionLog::get(&json_transaction_log.transaction_log_id, &conn).unwrap()
        };

        {
            log::info!(logger, "Adding block from transaction log");
            let conn = service.wallet_db.get_conn().unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn, &transaction_log);
        }

        std::thread::sleep(Duration::from_secs(8));

        let alice_balance = service.get_balance(&alice.account.account_id).unwrap();
        assert_eq!(alice_balance.unspent, "65990000000000");

        // Bob's balance should be = output_txo_value
        let bob_balance = service.get_balance(&bob.account.account_id).unwrap();
        assert_eq!(bob_balance.unspent, "33990000000000");
    }

    // FIXME: Test with 0 change transactions
    // FIXME: Test with balance > u64::max
    // FIXME: sending a transaction with value > u64::max
}
