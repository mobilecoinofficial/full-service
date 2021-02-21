// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet API for Full Service. Version 1.

use crate::service::{
    decorated_types::{JsonAccount, JsonCreateAccountResponse},
    wallet_impl::WalletService,
};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// FIXME: Update
pub fn help_str_v2() -> String {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequestV2::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    help_str
}

#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequestV2 {
    create_account {
        name: Option<String>,
        first_block: Option<String>,
    },
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
pub enum JsonCommandResponseV2 {
    create_account {
        entropy: String,
        account: JsonAccount,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRPCResponse {
    pub method: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRPCError>,
    pub jsonrpc: String,
    pub id: String,
}

impl From<JsonCommandResponseV2> for JsonRPCResponse {
    fn from(src: JsonCommandResponseV2) -> JsonRPCResponse {
        let json_response = json!(src);
        JsonRPCResponse {
            method: Some(json_response.get("method").unwrap().to_string()),
            result: Some(json_response.get("result").unwrap().clone()),
            error: None,
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum JsonRPCError {
    error {
        code: JsonRPCErrorCodes,
        message: String,
        data: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub enum JsonRPCErrorCodes {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    // ServerError(i32), // FIXME: WalletServiceError -> i32 between 32000 and 32099
}

// Helper method to format displaydoc errors in JSON RPC 2.0 format.
fn format_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> String {
    let data = json!({"server_error": format!("{:?}", e), "details": e.to_string()}).to_string();
    // FIXME: wrap in JsonRPCResponse
    let json_resp = JsonRPCError::error {
        code: JsonRPCErrorCodes::InternalError,
        message: "Internal error".to_string(),
        data,
    };
    json!(json_resp).to_string()
}

// The Wallet API inner method, which handles switching on the method enum.
//
// Note that this is structured this way so that the routes can be defined to
// take explicit Rocket state, and then pass the service to the inner method.
// This allows us to properly construct state with Mock Connection Objects in
// tests. This also allows us to version the overall API easily.
pub fn wallet_api_inner_v2<T, FPR>(
    service: &WalletService<T, FPR>,
    command: Json<JsonCommandRequestV2>,
) -> Result<Json<JsonRPCResponse>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    println!("\x1b[1;32m V2 baby!\x1b[0m");
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
