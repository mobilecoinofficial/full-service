use mc_util_uri::ConsensusClientUri;
use serde::Serialize;
use std::path::PathBuf;
use structopt::StructOpt;
se mc_util_parse::parse_duration_in_seconds;

/// Configuration options for the validator service
#[derive(Clone, Debug, StructOpt, Serialize)]
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

    /*
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
    */


    /// How many seconds to wait between polling.
    #[structopt(long, default_value = "5", parse(try_from_str=parse_duration_in_seconds))]
    pub poll_interval: Duration,
}
