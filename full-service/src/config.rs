// Copyright (c) 2020-2021 MobileCoin Inc.

//! Config definition and processing for Wallet Service.

use mc_attest_core::MrSigner;
use mc_attestation_verifier::{TrustedIdentity, TrustedMrSignerIdentity};
use mc_blockchain_types::BlockData;
use mc_common::{
    logger::{log, Logger},
    ResponderId,
};
use mc_connection::{ConnectionManager, HardcodedCredentialsProvider, ThickClient};
use mc_consensus_scp::QuorumSet;
use mc_fog_report_connection::GrpcFogReportConnection;
use mc_fog_report_resolver::FogResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_sgx_css::Signature;
use mc_util_parse::parse_duration_in_seconds;
use mc_util_uri::{ConnectionUri, ConsensusClientUri, FogUri};
use mc_validator_api::ValidatorUri;

use clap::Parser;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::service::t3_sync::T3Config;

/// Command line config for the Wallet API
#[derive(Clone, Debug, Parser)]
#[clap(
    name = "full-service",
    about = "An HTTP wallet service for MobileCoin",
    version
)]
pub struct APIConfig {
    /// Host to listen on.
    #[clap(long, default_value = "127.0.0.1", env = "MC_LISTEN_HOST")]
    pub listen_host: String,

    /// Port to start webserver on.
    #[clap(long, default_value = "9090", env = "MC_LISTEN_PORT")]
    pub listen_port: u16,

    /// Path to WalletDb.
    #[clap(long, value_parser, env = "MC_WALLET_DB")]
    pub wallet_db: Option<PathBuf>,

    #[clap(flatten)]
    pub ledger_db_config: LedgerDbConfig,

    #[clap(flatten)]
    pub peers_config: PeersConfig,

    /// How many seconds to wait between polling.
    #[clap(long, default_value = "5", value_parser = parse_duration_in_seconds, env = "MC_POLL_INTERVAL")]
    pub poll_interval: Duration,

    /// Offline mode.
    #[clap(long, env = "MC_OFFLINE")]
    pub offline: bool,

    /// Fog ingest enclave CSS file (needed in order to enable sending
    /// transactions to fog recipients).
    #[clap(long, value_parser = load_css_file, env = "MC_FOG_INGEST_ENCLAVE_CSS")]
    pub fog_ingest_enclave_css: Option<Signature>,

    /// Validator service to connect to, when not connecting to the consensus
    /// network directly.
    #[clap(long, env = "MC_VALIDATOR")]
    pub validator: Option<ValidatorUri>,

    /// Path to watcher db (lmdb). When provided, watcher syncing will take
    /// place.
    #[clap(long, value_parser, env = "MC_WATCHER_DB")]
    pub watcher_db: Option<PathBuf>,

    /// Allowed CORS origin. When provided, the http server will add CORS
    /// headers for the provided origin. If not provided, the http server
    /// will not add any CORS headers
    #[clap(long, env = "MC_ALLOWED_ORIGIN")]
    pub allowed_origin: Option<String>,

    /// T3 Server to connect to and the api key to use for authorization.
    #[clap(flatten)]
    pub t3_sync_config: T3Config,

    /// Webhook configuration to notify an external server listening for
    /// deposit notifications.
    ///
    /// The format of the webhook is a POST request with the following
    /// parameters:
    ///
    /// POST /webhook -H "Content-Type: application/json" \
    ///     -d '{"accounts": [A,B,C]}'
    ///
    /// The expected action to take in response to the webhook is to call
    /// the `get_txos` API endpoint for the given accounts to retrieve more
    /// details about the TXOs received.
    ///
    /// It is also expected for the client to call get_txos on startup and
    /// periodically to ensure that no TXOs are missed.
    ///
    /// We expect a 200 response code to indicate that the webhook was
    /// received, and we do not further inspect the response body. Even if
    /// not a 200 response, we will continue to attempt to reach the webhook
    /// on subsequent deposits.
    #[clap(long, value_parser = Url::parse, env = "MC_DEPOSITS_WEBHOOK_URL")]
    pub deposits_webhook_url: Option<Url>,
}

fn parse_quorum_set_from_json(src: &str) -> Result<QuorumSet<ResponderId>, String> {
    let quorum_set: QuorumSet<ResponderId> = serde_json::from_str(src)
        .map_err(|err| format!("Error parsing quorum set {src}: {err:?}"))?;

    if !quorum_set.is_valid() {
        return Err(format!("Invalid quorum set: {quorum_set:?}"));
    }

    Ok(quorum_set)
}

