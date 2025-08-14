// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{
    config::NetworkConfig,
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::JsonRPCResponse,
        v2::api::{
            request::JsonCommandRequest, response::JsonCommandResponse, wallet::wallet_api_inner,
        },
    },
    service::{t3_sync::T3Config, WalletService},
    test_utils::{
        get_resolver_factory, get_test_ledger, setup_peer_manager_and_network_state,
        MockPollingNetworkState, TestFogPubkeyResolver, WalletDbTestContext,
    },
    wallet::{APIKeyState, ApiKeyGuard},
};

use mc_account_keys::PublicAddress;
use mc_blockchain_types::BlockSignature;
use mc_common::logger::{log, Logger};
use mc_connection_test_utils::MockBlockchainConnection;
use mc_crypto_keys::Ed25519Pair;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::PollingNetworkState;
use mc_rand::{CryptoRng, RngCore};
use mc_util_from_random::FromRandom;
use mc_watcher::watcher_db::WatcherDB;

use rand::rngs::StdRng;
use rocket::{
    http::{ContentType, Header, Status},
    local::blocking::Client,
    post, routes,
    serde::json::{json, Json, Value as JsonValue},
    Build,
};
use tempdir::TempDir;
use url::Url;

use crate::config::WebhookConfig;
use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc, RwLock,
    },
    time::Duration,
};

pub fn get_free_port() -> u16 {
    static PORT_NR: AtomicUsize = AtomicUsize::new(0);
    PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
}

pub struct TestWalletState {
    pub service: WalletService<MockBlockchainConnection<LedgerDB>, TestFogPubkeyResolver>,
}

// Note: the reason this is duplicated from wallet.rs is to be able to pass the
// TestWalletState, which handles Mock objects.
#[post("/wallet/v2", format = "json", data = "<command>")]
async fn test_wallet_api(
    _guard: ApiKeyGuard,
    state: &rocket::State<TestWalletState>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse>>, String> {
    let req: JsonRPCRequest = command.0.clone();

    let mut response = JsonRPCResponse {
        method: Some(command.0.method),
        result: None,
        error: None,
        jsonrpc: "2.0".to_string(),
        id: command.0.id,
    };

    match wallet_api_inner(&state.service, JsonCommandRequest::try_from(&req)?).await {
        Ok(command_response) => {
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            response.error = Some(rpc_error);
        }
    };

    Ok(Json(response))
}

pub fn test_rocket(rocket_config: rocket::Config, state: TestWalletState) -> rocket::Rocket<Build> {
    rocket::custom(rocket_config)
        .mount("/", routes![test_wallet_api])
        .manage(state)
}

pub const BASE_TEST_BLOCK_HEIGHT: usize = 12;

fn create_test_watcher_db(
    ledger_db: &LedgerDB,
    logger: &Logger,
    rng: &mut (impl RngCore + CryptoRng),
) -> WatcherDB {
    let url1 = Url::parse("http://www.my_url1.com").unwrap();
    let url2 = Url::parse("http://www.my_url2.com").unwrap();
    let urls = [url1, url2];

    let db_tmp = TempDir::new("watcher").expect("Could not make tempdir for wallet db");
    WatcherDB::create(db_tmp.path()).expect("Failed to create WatcherDB");
    let watcher_db = WatcherDB::open_rw(db_tmp.path(), &urls, logger.clone()).unwrap();

    let signing_key_a = Ed25519Pair::from_random(rng);
    let signing_key_b = Ed25519Pair::from_random(rng);

    let filename = String::from("00/00");

    let blocks =
        (0..ledger_db.num_blocks().unwrap()).map(|index| ledger_db.get_block(index).unwrap());

    for block in blocks {
        let signed_block_a =
            BlockSignature::from_block_and_keypair(&block, &signing_key_a).unwrap();
        watcher_db
            .add_block_signature(&urls[0], block.index, signed_block_a, filename.clone())
            .unwrap();

        let signed_block_b =
            BlockSignature::from_block_and_keypair(&block, &signing_key_b).unwrap();
        watcher_db
            .add_block_signature(&urls[0], block.index, signed_block_b, filename.clone())
            .unwrap();
    }

    watcher_db
}

pub fn create_test_setup(
    mut rng: &mut StdRng,
    use_wallet_db: bool,
    use_watcher_db: bool,
    webhook_config: Option<WebhookConfig>,
    logger: Logger,
) -> (
    rocket::Rocket<Build>,
    LedgerDB,
    WalletDbTestContext,
    MockPollingNetworkState,
) {
    let db_test_context = WalletDbTestContext::default();
    let wallet_db = match use_wallet_db {
        true => Some(db_test_context.get_db_instance(logger.clone())),
        false => None,
    };
    let known_recipients: Vec<PublicAddress> = Vec::new();
    let ledger_db = get_test_ledger(5, &known_recipients, BASE_TEST_BLOCK_HEIGHT, &mut rng);
    let (peer_manager, network_state) =
        setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone(), false);

    let watcher_db = if use_watcher_db {
        Some(create_test_watcher_db(&ledger_db, &logger, &mut rng))
    } else {
        None
    };

    let network_setup_config = NetworkConfig {
        offline: false,
        chain_id: "rust_tests".to_string(),
        peers: None,
        tx_sources: None,
    };

    let service = WalletService::new(
        wallet_db,
        ledger_db.clone(),
        watcher_db,
        peer_manager,
        network_setup_config,
        network_state.clone(),
        get_resolver_factory(rng).unwrap(),
        false,
        T3Config::default(),
        webhook_config,
        logger,
    );

    let rocket_config = rocket::Config::figment()
        .merge(("port", get_free_port()))
        .extract()
        .unwrap();

    let rocket_instance = test_rocket(rocket_config, TestWalletState { service });

    (rocket_instance, ledger_db, db_test_context, network_state)
}

