// Copyright (c) 2020-2021 MobileCoin Inc.

//! MobileCoin wallet service

#![feature(proc_macro_hygiene, decl_macro)]
use clap::Parser;
use diesel::{connection::SimpleConnection, prelude::*, SqliteConnection};
use dotenv::dotenv;
use mc_attest_core::MrSigner;
use mc_attestation_verifier::{TrustedIdentity, TrustedMrSignerIdentity};
use mc_common::logger::{create_app_logger, log, o, Logger};
use mc_connection::ConnectionManager;
use mc_consensus_scp::QuorumSet;
use mc_fog_report_resolver::FogResolver;
use mc_full_service::{
    check_host,
    config::{APIConfig, NetworkConfig},
    wallet::{consensus_backed_rocket, validator_backed_rocket, APIKeyState, WalletState},
    ValidatorLedgerSyncThread, WalletDb, WalletService,
};
use mc_ledger_sync::{LedgerSyncServiceThread, PollingNetworkState, ReqwestTransactionsFetcher};
use mc_util_uri::ConnectionUri;
use mc_validator_api::ValidatorUri;
use mc_validator_connection::ValidatorConnection;
use mc_watcher::{watcher::WatcherSyncThread, watcher_db::create_or_open_rw_watcher_db};
use rocket::{launch, Build, Rocket};
use std::{
    env,
    net::IpAddr,
    process::exit,
    str::FromStr,
    sync::{Arc, RwLock},
};

#[allow(unused_imports)] // Needed for embedded_migrations!
#[macro_use]
extern crate diesel_migrations;

// Exit codes.
const EXIT_NO_DATABASE_CONNECTION: i32 = 2;
const EXIT_WRONG_PASSWORD: i32 = 3;
const EXIT_INVALID_HOST: i32 = 4;

#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();

    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = APIConfig::parse();

    // Exit if the user is not in an authorized country.
    if !cfg!(debug_assertions)
        && !config.offline
        && config.validator.is_none()
        && check_host::check_host_is_allowed_country_and_region().is_err()
    {
        eprintln!("Could not validate host");
        exit(EXIT_INVALID_HOST);
    }

    let (logger, global_logger_guard) = create_app_logger(o!());

    // This is necessary to prevent the logger from being reset when it goes out of
    // scope so that rocket can use it in its own async context
    global_logger_guard.cancel_reset();

    let wallet_db = match config.wallet_db {
        Some(ref wallet_db_path_buf) => {
            let wallet_db_path = wallet_db_path_buf.to_str().unwrap();
            // Connect to the database and run the migrations
            let conn = &mut SqliteConnection::establish(wallet_db_path).unwrap_or_else(|err| {
                eprintln!("Cannot open database {wallet_db_path:?}: {err:?}");
                exit(EXIT_NO_DATABASE_CONNECTION);
            });
            WalletDb::set_db_encryption_key_from_env(conn);
            WalletDb::try_change_db_encryption_key_from_env(conn);
            if !WalletDb::check_database_connectivity(conn) {
                eprintln!("Incorrect password for database {wallet_db_path:?}.");
                exit(EXIT_WRONG_PASSWORD);
            };
            WalletDb::add_mising_migrations(conn);
            conn.batch_execute("PRAGMA foreign_keys = OFF;")
                .expect("failed disabling foreign keys");
            WalletDb::run_migrations(conn);
            WalletDb::validate_foreign_keys(conn);
            conn.batch_execute("PRAGMA foreign_keys = ON;")
                .expect("failed enabling foreign keys");
            WalletDb::run_proto_conversions_if_necessary(conn);
            log::info!(logger, "Connected to database.");

            Some(WalletDb::new_from_url(wallet_db_path, 10).expect("Could not access wallet db"))
        }
        None => None,
    };

    let rocket_config = rocket::Config {
        address: IpAddr::from_str(&config.listen_host).expect("failed parsing host"),
        port: config.listen_port,
        ..rocket::Config::default()
    };

    let chain_id = config.peers_config.chain_id.clone();
    let tx_sources: Option<Vec<String>> = config.peers_config.tx_source_urls.clone();
    let peers: Option<Vec<String>> = config.peers_config.peers.clone().map(|peers| {
        peers
            .iter()
            .map(|peer_uri| peer_uri.url().clone().into())
            .collect()
    });

    let network_config = NetworkConfig {
        offline: config.offline,
        chain_id,
        peers,
        tx_sources,
    };

    let rocket = if let Some(validator_uri) = config.validator.as_ref() {
        validator_backed_full_service(
            validator_uri,
            &config,
            network_config,
            wallet_db,
            rocket_config,
            logger,
        )
    } else {
        consensus_backed_full_service(&config, network_config, wallet_db, rocket_config, logger)
    };

    let api_key = env::var("MC_API_KEY").unwrap_or_default();
    rocket.manage(APIKeyState(api_key))
}

