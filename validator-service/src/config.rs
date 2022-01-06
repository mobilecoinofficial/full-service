use mc_util_uri::ConsensusClientUri;
use serde::Serialize;
use std::path::{PathBuf};
use structopt::StructOpt;

/// Configuration options for the validator service
#[derive(Clone, Debug, StructOpt, Serialize)]
#[structopt(name = "validator-service", about = "Ledger validator service. Provides a ledger for Full Service without attestation.")]
pub struct Config {
    /// Client listening URI.
    #[structopt(long)]
    pub client_listen_uri: ConsensusClientUri,

    /// Path to LedgerDB
    #[structopt(long, parse(from_os_str))]
    pub ledger_db: PathBuf,

    /// Path to existing ledger db that contains the origin block, used when
    /// initializing new ledger dbs.
    #[structopt(long)]
    pub ledger_db_bootstrap: Option<String>,

    /// The location for the network.toml/json configuration file.
    #[structopt(long = "network", parse(from_os_str))]
    pub network_path: PathBuf,
    
    /// Offline mode.
    #[structopt(long)]
    pub offline: bool,
}