pub fn setup(
    rng: &mut StdRng,
    logger: Logger,
) -> (
    Client,
    LedgerDB,
    WalletDbTestContext,
    MockPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, true, false, None, logger);

    let rocket = rocket_instance.manage(APIKeyState("".to_string()));
    (
        Client::untracked(rocket).expect("valid rocket instance"),
        ledger_db,
        db_test_context,
        network_state,
    )
}

pub fn setup_with_webhook(
    rng: &mut StdRng,
    webhook_config: WebhookConfig,
    logger: Logger,
) -> (
    Client,
    LedgerDB,
    WalletDbTestContext,
    MockPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, true, false, Some(webhook_config), logger);

    let rocket = rocket_instance.manage(APIKeyState("".to_string()));
    (
        Client::untracked(rocket).expect("valid rocket instance"),
        ledger_db,
        db_test_context,
        network_state,
    )
}

pub fn setup_with_watcher(
    rng: &mut StdRng,
    logger: Logger,
) -> (
    Client,
    LedgerDB,
    WalletDbTestContext,
    MockPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, true, true, None, logger);

    let rocket = rocket_instance.manage(APIKeyState("".to_string()));
    (
        Client::untracked(rocket).expect("valid rocket instance"),
        ledger_db,
        db_test_context,
        network_state,
    )
}

pub fn setup_no_wallet_db(
    rng: &mut StdRng,
    logger: Logger,
) -> (
    Client,
    LedgerDB,
    WalletDbTestContext,
    MockPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, false, false, None, logger);

    let rocket = rocket_instance.manage(APIKeyState("".to_string()));
    (
        Client::untracked(rocket).expect("valid rocket instance"),
        ledger_db,
        db_test_context,
        network_state,
    )
}

pub fn setup_with_api_key(
    rng: &mut StdRng,
    logger: Logger,
    api_key: String,
) -> (
    Client,
    LedgerDB,
    WalletDbTestContext,
    MockPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, true, false, None, logger);

    let rocket = rocket_instance.manage(APIKeyState(api_key));

    (
        Client::untracked(rocket).expect("valid rocket instance"),
        ledger_db,
        db_test_context,
        network_state,
    )
}

pub fn dispatch(client: &Client, request_body: JsonValue, logger: &Logger) -> JsonValue {
    log::info!(logger, "Attempting dispatch of\n{:?}\n", request_body,);
    let request_body = request_body.to_string();

    let res = client
        .post("/wallet/v2")
        .header(ContentType::JSON)
        .body(request_body)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let response_body = res.into_string().unwrap();
    log::info!(logger, "Got response\n{}\n", response_body);

    let res: JsonValue = serde_json::from_str(&response_body).unwrap();
    res
}

pub fn dispatch_with_header(
    client: &Client,
    request_body: JsonValue,
    header: Header<'static>,
    logger: &Logger,
) -> JsonValue {
    log::info!(logger, "Attempting dispatch of\n{:?}\n", request_body,);
    let request_body = request_body.to_string();
    log::info!(logger, "Attempting dispatch of\n{}\n", request_body,);

    let res = client
        .post("/wallet/v2")
        .header(ContentType::JSON)
        .header(header)
        .body(request_body)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let response_body = res.into_string().unwrap();
    log::info!(logger, "Got response\n{}\n", response_body);

    let res: JsonValue = serde_json::from_str(&response_body).unwrap();
    res
}

pub fn dispatch_with_header_expect_error(
    client: &Client,
    request_body: JsonValue,
    header: Header<'static>,
    _logger: &Logger,
    expected_err: Status,
) {
    let res = client
        .post("/wallet/v2")
        .header(ContentType::JSON)
        .header(header)
        .body(request_body.to_string())
        .dispatch();
    assert_eq!(res.status(), expected_err);
}

pub fn dispatch_expect_error(
    client: &Client,
    request_body: JsonValue,
    logger: &Logger,
    expected_err: String,
) {
    let res = client
        .post("/wallet/v2")
        .header(ContentType::JSON)
        .body(request_body.to_string())
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let response_body = res.into_string().unwrap();

    log::info!(
        logger,
        "Attempted dispatch of {:?} got response {:?}",
        request_body,
        response_body
    );
    let response_json: serde_json::Value = serde_json::from_str(&response_body).unwrap();
    let expected_json: serde_json::Value = serde_json::from_str(&expected_err).unwrap();
    assert_eq!(response_json, expected_json);
}

pub fn wait_for_sync(
    client: &Client,
    ledger_db: &LedgerDB,
    network_state: &Arc<RwLock<PollingNetworkState<MockBlockchainConnection<LedgerDB>>>>,
    logger: &Logger,
) {
    let mut count = 0;
    loop {
        // Sleep to let the sync thread process the txos
        std::thread::sleep(Duration::from_millis(2000));

        // Check that syncing is working
        let body = json!({
            "jsonrpc": "2.0",
            "method": "get_wallet_status",
            "id": 1,
        });
        let res = dispatch(client, body, logger);
        let status = res["result"]["wallet_status"].clone();

        let is_synced_all = status["is_synced_all"].as_bool().unwrap();
        if is_synced_all {
            let local_height = status["local_block_height"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            assert_eq!(local_height, ledger_db.num_blocks().unwrap());
            // In the test context, we often add a block manually locally before updating
            // the network_state. In the wild, the local_height should never be
            // greater than the network_height.
            assert!(
                status["network_block_height"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
                    <= local_height
            );
            break;
        }

        // Have to manually call poll() on network state to get it to update for these
        // tests
        network_state.write().unwrap().poll();

        count += 1;
        if count > 10 {
            panic!("Service did not sync after 10 iterations");
        }
    }
}
