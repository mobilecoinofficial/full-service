// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{
    json_rpc::{
        api_v1::wallet_api::{wallet_api_inner_v1, JsonCommandRequestV1},
        json_rpc_request::{JsonCommandRequest, JsonCommandRequestV2},
        json_rpc_response::JsonCommandResponse,
        wallet::wallet_api_inner_v2,
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
pub fn test_wallet_api(
    state: rocket::State<TestWalletState>,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, String> {
    let req: JsonCommandRequest = command.0.clone();
    if let Some(version) = command.0.api_version.clone() {
        wallet_api_inner_v2(
            &state.service,
            Json(JsonCommandRequestV2::try_from(&req).map_err(|e| e)?),
        )
        .and_then(|res| {
            Ok(Json(JsonCommandResponse {
                method: res.0.method,
                result: res.0.result,
                error: res.0.error,
                jsonrpc: Some("2.0".to_string()),
                id: command.0.id,
                api_version: Some(version),
            }))
        })
    } else {
        wallet_api_inner_v1(
            &state.service,
            Json(JsonCommandRequestV1::try_from(&req).map_err(|e| e)?),
        )
        .and_then(|res| {
            let json_response: serde_json::Value = serde_json::json!(res.0);
            Ok(Json(JsonCommandResponse {
                method: Some(json_response.get("method").unwrap().to_string()),
                result: Some(json_response.get("result").unwrap().clone()),
                error: None,
                jsonrpc: None,
                id: None,
                api_version: None,
            }))
        })
    }
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
        std::thread::sleep(Duration::from_secs(1));

        // Have to manually call poll() on network state to get it to update for these
        // tests
        network_state.write().unwrap().poll();

        // Check that syncing is working
        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
            "method": "get_wallet_status",
        });
        let res = dispatch(&client, body, &logger);
        let status = res.get("result").unwrap().get("wallet_status").unwrap();

        let is_synced_all = status.get("is_synced_all").unwrap().as_bool().unwrap();
        if is_synced_all {
            let local_height = status
                .get("local_block_count")
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            assert_eq!(local_height, ledger_db.num_blocks().unwrap());
            assert!(
                status
                    .get("network_block_count")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
                    >= local_height
            );
            break;
        }

        count += 1;
        if count > 10 {
            panic!("Service did not sync after 10 iterations");
        }
    }
}
