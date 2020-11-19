// Copyright (c) 2020 MobileCoin Inc.

//! MobileCoin wallet service

#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use mc_common::logger::{create_app_logger, log, o};
use mc_wallet_service::service::rocket;
use rocket::config::{Config, Environment, Value};
use rocket_contrib::databases::database_config;
use std::collections::HashMap;
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
    // FIXME: add all the mobilecoind config params too
}

fn main() {
    dotenv().ok();

    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = APIConfig::from_args();

    let (logger, _global_logger_guard) = create_app_logger(o!());
    log::info!(
        logger,
        "Starting MobileCoin Wallet Service on {}:{}",
        config.listen_host,
        config.listen_port,
    );
    log::info!(
        logger,
        "\x1b[1;33m cur dir = {:?}\x1b[0m",
        std::env::current_dir()
    );

    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    // This is the same as the following TOML:
    // my_db = { url = "database.sqlite" }
    database_config.insert("url", Value::from("./src/db/test.db"));
    databases.insert("posts_db", Value::from(database_config));

    let rocket_config: rocket::Config =
        rocket::Config::build(rocket::config::Environment::Development)
            .address(&config.listen_host)
            .port(config.listen_port)
            .extra("databases", databases)
            .unwrap();
    log::info!(
        logger,
        "\x1b[1;32m rocket config = {:?}\x1b[0m",
        rocket_config
    );

    let rocket = rocket(rocket_config);
    log::info!(logger, "\x1b[1;33m HELLO WORLD\x1b[0m");
    // let config = database_config("posts_db", rocket.config()).unwrap();
    // log::info!(logger, "\x1b[1;36m database config = {:?}\x1b[0m", config);

    rocket.launch();
}
