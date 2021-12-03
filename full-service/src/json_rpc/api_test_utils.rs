// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{
    json_rpc::{
        json_rpc_request::{JsonCommandRequest, JsonRPCRequest},
        json_rpc_response::JsonRPCResponse,
        wallet::wallet_api_inner,
    },
    service::WalletService,
    test_utils::{
        get_resolver_factory, get_test_ledger, setup_peer_manager_and_network_state,
        WalletDbTestContext,
    },
};
use mc_account_keys::PublicAddress;
use mc_common::logger::{log, Logger};
use mc_connection_test_utils::MockBlockchainConnection;
use mc_fog_report_validation::MockFogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::PollingNetworkState;
use rand::rngs::StdRng;
use rocket::{
    http::{ContentType, Status},
    local::Client,
    post, routes,
};
use rocket_contrib::json::{Json, JsonValue};
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
    pub service: WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
}

// Note: the reason this is duplicated from wallet.rs is to be able to pass the
// TestWalletState, which handles Mock objects.
#[post("/wallet", format = "json", data = "<command>")]
fn test_wallet_api(
    state: rocket::State<TestWalletState>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse>, String> {
    let req: JsonRPCRequest = command.0.clone();

    let mut response = JsonRPCResponse {
        method: Some(command.0.method),
        result: None,
        error: None,
        jsonrpc: "2.0".to_string(),
        id: command.0.id,
    };

    match wallet_api_inner(
        &state.service,
        JsonCommandRequest::try_from(&req).map_err(|e| e)?,
    ) {
        Ok(command_response) => {
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            response.error = Some(rpc_error);
        }
    };

    Ok(Json(response))
}

pub fn test_rocket(rocket_config: rocket::Config, state: TestWalletState) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![test_wallet_api])
        .manage(state)
}

pub fn setup(
    mut rng: &mut StdRng,
    logger: Logger,
) -> (
    Client,
    LedgerDB,
    WalletDbTestContext,
    Arc<RwLock<PollingNetworkState<MockBlockchainConnection<LedgerDB>>>>,
) {
    let db_test_context = WalletDbTestContext::default();
    let wallet_db = db_test_context.get_db_instance(logger.clone());
    let known_recipients: Vec<PublicAddress> = Vec::new();
    let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
    let (peer_manager, network_state) =
        setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

    let service: WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> =
        WalletService::new(
            wallet_db,
            ledger_db.clone(),
            peer_manager,
            network_state.clone(),
            get_resolver_factory(&mut rng).unwrap(),
            None,
            false,
            logger,
        );

    let rocket_config: rocket::Config =
        rocket::Config::build(rocket::config::Environment::Development)
            .port(get_free_port())
            .unwrap();
    let rocket = test_rocket(rocket_config, TestWalletState { service });
    (
        Client::new(rocket).expect("valid rocket instance"),
        ledger_db,
        db_test_context,
        network_state,
    )
}

pub fn dispatch(client: &Client, request_body: JsonValue, logger: &Logger) -> JsonValue {
    log::info!(logger, "Attempting dispatch of\n{:?}\n", request_body,);
    let request_body = request_body.to_string();
    log::info!(logger, "Attempting dispatch of\n{}\n", request_body,);

    let mut res = client
        .post("/wallet")
        .header(ContentType::JSON)
        .body(request_body)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let response_body = res.body().unwrap().into_string().unwrap();
    log::info!(logger, "Got response\n{}\n", response_body);

    let res: JsonValue = serde_json::from_str(&response_body).unwrap();
    res
}

pub fn dispatch_expect_error(
    client: &Client,
    request_body: JsonValue,
    logger: &Logger,
    expected_err: String,
) {
    let mut res = client
        .post("/wallet")
        .header(ContentType::JSON)
        .body(request_body.to_string())
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let response_body = res.body().unwrap().into_string().unwrap();
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
        let res = dispatch(&client, body, &logger);
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
