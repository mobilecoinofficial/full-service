use mc_full_service::config::{LedgerDbConfig, PeersConfig};
use mc_util_parse::parse_duration_in_seconds;
use mc_validator_api::ValidatorUri;
use std::time::Duration;
use structopt::StructOpt;

/// Configuration options for the validator service
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "mc-validator-service",
    about = "Ledger validator service. Provides a ledger and transaction proxying for Full Service (without attestation)."
)]
pub struct Config {
    /// Listening URI.
    #[structopt(long, default_value = "insecure-validator://127.0.0.1/")]
    pub listen_uri: ValidatorUri,

    #[structopt(flatten)]
    pub ledger_db_config: LedgerDbConfig,

    #[structopt(flatten)]
    pub peers_config: PeersConfig,

    /// How many seconds to wait between polling.
    #[structopt(long, default_value = "5", parse(try_from_str=parse_duration_in_seconds))]
    pub poll_interval: Duration,

    #[structopt(long)]
    pub chain_id: String,
}
