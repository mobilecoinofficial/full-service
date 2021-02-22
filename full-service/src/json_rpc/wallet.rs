// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    db, json_rpc,
    json_rpc::{
        api_v1::wallet_api::{help_str_v1, wallet_api_inner_v1, JsonCommandRequestV1},
        json_rpc_request::{help_str_v2, JsonCommandRequest, JsonCommandRequestV2},
        json_rpc_response::{
            format_error, JsonCommandResponse, JsonCommandResponseV2, JsonRPCResponse,
        },
    },
    service::{account::AccountService, WalletService},
};
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde_json::Map;
use std::{convert::TryFrom, iter::FromIterator};

/// State managed by rocket.
pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    /// The Wallet Service implementation.
    pub service: WalletService<T, FPR>,
}

/// The route for the Full Service Wallet API.
#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
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

/// The Wallet API inner method, which handles switching on the method enum.
///
/// Note that this is structured this way so that the routes can be defined to
/// take explicit Rocket state, and then pass the service to the inner method.
/// This allows us to properly construct state with Mock Connection Objects in
/// tests. This also allows us to version the overall API easily.
pub fn wallet_api_inner_v2<T, FPR>(
    service: &WalletService<T, FPR>,
    command: Json<JsonCommandRequestV2>,
) -> Result<Json<JsonRPCResponse>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    let result: JsonCommandResponseV2 = match command.0 {
        JsonCommandRequestV2::create_account { name, first_block } => {
            let fb = first_block
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let account: db::models::Account =
                service.create_account(name, fb).map_err(format_error)?;

            JsonCommandResponseV2::create_account {
                account: json_rpc::account::Account::try_from(&account)
                    .map_err(|e| format!("Could not get RPC Account from DB Account {:?}", e))?,
            }
        }
        JsonCommandRequestV2::import_account {
            entropy,
            name,
            first_block,
        } => {
            let fb = first_block
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            JsonCommandResponseV2::import_account {
                account: json_rpc::account::Account::try_from(
                    &service
                        .import_account(entropy, name, fb)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::get_all_accounts => {
            let accounts = service.list_accounts().map_err(format_error)?;
            let json_accounts: Vec<(String, serde_json::Value)> = accounts
                .iter()
                .map(|a| {
                    json_rpc::account::Account::try_from(a).and_then(|v| {
                        serde_json::to_value(v)
                            .and_then(|v| Ok((a.account_id_hex.clone(), v)))
                            .map_err(format_error)
                    })
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, String>>()?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(json_accounts);
            JsonCommandResponseV2::get_all_accounts {
                account_ids: accounts.iter().map(|a| a.account_id_hex.clone()).collect(),
                account_map,
            }
        }
    };
    let response = Json(JsonRPCResponse::from(result));
    Ok(response)
}

#[get("/wallet")]
fn wallet_help_v2() -> Result<String, String> {
    Ok(help_str_v2())
}

#[get("/wallet/v1")]
fn wallet_help_v1() -> Result<String, String> {
    Ok(help_str_v1())
}

/// Returns an instance of a Rocker server.
pub fn rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api, wallet_help_v2, wallet_help_v1])
        .manage(state)
}