fn load_css_file(filename: &str) -> Result<Signature, String> {
    let bytes =
        fs::read(filename).map_err(|err| format!("Failed reading file '{filename}': {err}"))?;
    let signature = Signature::try_from(&bytes[..])
        .map_err(|err| format!("Failed parsing CSS file '{filename}': {err}"))?;
    Ok(signature)
}

impl APIConfig {
    /// Get the attestation verifier used to verify fog reports when sending to
    /// fog recipients.
    pub fn get_fog_ingest_identity(&self) -> Option<TrustedIdentity> {
        self.fog_ingest_enclave_css.as_ref().map(|signature| {
            let config_advisories: Vec<&str> = vec![];
            TrustedIdentity::MrSigner(TrustedMrSignerIdentity::new(
                MrSigner::from(signature.mrsigner()),
                signature.product_id(),
                signature.version(),
                config_advisories,
                mc_consensus_enclave_measurement::HARDENING_ADVISORIES,
            ))
        })
    }

    /// Get the function which creates FogResolver given a list of recipient
    /// addresses.
    ///
    /// The string error should be mapped by invoker of this factory to
    /// Error::FogError.
    #[allow(clippy::type_complexity)]
    pub fn get_fog_resolver_factory(
        &self,
        logger: Logger,
    ) -> Arc<dyn Fn(&[FogUri]) -> Result<FogResolver, String> + Send + Sync> {
        let env = Arc::new(
            grpcio::EnvBuilder::new()
                .name_prefix("FogPubkeyResolver-RPC".to_string())
                .build(),
        );

        let conn =
            GrpcFogReportConnection::new(self.peers_config.chain_id.clone(), env, logger.clone());

        let trusted_identity = self.get_fog_ingest_identity();

        Arc::new(move |fog_uris| -> Result<FogResolver, String> {
            if fog_uris.is_empty() {
                Ok(Default::default())
            } else if let Some(trusted_identity) = trusted_identity.as_ref() {
                let report_responses = conn
                    .fetch_fog_reports(fog_uris.iter().cloned())
                    .map_err(|err| format!("Failed fetching fog reports: {err}"))?;
                log::debug!(logger, "Got report responses {:?}", report_responses);
                Ok(FogResolver::new(report_responses, vec![trusted_identity])
                    .expect("Could not construct fog resolver"))
            } else {
                Err(
                    "Some recipients have fog, but no fog ingest report verifier was configured"
                        .to_string(),
                )
            }
        })
    }
}

#[derive(Clone, Debug, Parser)]
pub struct PeersConfig {
    /// validator nodes to connect to.
    #[clap(long = "peer", required_unless_present_any = &["offline", "validator"], conflicts_with_all = &["offline", "validator"], use_value_delimiter = true, env = "MC_PEER")]
    pub peers: Option<Vec<ConsensusClientUri>>,

    /// Quorum set for ledger syncing. By default, the quorum set would include
    /// all peers.
    ///
    /// The quorum set is represented in JSON. For example:
    /// {"threshold":1,"members":[{"type":"Node","args":"node2.test.mobilecoin.
    /// com:443"},{"type":"Node","args":"node3.test.mobilecoin.com:443"}]}
    #[clap(long, value_parser = parse_quorum_set_from_json, conflicts_with_all = &["offline", "validator"], env = "MC_QUORUM_SET")]
    quorum_set: Option<QuorumSet<ResponderId>>,

    /// URLs to use for transaction data.
    ///
    /// For example: https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/
    #[clap(long = "tx-source-url", required_unless_present_any = &["offline", "validator"], conflicts_with_all = &["offline", "validator"], use_value_delimiter = true, env = "MC_TX_SOURCE_URL")]
    pub tx_source_urls: Option<Vec<String>>,

    /// Chain Id
    #[clap(default_value = "", long, env = "MC_CHAIN_ID")]
    pub chain_id: String,
}

/// The Network Setup object.
/// This holds a copy of the network parameters used to start full-service
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub offline: bool,
    pub chain_id: String,
    pub peers: Option<Vec<String>>,
    pub tx_sources: Option<Vec<String>>,
}

