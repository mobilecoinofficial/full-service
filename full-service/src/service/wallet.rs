// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API

use crate::service::{
    api_v1::{help_str_v1, wallet_api_inner_v1, JsonCommandRequestV1},
    api_v2::{help_str_v2, wallet_api_inner_v2, JsonCommandRequestV2, JsonRPCError},
    wallet_impl::WalletService,
};
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    pub service: WalletService<T, FPR>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct JsonCommandRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub jsonrpc: Option<String>,
    pub id: Option<String>,
    pub api_version: Option<String>,
}

impl TryFrom<&JsonCommandRequest> for JsonCommandRequestV1 {
    type Error = String;

    fn try_from(src: &JsonCommandRequest) -> Result<JsonCommandRequestV1, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

impl TryFrom<&JsonCommandRequest> for JsonCommandRequestV2 {
    type Error = String;

    fn try_from(src: &JsonCommandRequest) -> Result<JsonCommandRequestV2, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types)]
pub struct JsonCommandResponse {
    pub method: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRPCError>,
    pub jsonrpc: Option<String>,
    pub id: Option<String>,
    pub api_version: Option<String>,
}

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

#[get("/wallet")]
fn wallet_help_v2() -> Result<String, String> {
    Ok(help_str_v2())
}

#[get("/wallet/v1")]
fn wallet_help_v1() -> Result<String, String> {
    Ok(help_str_v1())
}

pub fn rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api, wallet_help_v2, wallet_help_v1])
        .manage(state)
}
