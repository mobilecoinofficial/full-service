// Copyright (c) 2020-2023 MobileCoin Inc.

use clap::Parser;
use mc_signer::service::api::{signer_service_api, version};
use rocket::{self, launch, routes, Build, Rocket};
use std::{net::IpAddr, str::FromStr};

/// Command line config for the Wallet API
#[derive(Clone, Debug, Parser)]
#[clap(
    name = "signer-service",
    about = "An HTTP signer service for MobileCoin",
    version
)]
pub struct APIConfig {
    /// Host to listen on.
    #[clap(long, default_value = "127.0.0.1", env = "MC_SIGNER_LISTEN_HOST")]
    pub listen_host: String,

    /// Port to start webserver on.
    #[clap(long, default_value = "9092", env = "MC_SIGNER_LISTEN_PORT")]
    pub listen_port: u16,
}

#[launch]
fn rocket() -> Rocket<Build> {
    let config = APIConfig::parse();

    let rocket_config = rocket::Config {
        address: IpAddr::from_str(&config.listen_host).expect("failed parsing host"),
        port: config.listen_port,
        ..rocket::Config::default()
    };

    rocket::custom(rocket_config).mount("/", routes![signer_service_api, version])
}
