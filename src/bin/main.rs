// Copyright (c) 2020 MobileCoin Inc.

//! MobileCoin wallet service

#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use mc_common::logger::{create_app_logger, o};
use mc_wallet_service::service::{rocket, State};
use mc_wallet_service::{WalletDb, WalletService};
use std::path::PathBuf;
use structopt::StructOpt;

/// Command line config
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

    /// Path to WalletDb
    #[structopt(long, default_value = "/tmp/walletdb", parse(from_os_str))]
    pub wallet_db: PathBuf,
}

fn main() {
    dotenv().ok();

    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = APIConfig::from_args();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let rocket_config: rocket::Config =
        rocket::Config::build(rocket::config::Environment::Development)
            .address(&config.listen_host)
            .port(config.listen_port)
            .unwrap();

    let walletdb = WalletDb::new_from_url(
        config
            .wallet_db
            .to_str()
            .expect("Could not get wallet_db path"),
    )
    .expect("Could not access wallet db");
    let state = State {
        service: WalletService::new(walletdb, logger),
    };

    let rocket = rocket(rocket_config, state);

    rocket.launch();
}
