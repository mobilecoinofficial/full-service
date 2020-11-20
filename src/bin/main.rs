// Copyright (c) 2020 MobileCoin Inc.

//! MobileCoin wallet service

#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use mc_common::logger::{create_app_logger, o};
use mc_wallet_service::service::rocket;
use rocket::config::Value;
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
}

fn main() {
    dotenv().ok();

    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = APIConfig::from_args();

    let (_logger, _global_logger_guard) = create_app_logger(o!());

    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    // Note: This is the same as the following TOML in Rocket.toml:
    // wallet_db = { url = "./src/db/test.db" }
    // But we cannot use Rocket.toml because it is ignored Config::build
    database_config.insert("url", Value::from("./src/db/test.db"));
    databases.insert("wallet_db", Value::from(database_config));

    let rocket_config: rocket::Config =
        rocket::Config::build(rocket::config::Environment::Development)
            .address(&config.listen_host)
            .port(config.listen_port)
            .extra("databases", databases)
            .unwrap();

    let rocket = rocket(rocket_config);

    rocket.launch();
}
