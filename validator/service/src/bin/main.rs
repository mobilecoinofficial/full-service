// Copyright (c) 2018-2022 MobileCoin, Inc.

//! The entrypoint for the Ledger Validator Service.

use mc_attest_verifier::{MrSignerVerifier, Verifier, DEBUG_ENCLAVE};
use mc_common::logger::{create_app_logger, log, o};
use mc_ledger_sync::{LedgerSyncServiceThread, PollingNetworkState, ReqwestTransactionsFetcher};
use mc_validator_service::{Config, Service};
use std::sync::{Arc, RwLock};
use structopt::StructOpt;

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let config = Config::from_args();

    log::info!(logger, "Read Configs: {:?}", config);

    // Create enclave verifier.
    let mut mr_signer_verifier =
        MrSignerVerifier::from(mc_consensus_enclave_measurement::sigstruct());
    mr_signer_verifier.allow_hardening_advisory("INTEL-SA-00334");

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
    let ledger_db =
        config
            .ledger_db_config
            .create_or_open_ledger_db(&transactions_fetcher, false, &logger);

    // Start ledger sync thread.
    let _ledger_sync_service_thread = LedgerSyncServiceThread::new(
        ledger_db,
        peer_manager,
        network_state,
        transactions_fetcher,
        config.poll_interval,
        logger.clone(),
    );

    // Start GRPC service.
    let _service = Service::new(&config.listen_uri, ledger_db, logger);

    // Sleep indefinitely.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
