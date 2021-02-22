// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    json_rpc::{
        api_v1::{
            decorated_types::JsonCreateAccountResponse,
            wallet_api::{help_str_v1, wallet_api_inner_v1, JsonCommandRequestV1},
        },
        json_rpc_request::{help_str_v2, JsonCommandRequestV2},
        json_rpc_response::{format_error, JsonCommandResponseV2, JsonRPCError, JsonRPCResponse},
    },
    service::WalletService,
};
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// State managed by rocket.
pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    /// The Wallet Service implementation.
    pub service: WalletService<T, FPR>,
}

/// JSON RPC 2.0 Request.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct JsonCommandRequest {
    /// The method to be invoked on the server.
    pub method: String,

    /// The parameters to be provided to the method.
    ///
    /// Optional, as some methods do not take parameters.
    pub params: Option<serde_json::Value>,

    /// The JSON RPC Version (Should always be 2.0)
    ///
    /// Optional for backwards compatibility because the previous version of
    /// this API (v1) did not require the jsonrpc parameter.
    pub jsonrpc: Option<String>,

    /// The ID to be associated with this request.
    ///
    /// Optional because a "notify" method does not need to correlate an ID on
    /// the response.
    pub id: Option<String>,

    /// The Full Service Wallet API version.
    ///
    /// Optional: If omitted, assumes V1.
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

/// JSON RPC 2.0 Response.
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types)]
pub struct JsonCommandResponse {
    /// The method which was invoked on the server.
    ///
    /// Optional because JSON RPC does not require returning the method invoked,
    /// as that can be determined by the id. We return it as a convenience.
    pub method: Option<String>,

    /// The result of invoking the method on the server.
    ///
    /// Optional: if error occurs, result is not returned.
    pub result: Option<serde_json::Value>,

    /// The error that occurred when invoking the method on the server.
    ///
    /// Optional: if method was successful, error is not returned.
    pub error: Option<JsonRPCError>,

    /// The JSON RPC version. Should always be 2.0.
    pub jsonrpc: Option<String>,

    /// The id of the Request object to which this response corresponds.
    pub id: Option<String>,

    /// The Full Service Wallet API version.
    ///
    /// Optional: If omitted, assumes V1.
    pub api_version: Option<String>,
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

            let result: JsonCreateAccountResponse =
                service.create_account(name, fb).map_err(format_error)?;

            JsonCommandResponseV2::create_account {
                entropy: result.entropy,
                account: result.account,
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