impl PeersConfig {
    pub fn quorum_set(&self) -> QuorumSet<ResponderId> {
        // If we have an explicit quorum set, use that.
        if let Some(quorum_set) = &self.quorum_set {
            return quorum_set.clone();
        }

        // Otherwise create a quorum set that includes all of the peers we know about.
        let node_ids = self
            .peers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|p| {
                p.responder_id().unwrap_or_else(|e| {
                    panic!("Could not get responder_id from uri {}: {:?}", p, e)
                })
            })
            .collect::<Vec<ResponderId>>();
        QuorumSet::new_with_node_ids(node_ids.len() as u32, node_ids)
    }

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
        trusted_identity: TrustedIdentity,
        grpc_env: Arc<grpcio::Environment>,
        logger: Logger,
    ) -> Vec<ThickClient<HardcodedCredentialsProvider>> {
        self.peers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|client_uri| {
                ThickClient::new(
                    self.chain_id.clone(),
                    client_uri.clone(),
                    vec![trusted_identity.clone()],
                    grpc_env.clone(),
                    HardcodedCredentialsProvider::from(client_uri),
                    logger.clone(),
                )
                .expect("Could not create thick client.")
            })
            .collect()
    }

    pub fn create_peer_manager(
        &self,
        trusted_identity: TrustedIdentity,
        logger: &Logger,
    ) -> ConnectionManager<ThickClient<HardcodedCredentialsProvider>> {
        let grpc_env = Arc::new(
            grpcio::EnvBuilder::new()
                .cq_count(1)
                .name_prefix("peer")
                .build(),
        );
        let peers = self.create_peers(trusted_identity, grpc_env, logger.clone());

        ConnectionManager::new(peers, logger.clone())
    }
}

#[derive(Clone, Debug, Parser)]
pub struct LedgerDbConfig {
    /// Path to LedgerDB
    #[clap(long, value_parser, env = "MC_LEDGER_DB")]
    pub ledger_db: PathBuf,

    /// Path to existing ledger db that contains the origin block, used when
    /// initializing new ledger dbs.
    #[clap(long, env = "MC_LEDGER_DB_BOOTSTRAP")]
    pub ledger_db_bootstrap: Option<String>,
}

impl LedgerDbConfig {
    pub fn create_or_open_ledger_db(
        &self,
        get_origin_block_and_transactions: impl Fn() -> Result<BlockData, String>,
        offline: bool,
        logger: &Logger,
    ) -> LedgerDB {
        let ledger_db_file = Path::new(&self.ledger_db).join("data.mdb");

        // Attempt to run migrations if ledger is available.
        if ledger_db_file.exists() {
            mc_ledger_migration::migrate(&self.ledger_db, logger);
        }

        // Attempt to open the ledger and see if it has anything in it.
        if let Ok(ledger_db) = LedgerDB::open(&self.ledger_db) {
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

        // Ledger doesn't exist, or is empty. Copy a bootstrapped ledger or try and get
        // it from the network.
        match &self.ledger_db_bootstrap {
            Some(ledger_db_bootstrap) => {
                log::debug!(
                    logger,
                    "Ledger DB {:?} does not exist, copying from {}",
                    self.ledger_db,
                    ledger_db_bootstrap
                );

                // Try and create directory in case it doesn't exist. We need it to exist before
                // we can copy the data.mdb file.
                if !Path::new(&self.ledger_db).exists() {
                    std::fs::create_dir_all(self.ledger_db.clone()).unwrap_or_else(|_| {
                        panic!("Failed creating directory {:?}", self.ledger_db)
                    });
                }

                let src = format!("{ledger_db_bootstrap}/data.mdb");
                std::fs::copy(src.clone(), ledger_db_file.clone()).unwrap_or_else(|_| {
                    panic!(
                        "Failed copying ledger from {} into directory {}",
                        src,
                        ledger_db_file.display()
                    )
                });
            }
            None => {
                std::fs::create_dir_all(self.ledger_db.clone())
                    .expect("Could not create ledger dir");
                LedgerDB::create(&self.ledger_db).expect("Could not create ledger_db");
                if !offline {
                    log::info!(
                        logger,
                        "Ledger DB {:?} does not exist, bootstrapping from peer, this may take a few minutes",
                        self.ledger_db
                    );
                    let block_data = get_origin_block_and_transactions()
                        .expect("Failed to download initial transactions");
                    let mut db = LedgerDB::open(&self.ledger_db).expect("Could not open ledger_db");
                    db.append_block_data(&block_data)
                        .expect("Failed to append initial transactions");
                    log::info!(logger, "Bootstrapping completed!");
                }
            }
        }

        // Open ledger and verify it has (at least) the origin block.
        log::debug!(logger, "Opening Ledger DB {:?}", self.ledger_db);
        let ledger_db = LedgerDB::open(&self.ledger_db)
            .unwrap_or_else(|_| panic!("Could not open ledger db inside {:?}", self.ledger_db));

        let num_blocks = ledger_db
            .num_blocks()
            .expect("Failed getting number of blocks");
        if num_blocks == 0 {
            log::info!(logger, "Ledger DB is empty. You can still perform some wallet actions, such as creating addresses, but you will not be able to sync Txos.");
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

/// The Webhook Setup object.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebhookConfig {
    pub url: Url,
    pub poll_interval: Duration,
}
