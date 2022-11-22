use clap::Parser;
use mc_full_service::config::{LedgerDbConfig, PeersConfig};
use mc_util_parse::parse_duration_in_seconds;
use mc_validator_api::ValidatorUri;
use std::time::Duration;

/// Configuration options for the validator service
#[derive(Clone, Debug, Parser)]
#[clap(
    name = "mc-validator-service",
    about = "Ledger validator service. Provides a ledger and transaction proxying for Full Service (without attestation)."
)]
pub struct Config {
    /// Listening URI.
    #[clap(
        long,
        default_value = "insecure-validator://127.0.0.1/",
        env = "MC_LISTEN_URI"
    )]
    pub listen_uri: ValidatorUri,

    #[clap(flatten)]
    pub ledger_db_config: LedgerDbConfig,

    #[clap(flatten)]
    pub peers_config: PeersConfig,

    /// How many seconds to wait between polling.
    #[clap(long, default_value = "5", value_parser = parse_duration_in_seconds, env = "MC_POLL_INTERVAL")]
    pub poll_interval: Duration,
}
