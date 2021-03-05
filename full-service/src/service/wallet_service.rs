// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet Service for interacting with the wallet.

use crate::{db::WalletDb, service::sync::SyncThread};
use mc_common::logger::{log, Logger};
use mc_connection::{
    BlockchainConnection, ConnectionManager as McConnectionManager, UserTxConnection,
};
use mc_crypto_rand::rand_core::RngCore;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::PollingNetworkState;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use mc_transaction_core::tx::{Tx, TxOut};
use mc_util_uri::FogUri;
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
}
