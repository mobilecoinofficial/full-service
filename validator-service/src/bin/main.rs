// Copyright 2018-2021 MobileCoin, Inc.

//! Main Method for the stub server

use mc_common::logger::{create_app_logger, log, o};
use mc_full_service::{self, config::APIConfig};
use mc_ledger_sync::ReqwestTransactionsFetcher;
use structopt::StructOpt;
use validator_server::Server;

fn main() {
    mc_common::setup_panic_handler();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let config = APIConfig::from_args();

    let transactions_fetcher = ReqwestTransactionsFetcher::new(
        config.tx_source_urls.clone().unwrap_or_default(),
        logger.clone(),
    )
    .expect("Failed creating ReqwestTransactionsFetcher");

    // Create the ledger_db.
    let ledger_db = config.create_or_open_ledger_db(&logger, &transactions_fetcher);

    log::info!(logger, "Read Configs: {:?}", config);

    let mut server = Server::new(ledger_db, logger.clone());
    log::info!(logger, "Build server");
    server.start();
    log::info!(logger, "Started server");

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
