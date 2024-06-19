// Copyright (c) 2018-2023 MobileCoin, Inc.

//! The entrypoint for the Ledger Validator Service.

use clap::Parser;
use mc_attest_core::MrSigner;
use mc_attestation_verifier::{TrustedIdentity, TrustedMrSignerIdentity};
use mc_common::logger::{create_app_logger, log, o};
use mc_ledger_sync::{LedgerSyncServiceThread, PollingNetworkState, ReqwestTransactionsFetcher};
use mc_validator_service::{Config, Service};
use std::{
    process::exit,
    sync::{Arc, RwLock},
};

// Exit codes.

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let config = Config::parse();

    log::info!(logger, "Read Configs: {:?}", config);

    // Create enclave trusted identity.
    let config_advisories: Vec<&str> = vec![];
    let signature = mc_consensus_enclave_measurement::sigstruct();
    let trusted_identity = TrustedIdentity::MrSigner(TrustedMrSignerIdentity::new(
        MrSigner::from(signature.mrsigner()),
        signature.product_id(),
        signature.version(),
        config_advisories,
        mc_consensus_enclave_measurement::HARDENING_ADVISORIES,
    ));

    log::debug!(logger, "TrustedIdentity: {:?}", trusted_identity);

    // Create peer manager.
    let peer_manager = config
        .peers_config
        .create_peer_manager(trusted_identity, &logger);

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
