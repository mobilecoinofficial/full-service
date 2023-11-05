// Copyright (c) 2018-2023 MobileCoin, Inc.

//! The entrypoint for the Ledger Validator Service.

use clap::Parser;
use mc_attest_verifier::{MrSignerVerifier, Verifier, DEBUG_ENCLAVE};
use mc_common::logger::{create_app_logger, log, o};
use mc_full_service::check_host;
use mc_ledger_sync::{LedgerSyncServiceThread, PollingNetworkState, ReqwestTransactionsFetcher};
use mc_validator_service::{Config, Service};
use std::{
    process::exit,
    sync::{Arc, RwLock},
};

// Exit codes.
const EXIT_INVALID_HOST: i32 = 4;

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let config = Config::parse();

    // Exit if the user is not in an authorized country.
    if check_host::check_host_is_allowed_country_and_region().is_err() {
        eprintln!("Could not validate host");
        exit(EXIT_INVALID_HOST);
    }

    log::info!(logger, "Read Configs: {:?}", config);

    // Create enclave verifier.
    let mut mr_signer_verifier =
        MrSignerVerifier::from(mc_consensus_enclave_measurement::sigstruct());
    mr_signer_verifier
        .allow_hardening_advisories(mc_consensus_enclave_measurement::HARDENING_ADVISORIES);

    let mut verifier = Verifier::default();
    verifier.mr_signer(mr_signer_verifier).debug(DEBUG_ENCLAVE);

    log::debug!(logger, "Verifier: {:?}", verifier);

    // Create peer manager.
    let peer_manager = config.peers_config.create_peer_manager(verifier, &logger);

    // Create network state, transactions fetcher and ledger sync.
    let network_state = Arc::new(RwLock::new(PollingNetworkState::new(
        config.peers_config.quorum_set(),
        peer_manager.clone(),
        logger.clone(),
    )));

    let transactions_fetcher = ReqwestTransactionsFetcher::new(
        config
            .peers_config
            .tx_source_urls
            .clone()
            .unwrap_or_default(),
        logger.clone(),
    )
    .expect("Failed creating ReqwestTransactionsFetcher");

    // Create the ledger_db.
    let ledger_db = config.ledger_db_config.create_or_open_ledger_db(
        || {
            transactions_fetcher
                .get_origin_block_and_transactions()
                .map_err(|err| err.to_string())
        },
        false,
        &logger,
    );

    // Start ledger sync thread.
    let _ledger_sync_service_thread = LedgerSyncServiceThread::new(
        ledger_db.clone(),
        peer_manager.clone(),
        network_state,
        transactions_fetcher,
        config.poll_interval,
        logger.clone(),
    );

    // Start GRPC service.
    let _service = Service::new(
        &config.listen_uri,
        config.peers_config.chain_id,
        ledger_db,
        peer_manager,
        logger,
    );

    // Sleep indefinitely.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