fn consensus_backed_full_service(
    config: &APIConfig,
    network_config: NetworkConfig,
    wallet_db: Option<WalletDb>,
    rocket_config: rocket::Config,
    logger: Logger,
) -> Rocket<Build> {
    
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
    let peer_manager = config.peers_config.create_peer_manager(trusted_identity, &logger);

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
        config.offline,
        &logger,
    );

    // Start ledger sync thread unless running in offline mode.
    let ledger_sync_service_thread = if config.offline {
        None
    } else {
        Some(LedgerSyncServiceThread::new(
            ledger_db.clone(),
            peer_manager.clone(),
            network_state.clone(),
            transactions_fetcher.clone(),
            config.poll_interval,
            logger.clone(),
        ))
    };

    // Optionally instantiate the watcher sync thread and get the watcher_db handle.
    let (watcher_db, watcher_sync_thread) = match &config.watcher_db {
        Some(watcher_db_path) => {
            log::info!(logger, "Launching watcher.");

            log::info!(logger, "Opening watcher db at {:?}.", watcher_db_path);
            let watcher_db = create_or_open_rw_watcher_db(
                watcher_db_path,
                &transactions_fetcher.source_urls,
                logger.clone(),
            )
            .expect("Could not create or open WatcherDB");

            // Start watcher db sync thread, unless running in offline mode.
            let watcher_sync_thread = if config.offline {
                panic!("Attempted to start watcher but we are configured in offline mode");
            } else {
                log::info!(logger, "Starting watcher sync thread");
                Some(
                    WatcherSyncThread::new(
                        watcher_db.clone(),
                        ledger_db.clone(),
                        config.poll_interval,
                        false,
                        logger.clone(),
                    )
                    .expect("Failed starting watcher thread"),
                )
            };
            (Some(watcher_db), watcher_sync_thread)
        }
        None => (None, None),
    };

    let service = WalletService::new(
        wallet_db,
        ledger_db,
        watcher_db,
        peer_manager,
        network_config,
        network_state,
        config.get_fog_resolver_factory(logger.clone()),
        config.offline,
        config.t3_sync_config.clone(),
        logger,
    );

    consensus_backed_rocket(rocket_config, config.allowed_origin.clone())
        .manage(WalletState { service })
        .manage(ledger_sync_service_thread)
        .manage(watcher_sync_thread)
}

fn validator_backed_full_service(
    validator_uri: &ValidatorUri,
    config: &APIConfig,
    network_config: NetworkConfig,
    wallet_db: Option<WalletDb>,
    rocket_config: rocket::Config,
    logger: Logger,
) -> Rocket<Build> {
    if config.watcher_db.is_some() {
        panic!("Watcher syncing is not yet supported in a validator configuration");
    }

    let validator_conn = ValidatorConnection::new(
        validator_uri,
        config.peers_config.chain_id.clone(),
        logger.clone(),
    );

    // Create the ledger_db.
    let ledger_db = config.ledger_db_config.create_or_open_ledger_db(
        || {
            // Get the origin block.
            let blocks_data = validator_conn
                .get_blocks_data(0, 1)
                .map_err(|err| err.to_string())?;
            assert_eq!(blocks_data.len(), 1);

            Ok(blocks_data[0].clone())
        },
        false,
        &logger,
    );

    // Create connections manager.
    let conn_manager = ConnectionManager::new(vec![validator_conn.clone()], logger.clone());

    // Create network state
    // Note: There's onlu one node but we still need a quorum set.
    let node_ids = conn_manager.responder_ids();
    let quorum_set = QuorumSet::new_with_node_ids(node_ids.len() as u32, node_ids);

    let network_state = Arc::new(RwLock::new(PollingNetworkState::new(
        quorum_set,
        conn_manager.clone(),
        logger.clone(),
    )));

    // Create the ledger sync thread.
    let ledger_sync_thread = ValidatorLedgerSyncThread::new(
        validator_uri,
        config.peers_config.chain_id.clone(),
        config.poll_interval,
        ledger_db.clone(),
        network_state.clone(),
        logger.clone(),
    );

    let fog_ingest_identity = config.get_fog_ingest_identity();
    let logger2 = logger.clone();
    let service = WalletService::new(
        wallet_db,
        ledger_db,
        None,
        conn_manager,
        network_config,
        network_state,
        Arc::new(move |fog_uris| -> Result<FogResolver, String> {
            if fog_uris.is_empty() {
                Ok(Default::default())
            } else if let Some(trusted_identity) = fog_ingest_identity.as_ref() {
                let report_responses = validator_conn
                    .fetch_fog_reports(fog_uris.iter().cloned())
                    .map_err(|err| {
                    format!("Error fetching fog reports (via validator) for {fog_uris:?}: {err}")
                })?;

                log::debug!(logger2, "Got report responses {:?}", report_responses);
                Ok(FogResolver::new(report_responses, vec![trusted_identity])
                    .expect("Could not construct fog resolver"))
            } else {
                Err(
                    "Some recipients have fog, but no fog ingest report verifier was configured"
                        .to_string(),
                )
            }
        }),
        false,
        config.t3_sync_config.clone(),
        logger,
    );

    validator_backed_rocket(rocket_config, config.allowed_origin.clone())
        .manage(WalletState { service })
        .manage(ledger_sync_thread)
}
