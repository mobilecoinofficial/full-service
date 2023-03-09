// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet Service for interacting with the wallet.

use crate::{
    config::{NetworkSetupConfig, PeersConfig},
    db::{Conn, WalletDb, WalletDbError},
    service::sync::SyncThread,
};
use mc_common::logger::{log, Logger};
use mc_connection::{
    BlockchainConnection, ConnectionManager as McConnectionManager, UserTxConnection,
};
use mc_crypto_rand::rand_core::RngCore;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::LedgerDB;
use mc_ledger_sync::PollingNetworkState;
use mc_util_uri::{ConnectionUri, FogUri};
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
    pub network_setup_config: NetworkSetupConfig,

    /// Representation of the current network state.
    pub network_state: Arc<RwLock<PollingNetworkState<T>>>,

    /// Fog resolver factory to obtain the public key of the ingest enclave from
    /// a fog address.
    pub fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Background ledger sync thread.
    _sync_thread: Option<SyncThread>,

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
        wallet_db: Option<WalletDb>,
        ledger_db: LedgerDB,
        watcher_db: Option<WatcherDB>,
        peer_manager: McConnectionManager<T>,
        peers_config: Option<PeersConfig>,
        network_state: Arc<RwLock<PollingNetworkState<T>>>,
        fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,
        offline: bool,
        logger: Logger,
    ) -> Self {
        let sync_thread = if let Some(wallet_db) = wallet_db.clone() {
            log::info!(logger, "Starting Wallet TXO Sync Task Thread");
            Some(SyncThread::start(
                ledger_db.clone(),
                wallet_db,
                logger.clone(),
            ))
        } else {
            None
        };

        let mut chain_id = "".to_string();
        let mut tx_sources: Option<Vec<String>> = None;
        let mut peers: Option<Vec<String>> = None;

        match peers_config {
            None => (),
            Some(peers_config) => {
                chain_id = peers_config.chain_id;
                tx_sources = peers_config.tx_source_urls;
                peers = match peers_config.peers {
                    None => None,
                    Some(peers) => Some(
                        peers
                            .iter()
                            .map(|peer_uri| peer_uri.url().clone().into())
                            .collect(),
                    ),
                }
            }
        }

        let network_setup_config = NetworkSetupConfig {
            offline,
            chain_id,
            peers,
            tx_sources,
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
            submit_node_offset: Arc::new(AtomicUsize::new(rng.next_u64() as usize)),
            offline,
            logger,
        }
    }

    pub fn get_conn(&self) -> Result<Conn, WalletDbError> {
        self.wallet_db
            .as_ref()
            .ok_or(WalletDbError::WalletFunctionsDisabled)?
            .get_conn()
    }
}
