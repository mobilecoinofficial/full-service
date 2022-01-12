use mc_full_service::config::PeersConfig;
use mc_util_parse::parse_duration_in_seconds;
use std::{path::PathBuf, time::Duration};
use structopt::StructOpt;

/// Configuration options for the validator service
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "mc-validator-service",
    about = "Ledger validator service. Provides a ledger and transaction proxying for Full Service (without attestation)."
)]
pub struct Config {
    /// Listening URI.
    // #[structopt(long)]
    // pub listen_uri: ValidatorServiceUri,

    /// Path to LedgerDB.
    #[structopt(long, parse(from_os_str))]
    pub ledger_db: PathBuf,

    /// Path to existing ledger db that contains the origin block, used when
    /// initializing new ledger dbs.
    #[structopt(long)]
    pub ledger_db_bootstrap: Option<String>,

    /// The location for the network.toml/json configuration file.
    #[structopt(flatten)]
    pub peers_config: PeersConfig,

    /// How many seconds to wait between polling.
    #[structopt(long, default_value = "5", parse(try_from_str=parse_duration_in_seconds))]
    pub poll_interval: Duration,
}
