// Copyright (c) 2020-2021 MobileCoin Inc.

//! Config definition and processing for Wallet Service.

use mc_attest_verifier::{MrSignerVerifier, Verifier, DEBUG_ENCLAVE};
use mc_common::{
    logger::{log, Logger},
    ResponderId,
};
use mc_connection::{ConnectionManager, HardcodedCredentialsProvider, ThickClient};
use mc_consensus_scp::QuorumSet;
use mc_fog_report_connection::GrpcFogReportConnection;
use mc_fog_report_validation::FogResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::ReqwestTransactionsFetcher;
use mc_sgx_css::Signature;
use mc_util_uri::{ConnectionUri, ConsensusClientUri, FogUri};

use displaydoc::Display;
#[cfg(feature = "ip-check")]
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
};
use std::{
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use structopt::StructOpt;

#[derive(Display, Debug)]
pub enum ConfigError {
    /// Error parsing json {0}
    Json(serde_json::Error),

    /// Error handling reqwest {0}
    Reqwest(reqwest::Error),

    /// Invalid country
    InvalidCountry,

    /// Data missing in the response {0}
    DataMissing(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<reqwest::Error> for ConfigError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

/// Command line config for the Wallet API
#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "full-service", about = "An HTTP wallet service for MobileCoin")]
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

    /// Path to existing ledger db that contains the origin block, used when
    /// initializing new ledger dbs.
    #[structopt(long)]
    pub ledger_db_bootstrap: Option<String>,

    #[structopt(flatten)]
    pub peers_config: PeersConfig,

    /// Quorum set for ledger syncing. By default, the quorum set would include
    /// all peers.
    ///
    /// The quorum set is represented in JSON. For example:
    /// {"threshold":1,"members":[{"type":"Node","args":"node2.test.mobilecoin.
    /// com:443"},{"type":"Node","args":"node3.test.mobilecoin.com:443"}]}
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

    /// Fog ingest enclave CSS file (needed in order to enable sending
    /// transactions to fog recipients).
    #[structopt(long, parse(try_from_str=load_css_file))]
    pub fog_ingest_enclave_css: Option<Signature>,
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

fn load_css_file(filename: &str) -> Result<Signature, String> {
    let bytes =
        fs::read(filename).map_err(|err| format!("Failed reading file '{}': {}", filename, err))?;
    let signature = Signature::try_from(&bytes[..])
        .map_err(|err| format!("Failed parsing CSS file '{}': {}", filename, err))?;
    Ok(signature)
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

    /// Get the attestation verifier used to verify fog reports when sending to
    /// fog recipients.
    pub fn get_fog_ingest_verifier(&self) -> Option<Verifier> {
        self.fog_ingest_enclave_css.as_ref().map(|signature| {
            let mr_signer_verifier = {
                let mut mr_signer_verifier = MrSignerVerifier::new(
                    signature.mrsigner().into(),
                    signature.product_id(),
                    signature.version(),
                );
                mr_signer_verifier.allow_hardening_advisories(&["INTEL-SA-00334"]);
                mr_signer_verifier
            };

            let mut verifier = Verifier::default();
            verifier.debug(DEBUG_ENCLAVE).mr_signer(mr_signer_verifier);
            verifier
        })
    }

    /// Get the function which creates FogResolver given a list of recipient
    /// addresses.
    ///
    /// The string error should be mapped by invoker of this factory to
    /// Error::FogError.
    pub fn get_fog_resolver_factory(
        &self,
        logger: Logger,
    ) -> Arc<dyn Fn(&[FogUri]) -> Result<FogResolver, String> + Send + Sync> {
        let env = Arc::new(
            grpcio::EnvBuilder::new()
                .name_prefix("FogPubkeyResolver-RPC".to_string())
                .build(),
        );

        let conn = GrpcFogReportConnection::new(env, logger.clone());

        let verifier = self.get_fog_ingest_verifier();

        Arc::new(move |fog_uris| -> Result<FogResolver, String> {
            if fog_uris.is_empty() {
                Ok(Default::default())
            } else if let Some(verifier) = verifier.as_ref() {
                let report_responses = conn
                    .fetch_fog_reports(fog_uris.iter().cloned())
                    .map_err(|err| format!("Failed fetching fog reports: {}", err))?;
                log::debug!(logger, "Got report responses {:?}", report_responses);
                Ok(FogResolver::new(report_responses, verifier)
                    .expect("Could not construct fog resolver"))
            } else {
                Err(
                    "Some recipients have fog, but no fog ingest report verifier was configured"
                        .to_string(),
                )
            }
        })
    }

    pub fn create_or_open_ledger_db(
        &self,
        logger: &Logger,
        transactions_fetcher: &ReqwestTransactionsFetcher,
    ) -> LedgerDB {
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
        let ledger_db_file = Path::new(&self.ledger_db).join("data.mdb");
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
                std::fs::create_dir_all(self.ledger_db.clone())
                    .expect("Could not create ledger dir");
                LedgerDB::create(&self.ledger_db).expect("Could not create ledger_db");
                if !self.offline {
                    log::info!(
                        logger,
                        "Ledger DB {:?} does not exist, bootstrapping from peer, this may take a few minutes",
                        self.ledger_db
                    );
                    let block_data = transactions_fetcher
                        .get_origin_block_and_transactions()
                        .expect("Failed to download initial transactions");
                    let mut db = LedgerDB::open(&self.ledger_db).expect("Could not open ledger_db");
                    db.append_block(
                        block_data.block(),
                        block_data.contents(),
                        block_data.signature().clone(),
                    )
                    .expect("Failed to appened initial transactions");
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

    /// Ensure local IP address is valid.
    ///
    /// Uses ipinfo.io for getting details about IP address.
    ///
    /// Note, both of these services are free tier and rate-limited.
    #[cfg(feature = "ip-check")]
    pub fn validate_host(&self) -> Result<(), ConfigError> {
        let client = Client::builder().gzip(true).use_rustls_tls().build()?;
        let mut json_headers = HeaderMap::new();
        json_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let response = client
            .get("https://ipinfo.io/json/")
            .headers(json_headers)
            .send()?
            .error_for_status()?;
        let data = response.text()?;
        let data_json: serde_json::Value = serde_json::from_str(&data)?;

        let data_missing_err = Err(ConfigError::DataMissing(data_json.to_string()));
        let country: &str = match data_json["country"].as_str() {
            Some(c) => c,
            None => return data_missing_err,
        };
        let region: &str = match data_json["region"].as_str() {
            Some(r) => r,
            None => return data_missing_err,
        };

        let err = Err(ConfigError::InvalidCountry);
        match country {
            "IR" | "SY" | "CU" | "KP" => err,
            "UA" => match region {
                "Crimea" => err,
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    #[cfg(not(feature = "ip-check"))]
    pub fn validate_host(&self) -> Result<(), ConfigError> {
        Ok(())
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
    ) -> Vec<ThickClient<HardcodedCredentialsProvider>> {
        self.peers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|client_uri| {
                ThickClient::new(
                    client_uri.clone(),
                    verifier.clone(),
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
        verifier: Verifier,
        logger: &Logger,
    ) -> ConnectionManager<ThickClient<HardcodedCredentialsProvider>> {
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
