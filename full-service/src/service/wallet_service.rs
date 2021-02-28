// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet Service for interacting with the wallet.

use crate::{
    db::{
        account::AccountID,
        assigned_subaddress::AssignedSubaddressModel,
        models::{AssignedSubaddress, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb,
    },
    error::WalletServiceError,
    json_rpc::api_v1::decorated_types::{
        JsonAddress, JsonBlock, JsonBlockContents, JsonProof, JsonTxo,
    },
    service::sync::SyncThread,
};
use mc_common::logger::{log, Logger};
use mc_connection::{
    BlockchainConnection, ConnectionManager as McConnectionManager, UserTxConnection,
};
use mc_crypto_rand::rand_core::RngCore;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::{NetworkState, PollingNetworkState};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use mc_transaction_core::tx::{Tx, TxOut, TxOutConfirmationNumber};
use mc_util_uri::FogUri;

use crate::service::transaction_log::TransactionLogService;
use diesel::prelude::*;
use std::sync::{atomic::AtomicUsize, Arc, RwLock};

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
    pub peer_manager: McConnectionManager<T>,

    /// Representation of the current network state.
    pub network_state: Arc<RwLock<PollingNetworkState<T>>>,

    /// Fog resolver factory to obtain the public key of the ingest enclave from
    /// a fog address.
    pub fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Background ledger sync thread.
    _sync_thread: SyncThread,

    /// Monotonically increasing counter. This is used for node round-robin
    /// selection.
    pub submit_node_offset: Arc<AtomicUsize>,

    /// Whether the service should run in offline mode.
    pub offline: bool,

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

    pub fn get_network_block_index(&self) -> Result<u64, WalletServiceError> {
        let network_state = self.network_state.read().expect("lock poisoned");
        Ok(network_state.highest_block_index_on_network().unwrap_or(0))
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
        let (_transaction_log, associated_txos) = self.get_transaction(&transaction_log_id)?;
        let proofs: Vec<JsonProof> = associated_txos
            .outputs
            .iter()
            .map(|txo_id| {
                self.get_txo(txo_id).and_then(|txo| {
                    txo.proof
                        .map(|proof| JsonProof {
                            object: "proof".to_string(),
                            txo_id: txo_id.to_string(),
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
            b58_encode,
            models::{TXO_STATUS_PENDING, TXO_STATUS_UNSPENT, TXO_TYPE_MINTED, TXO_TYPE_RECEIVED},
        },
        service::{
            account::AccountService, balance::BalanceService, transaction::TransactionService,
        },
        test_utils::{add_block_to_ledger_db, get_test_ledger, setup_wallet_service, MOB},
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::{
        logger::{test_with_logger, Logger},
        HashSet,
    };
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{iter::FromIterator, time::Duration};

    #[test_with_logger]
    fn test_txo_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger);
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
            .filter(|t| {
                t.account_status_map[&alice.account_id_hex]["txo_status"] == TXO_STATUS_PENDING
            })
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
        assert_eq!(balance.pending, 0);
        assert_eq!(balance.spent, 99990000000000);
        assert_eq!(balance.secreted, 0);
        assert_eq!(balance.orphaned, 100000000000000); // FIXME: Should not be
                                                       // orphaned?

        // FIXME: How to make the transaction actually hit the test ledger?
    }
}
