// Copyright (c) 2020 MobileCoin Inc.

//! Config definition and processing for Wallet Service.

use mc_attest_core::Verifier;
use mc_common::{
    logger::{log, Logger},
    ResponderId,
};
use mc_connection::{ConnectionManager, ThickClient};
use mc_consensus_scp::QuorumSet;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::ReqwestTransactionsFetcher;
use mc_util_uri::{ConnectionUri, ConsensusClientUri};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use structopt::StructOpt;

/// Command line config for the Wallet API
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "wallet-service",
    about = "An HTTP wallet service for MobileCoin"
)]
pub struct APIConfig {
    /// Host to listen on.
    #[structopt(long, default_value = "127.0.0.1")]
    pub listen_host: String,

    /// Port to start webserver on.
    #[structopt(long, default_value = "9090")]
    pub listen_port: u16,

    /// Path to WalletDb.
    #[structopt(long, parse(from_os_str))]
    pub wallet_db: PathBuf,

    /// Path to LedgerDB
    #[structopt(long, parse(from_os_str))]
    pub ledger_db: PathBuf,

    /// Path to existing ledger db that contains the origin block, used when initializing new ledger dbs.
    #[structopt(long)]
    pub ledger_db_bootstrap: Option<String>,

    #[structopt(flatten)]
    pub peers_config: PeersConfig,

    /// Quorum set for ledger syncing. By default, the quorum set would include all peers.
    ///
    /// The quorum set is represented in JSON. For example:
    /// {"threshold":1,"members":[{"type":"Node","args":"node2.test.mobilecoin.com:443"},{"type":"Node","args":"node3.test.mobilecoin.com:443"}]}
    #[structopt(long, parse(try_from_str=parse_quorum_set_from_json))]
    quorum_set: Option<QuorumSet<ResponderId>>,

    /// URLs to use for transaction data.
    ///
    /// For example: https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/
    #[structopt(long = "tx-source-url", required_unless = "offline")]
    pub tx_source_urls: Option<Vec<String>>,

    /// Number of worker threads to use for view key scanning.
    /// Defaults to number of logical CPU cores.
    #[structopt(long)]
    pub num_workers: Option<usize>,

    /// How many seconds to wait between polling.
    #[structopt(long, default_value = "5", parse(try_from_str=parse_duration_in_seconds))]
    pub poll_interval: Duration,

    /// Offline mode.
    #[structopt(long)]
    pub offline: bool,
}

fn parse_duration_in_seconds(src: &str) -> Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_secs(u64::from_str(src)?))
}

fn parse_quorum_set_from_json(src: &str) -> Result<QuorumSet<ResponderId>, String> {
    let quorum_set: QuorumSet<ResponderId> = serde_json::from_str(src)
        .map_err(|err| format!("Error parsing quorum set {}: {:?}", src, err))?;

    if !quorum_set.is_valid() {
        return Err(format!("Invalid quorum set: {:?}", quorum_set));
    }

    Ok(quorum_set)
}

