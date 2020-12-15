// Copyright (c) 2020 MobileCoin Inc.

//! MobileCoin wallet service

#![feature(proc_macro_hygiene, decl_macro)]
use diesel::{prelude::*, SqliteConnection};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use mc_attest_core::{MrSignerVerifier, Verifier, DEBUG_ENCLAVE};
use mc_common::logger::{create_app_logger, log, o};
use mc_ledger_sync::{LedgerSyncServiceThread, PollingNetworkState, ReqwestTransactionsFetcher};
use mc_wallet_service::config::APIConfig;
use mc_wallet_service::wallet::{rocket, WalletState};
use mc_wallet_service::{WalletDb, WalletService};
use std::sync::{Arc, RwLock};
use structopt::StructOpt;

#[allow(unused_imports)] // Needed for embedded_migrations!
#[macro_use]
extern crate diesel_migrations;

embed_migrations!("migrations/");

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

    // Connect to the database and run the migrations
    let conn =
        SqliteConnection::establish(&config.wallet_db.to_str().unwrap()).unwrap_or_else(|err| {
            panic!(
                "Cannot connect to {:?} database: {:?}",
                config.wallet_db, err
            )
        });

    embedded_migrations::run(&conn).expect("failed running migrations");

    let wallet_db = WalletDb::new_from_url(
        config
            .wallet_db
            .to_str()
            .expect("Could not get wallet_db path"),
        logger.clone(),
    )
    .expect("Could not access wallet db");

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
        config.quorum_set(),
        peer_manager.clone(),
        logger.clone(),
    )));

    let transactions_fetcher = ReqwestTransactionsFetcher::new(
        config.tx_source_urls.clone().unwrap_or_default(),
        logger.clone(),
    )
    .expect("Failed creating ReqwestTransactionsFetcher");

    // Create the ledger_db.
    let ledger_db = config.create_or_open_ledger_db(&logger, &transactions_fetcher);

    // Start ledger sync thread unless running in offline mode.
    let _ledger_sync_service_thread = if config.offline {
        None
    } else {
        Some(LedgerSyncServiceThread::new(
            ledger_db.clone(),
            peer_manager.clone(),
            network_state.clone(),
            transactions_fetcher,
            config.poll_interval,
            logger.clone(),
        ))
    };

    let state = WalletState {
        service: WalletService::new(
            wallet_db,
            ledger_db,
            peer_manager,
            network_state,
            config.get_fog_pubkey_resolver(logger.clone()).map(Arc::new),
            config.num_workers,
            logger,
        ),
    };

    let rocket = rocket(rocket_config, state);

    rocket.launch();
}
