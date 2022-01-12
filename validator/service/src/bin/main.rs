// Copyright (c) 2018-2022 MobileCoin, Inc.

//! The entrypoint for the Ledger Validator Service.

use mc_common::logger::{create_app_logger, log, o};
use mc_validator_service::{Config, Service};
use structopt::StructOpt;

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let config = Config::from_args();

    log::info!(logger, "Read Configs: {:?}", config);

    // Start GRPC service.
    let _service = Service::new(&config.listen_uri, logger);

    // Sleep indefinitely.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