impl APIConfig {
    pub fn quorum_set(&self) -> QuorumSet<ResponderId> {
        // If we have an explicit quorum set, use that.
        if let Some(quorum_set) = &self.quorum_set {
            return quorum_set.clone();
        }

        // Otherwise create a quorum set that includes all of the peers we know about.
        let node_ids = self
            .peers_config
            .peers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|p| {
                p.responder_id().unwrap_or_else(|e| {
                    panic!(
                        "Could not get responder_id from uri {}: {:?}",
                        p.to_string(),
                        e
                    )
                })
            })
            .collect::<Vec<ResponderId>>();
        QuorumSet::new_with_node_ids(node_ids.len() as u32, node_ids)
    }

    pub fn create_or_open_ledger_db(
        &self,
        logger: &Logger,
        transactions_fetcher: &ReqwestTransactionsFetcher,
    ) -> LedgerDB {
        // Attempt to open the ledger and see if it has anything in it.
        if let Ok(ledger_db) = LedgerDB::open(self.ledger_db.clone()) {
            if let Ok(num_blocks) = ledger_db.num_blocks() {
                if num_blocks > 0 {
                    // Successfully opened a ledger that has blocks in it.
                    log::info!(
                        logger,
                        "Ledger DB {:?} opened: num_blocks={} num_txos={}",
                        self.ledger_db,
                        num_blocks,
                        ledger_db.num_txos().expect("Failed getting number of txos")
                    );
                    return ledger_db;
                }
            }
        }

        // Ledger doesn't exist, or is empty. Copy a bootstrapped ledger or try and get it from the network.
        let ledger_db_file = Path::new(&self.ledger_db).join("data.mdb");
        match &self.ledger_db_bootstrap {
            Some(ledger_db_bootstrap) => {
                log::debug!(
                    logger,
                    "Ledger DB {:?} does not exist, copying from {}",
                    self.ledger_db,
                    ledger_db_bootstrap
                );

                // Try and create directory in case it doesn't exist. We need it to exist before we
                // can copy the data.mdb file.
                if !Path::new(&self.ledger_db).exists() {
                    std::fs::create_dir_all(self.ledger_db.clone()).unwrap_or_else(|_| {
                        panic!("Failed creating directory {:?}", self.ledger_db)
                    });
                }

                let src = format!("{}/data.mdb", ledger_db_bootstrap);
                std::fs::copy(src.clone(), ledger_db_file.clone()).unwrap_or_else(|_| {
                    panic!(
                        "Failed copying ledger from {} into directory {}",
                        src,
                        ledger_db_file.display()
                    )
                });
            }
            None => {
                log::info!(
                    logger,
                    "Ledger DB {:?} does not exist, bootstrapping from peer, this may take a few minutes",
                    self.ledger_db
                );
                std::fs::create_dir_all(self.ledger_db.clone())
                    .expect("Could not create ledger dir");
                LedgerDB::create(self.ledger_db.clone()).expect("Could not create ledger_db");
                let block_data = transactions_fetcher
                    .get_origin_block_and_transactions()
                    .expect("Failed to download initial transactions");
                let mut db =
                    LedgerDB::open(self.ledger_db.clone()).expect("Could not open ledger_db");
                db.append_block(
                    block_data.block(),
                    block_data.contents(),
                    block_data.signature().clone(),
                )
                .expect("Failed to appened initial transactions");
                log::info!(logger, "Bootstrapping completed!");
            }
        }

        // Open ledger and verify it has (at least) the origin block.
        log::debug!(logger, "Opening Ledger DB {:?}", self.ledger_db);
        let ledger_db = LedgerDB::open(self.ledger_db.clone())
            .unwrap_or_else(|_| panic!("Could not open ledger db inside {:?}", self.ledger_db));

        let num_blocks = ledger_db
            .num_blocks()
            .expect("Failed getting number of blocks");
        if num_blocks == 0 {
            panic!("Ledger DB is empty :(");
        }

        log::info!(
            logger,
            "Ledger DB {:?} opened: num_blocks={} num_txos={}",
            self.ledger_db,
            num_blocks,
            ledger_db.num_txos().expect("Failed getting number of txos")
        );

        ledger_db
    }
}

#[derive(Clone, Debug, StructOpt)]
#[structopt()]
pub struct PeersConfig {
    /// validator nodes to connect to.
    #[structopt(long = "peer", required_unless = "offline")]
    pub peers: Option<Vec<ConsensusClientUri>>,
}

impl PeersConfig {
    pub fn responder_ids(&self) -> Vec<ResponderId> {
        self.peers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|peer| {
                peer.responder_id()
                    .expect("Could not get responder_id from peer")
            })
            .collect()
    }

    pub fn create_peers(
        &self,
        verifier: Verifier,
        grpc_env: Arc<grpcio::Environment>,
        logger: Logger,
    ) -> Vec<ThickClient> {
        self.peers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|client_uri| {
                ThickClient::new(
                    client_uri.clone(),
                    verifier.clone(),
                    grpc_env.clone(),
                    logger.clone(),
                )
                .expect("Could not create thick client.")
            })
            .collect()
    }

    pub fn create_peer_manager(
        &self,
        verifier: Verifier,
        logger: &Logger,
    ) -> ConnectionManager<ThickClient> {
        let grpc_env = Arc::new(
            grpcio::EnvBuilder::new()
                .cq_count(1)
                .name_prefix("peer")
                .build(),
        );
        let peers = self.create_peers(verifier, grpc_env, logger.clone());

        ConnectionManager::new(peers, logger.clone())
    }
}
