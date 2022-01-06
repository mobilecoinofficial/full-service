// Copyright 2018-2021 MobileCoin, Inc.

//! Main Method for the stub server

use validator_server::Config;
use mc_common::logger::{create_app_logger, log, o};
use structopt::StructOpt;

fn main() {
    mc_common::setup_panic_handler();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let config = Config::from_args();

    log::info!(logger, "Read Configs: {:?}", config);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
