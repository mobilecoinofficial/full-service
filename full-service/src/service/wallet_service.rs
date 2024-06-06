// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet Service for interacting with the wallet.

use crate::{
    config::NetworkConfig,
    db::{WalletDb, WalletDbError},
    service::{
        sync::SyncThread,
        t3_sync::{T3Config, T3SyncThread},
    },
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    SqliteConnection,
};
use mc_common::logger::{log, Logger};
use mc_connection::{
    BlockchainConnection, ConnectionManager as McConnectionManager, UserTxConnection,
};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::LedgerDB;
use mc_ledger_sync::PollingNetworkState;
use mc_rand::rand_core::RngCore;
use mc_util_uri::FogUri;
use mc_watcher::watcher_db::WatcherDB;
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
    pub wallet_db: Option<WalletDb>,

    /// Ledger database.
    pub ledger_db: LedgerDB,

    /// Watcher database.
    pub watcher_db: Option<WatcherDB>,

    /// Peer manager for consensus validators to query for network height.
    pub peer_manager: McConnectionManager<T>,

    /// Peer network information
    pub network_setup_config: NetworkConfig,

    /// Representation of the current network state.
    pub network_state: Arc<RwLock<PollingNetworkState<T>>>,

    /// Fog resolver factory to obtain the public key of the ingest enclave from
    /// a fog address.
    #[allow(clippy::type_complexity)]
    pub fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Background ledger sync thread.
    _sync_thread: Option<SyncThread>,

    /// Background T3 sync thread.
    _t3_sync_thread: Option<T3SyncThread>,

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
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn new(
        wallet_db: Option<WalletDb>,
        ledger_db: LedgerDB,
        watcher_db: Option<WatcherDB>,
        peer_manager: McConnectionManager<T>,
        network_setup_config: NetworkConfig,
        network_state: Arc<RwLock<PollingNetworkState<T>>>,
        fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,
        offline: bool,
        t3_sync_config: T3Config,
        logger: Logger,
    ) -> Self {
        let sync_thread = if let Some(wallet_db) = wallet_db.clone() {
            log::info!(logger, "Starting Wallet TXO Sync Task Thread");
            Some(SyncThread::start(
                ledger_db.clone(),
                wallet_db,
                "/wallet", // FIXME
                logger.clone(),
            ))
        } else {
            None
        };

        let t3_sync_thread = if let (Some(wallet_db), Some(t3_uri), Some(t3_api_key)) = (
            wallet_db.clone(),
            t3_sync_config.t3_uri,
            t3_sync_config.t3_api_key,
        ) {
            log::info!(logger, "Starting T3 Sync Task Thread");
            Some(T3SyncThread::start(
                t3_uri,
                t3_api_key,
                wallet_db,
                logger.clone(),
            ))
        } else {
            None
        };

        let mut rng = rand::thread_rng();
        WalletService {
            wallet_db,
            ledger_db,
            watcher_db,
            peer_manager,
            network_setup_config,
            network_state,
            fog_resolver_factory,
            _sync_thread: sync_thread,
            _t3_sync_thread: t3_sync_thread,
            submit_node_offset: Arc::new(AtomicUsize::new(rng.next_u64() as usize)),
            offline,
            logger,
        }
    }

    pub fn get_pooled_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        self.wallet_db
            .as_ref()
            .ok_or(WalletDbError::WalletFunctionsDisabled)?
            .get_pooled_conn()
    }
}
