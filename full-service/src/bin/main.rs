// Copyright (c) 2020-2021 MobileCoin Inc.

//! MobileCoin wallet service

#![feature(proc_macro_hygiene, decl_macro)]
use bip39::Mnemonic;
use diesel::{prelude::*, SqliteConnection};
use dotenv::dotenv;
use mc_api::external::Tx;
use mc_attest_verifier::{MrSignerVerifier, Verifier, DEBUG_ENCLAVE};
use mc_common::logger::{create_app_logger, log, o, Logger};
use mc_connection::ConnectionManager;
use mc_consensus_scp::QuorumSet;
use mc_fog_report_validation::FogResolver;
use mc_full_service::{
    check_host,
    config::APIConfig,
    db::{
        account::AccountModel,
        models::{Account, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        Conn,
    },
    json_rpc::v1::models::{
        account::Account as AccountJSON, account_secrets::AccountSecrets,
        transaction_log::TransactionLog as JsonTransactionLog, txo::Txo as JsonTxo,
    },
    wallet::{consensus_backed_rocket, validator_backed_rocket, APIKeyState, WalletState},
    ValidatorLedgerSyncThread, WalletDb, WalletService,
};
use mc_ledger_sync::{LedgerSyncServiceThread, PollingNetworkState, ReqwestTransactionsFetcher};
use mc_mobilecoind_json::data_types::JsonTx;
use mc_validator_api::ValidatorUri;
use mc_validator_connection::ValidatorConnection;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use std::{
    convert::TryFrom,
    env,
    process::exit,
    sync::{Arc, RwLock},
};
use structopt::StructOpt;

#[allow(unused_imports)] // Needed for embedded_migrations!
#[macro_use]
extern crate diesel_migrations;

// Exit codes.
const EXIT_NO_DATABASE_CONNECTION: i32 = 2;
const EXIT_WRONG_PASSWORD: i32 = 3;
const EXIT_INVALID_HOST: i32 = 4;

fn main() {
    dotenv().ok();

    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = APIConfig::from_args();

    // Exit if the user is not in an authorized country.
    if !cfg!(debug_assertions)
        && !config.offline
        && config.validator.is_none()
        && check_host::check_host_is_allowed_country_and_region().is_err()
    {
        eprintln!("Could not validate host");
        exit(EXIT_INVALID_HOST);
    }

    let (logger, _global_logger_guard) = create_app_logger(o!());

    let rocket_config: rocket::Config =
        rocket::Config::build(rocket::config::Environment::Development)
            .address(&config.listen_host)
            .port(config.listen_port)
            .unwrap();

    // Connect to the database and run the migrations
    let conn =
        SqliteConnection::establish(config.wallet_db.to_str().unwrap()).unwrap_or_else(|err| {
            eprintln!("Cannot open database {:?}: {:?}", config.wallet_db, err);
            exit(EXIT_NO_DATABASE_CONNECTION);
        });
    WalletDb::set_db_encryption_key_from_env(&conn);
    WalletDb::try_change_db_encryption_key_from_env(&conn);
    if !WalletDb::check_database_connectivity(&conn) {
        eprintln!("Incorrect password for database {:?}.", config.wallet_db);
        exit(EXIT_WRONG_PASSWORD);
    };
    WalletDb::run_migrations(&conn);
    log::info!(logger, "Connected to database.");

    let wallet_db = WalletDb::new_from_url(
        config
            .wallet_db
            .to_str()
            .expect("Could not get wallet_db path"),
        10,
    )
    .expect("Could not access wallet db");

    if let Some(import_uri) = config.import_uri.clone() {
        import_accounts(&wallet_db.get_conn().unwrap(), &import_uri);
    }

    // Start WalletService based on our configuration
    if let Some(validator_uri) = config.validator.as_ref() {
        validator_backed_full_service(validator_uri, &config, wallet_db, rocket_config, logger)
    } else {
        consensus_backed_full_service(&config, wallet_db, rocket_config, logger)
    };
}

fn import_accounts(conn: &Conn, import_uri: &str) {
    let request_uri = format!("{}/wallet", import_uri);
    let get_accounts_body = json!({
        "method": "get_all_accounts",
        "jsonrpc": "2.0",
        "id": 1
    });
    let client = reqwest::blocking::Client::new();

    let response = client
        .post(request_uri.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(get_accounts_body.to_string())
        .send()
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();

    let account_map = response.get("result").unwrap().get("account_map").unwrap();
    let account_ids = response
        .get("result")
        .unwrap()
        .get("account_ids")
        .unwrap()
        .as_array()
        .unwrap();

    for account_id in account_ids {
        let account: AccountJSON = serde_json::from_value(
            account_map
                .get(account_id.as_str().unwrap())
                .unwrap()
                .clone(),
        )
        .unwrap();

        let get_account_secrets_body = json!({
            "method": "export_account_secrets",
            "jsonrpc": "2.0",
            "id": 1,
            "params": {
                "account_id": account_id.as_str().unwrap()
            }
        });

        let response = client
            .post(request_uri.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(get_account_secrets_body.to_string())
            .send()
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap();

        let account_secrets: AccountSecrets = serde_json::from_value(
            response
                .get("result")
                .unwrap()
                .get("account_secrets")
                .unwrap()
                .clone(),
        )
        .unwrap();

        if let Some(mnemonic) = account_secrets.mnemonic {
            let mnemonic = Mnemonic::from_phrase(&mnemonic, bip39::Language::English).unwrap();
            Account::import(
                &mnemonic,
                Some(account.name),
                0,
                Some(account.first_block_index.parse::<u64>().unwrap()),
                Some(account.next_subaddress_index.parse::<u64>().unwrap()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                conn,
            )
            .unwrap();
        } else if let Some(entropy) = account_secrets.entropy {
            panic!("Entropy not yet supported");
        } else {
            panic!("No entropy or mnemonic found for account {}", account_id);
        };

        let get_sent_transaction_logs_body = json!({
            "method": "get_sent_transaction_logs_for_account",
            "jsonrpc": "2.0",
            "id": 1,
            "params": {
                "account_id": account_id.as_str().unwrap()
            }
        });

        let response = client
            .post(request_uri.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(get_sent_transaction_logs_body.to_string())
            .send()
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap();

        let transaction_log_ids = response
            .get("result")
            .unwrap()
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();

        for transaction_log_id in transaction_log_ids {
            let transaction_log: JsonTransactionLog = serde_json::from_value(
                response
                    .get("result")
                    .unwrap()
                    .get("transaction_log_map")
                    .unwrap()
                    .get(transaction_log_id.as_str().unwrap())
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            for input_txo in &transaction_log.input_txos {
                let get_txo_body = json!({
                    "method": "get_txo",
                    "jsonrpc": "2.0",
                    "id": 1,
                    "params": {
                        "txo_id": input_txo.txo_id_hex
                    }
                });

                let response = client
                    .post(request_uri.clone())
                    .header(CONTENT_TYPE, "application/json")
                    .body(get_txo_body.to_string())
                    .send()
                    .unwrap()
                    .json::<serde_json::Value>()
                    .unwrap();

                let txo: JsonTxo = serde_json::from_value(
                    response.get("result").unwrap().get("txo").unwrap().clone(),
                )
                .unwrap();

                Txo::import_from_v1(txo, conn).unwrap();
            }

            for input_txo in &transaction_log.output_txos {
                let get_txo_body = json!({
                    "method": "get_txo",
                    "jsonrpc": "2.0",
                    "id": 1,
                    "params": {
                        "txo_id": input_txo.txo_id_hex
                    }
                });

                let response = client
                    .post(request_uri.clone())
                    .header(CONTENT_TYPE, "application/json")
                    .body(get_txo_body.to_string())
                    .send()
                    .unwrap()
                    .json::<serde_json::Value>()
                    .unwrap();

                let txo: JsonTxo = serde_json::from_value(
                    response.get("result").unwrap().get("txo").unwrap().clone(),
                )
                .unwrap();

                Txo::import_from_v1(txo, conn).unwrap();
            }

            for input_txo in &transaction_log.change_txos {
                let get_txo_body = json!({
                    "method": "get_txo",
                    "jsonrpc": "2.0",
                    "id": 1,
                    "params": {
                        "txo_id": input_txo.txo_id_hex
                    }
                });

                let response = client
                    .post(request_uri.clone())
                    .header(CONTENT_TYPE, "application/json")
                    .body(get_txo_body.to_string())
                    .send()
                    .unwrap()
                    .json::<serde_json::Value>()
                    .unwrap();

                let txo: JsonTxo = serde_json::from_value(
                    response.get("result").unwrap().get("txo").unwrap().clone(),
                )
                .unwrap();

                Txo::import_from_v1(txo, conn).unwrap();
            }

            TransactionLog::log_imported_from_v1(transaction_log, conn).unwrap();
        }
    }
}

fn consensus_backed_full_service(
    config: &APIConfig,
    wallet_db: WalletDb,
    rocket_config: rocket::Config,
    logger: Logger,
) {
    // Verifier
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
        config.offline,
        &logger,
    );

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

    let service = WalletService::new(
        wallet_db,
        ledger_db,
        peer_manager,
        network_state,
        config.get_fog_resolver_factory(logger.clone()),
        config.offline,
        logger,
    );
    let state = WalletState { service };

    let rocket = consensus_backed_rocket(rocket_config, state);
    let api_key = env::var("MC_API_KEY").unwrap_or_default();
    rocket.manage(APIKeyState(api_key)).launch();
}

fn validator_backed_full_service(
    validator_uri: &ValidatorUri,
    config: &APIConfig,
    wallet_db: WalletDb,
    rocket_config: rocket::Config,
    logger: Logger,
) {
    let validator_conn = ValidatorConnection::new(validator_uri, logger.clone());

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
    let _ledger_sync_thread = ValidatorLedgerSyncThread::new(
        validator_uri,
        config.poll_interval,
        ledger_db.clone(),
        network_state.clone(),
        logger.clone(),
    );

    let fog_ingest_verifier = config.get_fog_ingest_verifier();
    let logger2 = logger.clone();
    let service = WalletService::new(
        wallet_db,
        ledger_db,
        conn_manager,
        network_state,
        Arc::new(move |fog_uris| -> Result<FogResolver, String> {
            if fog_uris.is_empty() {
                Ok(Default::default())
            } else if let Some(verifier) = fog_ingest_verifier.as_ref() {
                let report_responses = validator_conn
                    .fetch_fog_reports(fog_uris.iter().cloned())
                    .map_err(|err| {
                    format!(
                        "Error fetching fog reports (via validator) for {:?}: {}",
                        fog_uris, err
                    )
                })?;

                log::debug!(logger2, "Got report responses {:?}", report_responses);
                Ok(FogResolver::new(report_responses, verifier)
                    .expect("Could not construct fog resolver"))
            } else {
                Err(
                    "Some recipients have fog, but no fog ingest report verifier was configured"
                        .to_string(),
                )
            }
        }),
        false,
        logger,
    );
    let state = WalletState { service };

    let rocket = validator_backed_rocket(rocket_config, state);
    let api_key = env::var("MC_API_KEY").unwrap_or_default();
    rocket.manage(APIKeyState(api_key)).launch();
}
