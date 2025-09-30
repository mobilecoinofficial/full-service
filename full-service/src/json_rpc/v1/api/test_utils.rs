// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{
    config::NetworkConfig,
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::JsonRPCResponse,
        v1::api::{
            request::JsonCommandRequest, response::JsonCommandResponse, wallet::wallet_api_inner,
        },
    },
    service::{t3_sync::T3Config, WalletService},
    test_utils::{
        get_resolver_factory, get_test_ledger, setup_peer_manager_and_network_state,
        MockBlockchainPollingNetworkState, TestFogPubkeyResolver, WalletDbTestContext,
    },
    wallet::{APIKeyState, ApiKeyGuard},
};

use mc_account_keys::PublicAddress;
use mc_common::logger::{log, Logger};
use mc_connection_test_utils::MockBlockchainConnection;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::PollingNetworkState;

use rand::rngs::StdRng;
use rocket::{
    http::{ContentType, Header, Status},
    local::blocking::Client,
    post, routes,
    serde::json::{json, Json, Value as JsonValue},
    Build,
};

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
#[post("/wallet", format = "json", data = "<command>")]
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

pub fn create_test_setup(
    mut rng: &mut StdRng,
    logger: Logger,
) -> (
    rocket::Rocket<Build>,
    LedgerDB,
    WalletDbTestContext,
    MockBlockchainPollingNetworkState,
) {
    let db_test_context = WalletDbTestContext::default();
    let wallet_db = db_test_context.get_db_instance(logger.clone());
    let known_recipients: Vec<PublicAddress> = Vec::new();
    let ledger_db = get_test_ledger(5, &known_recipients, BASE_TEST_BLOCK_HEIGHT, &mut rng);
    let (peer_manager, network_state) =
        setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone(), false);

    let network_setup_config = NetworkConfig {
        offline: false,
        chain_id: "rust_tests".to_string(),
        peers: None,
        tx_sources: None,
    };

    let service = WalletService::new(
        Some(wallet_db),
        ledger_db.clone(),
        None,
        peer_manager,
        network_setup_config,
        network_state.clone(),
        get_resolver_factory(rng).unwrap(),
        false,
        T3Config::default(),
        None,
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
    MockBlockchainPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, logger);

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
    MockBlockchainPollingNetworkState,
) {
    let (rocket_instance, ledger_db, db_test_context, network_state) =
        create_test_setup(rng, logger);

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
    log::info!(logger, "Attempting dispatch of\n{}\n", request_body,);

    let res = client
        .post("/wallet")
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
        .post("/wallet")
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
        .post("/wallet")
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
        .post("/wallet")
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
