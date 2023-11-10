// Copyright (c) 2020-2023 MobileCoin Inc.

use crate::{
    service,
    service::api::{request::JsonCommandRequest, response::JsonCommandResponse},
};
use anyhow::{anyhow, Result};
use mc_common::logger::global_log;
use mc_full_service::json_rpc::{
    json_rpc_request::JsonRPCRequest,
    json_rpc_response::{format_error, format_invalid_request_error, JsonRPCResponse},
};
use rocket::{get, post, serde::json::Json};

pub mod request;
pub mod response;

#[get("/version")]
pub fn version() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION").to_string(),
        "commit": env!("VERGEN_GIT_SHA"),
        "build_date": env!("VERGEN_BUILD_DATE"),
        "build_time": env!("VERGEN_BUILD_TIMESTAMP"),
        "target": env!("VERGEN_CARGO_TARGET_TRIPLE"),
    })
}

/// The route for the Signer Service API.
#[post("/api", format = "json", data = "<command>")]
pub fn signer_service_api(
    command: Json<JsonRPCRequest>,
) -> Json<JsonRPCResponse<JsonCommandResponse>> {
    let req: JsonRPCRequest = command.0.clone();

    let mut response: JsonRPCResponse<JsonCommandResponse> = JsonRPCResponse {
        method: Some(command.0.method),
        result: None,
        error: None,
        jsonrpc: "2.0".to_string(),
        id: command.0.id,
    };

    let request = match JsonCommandRequest::try_from(&req) {
        Ok(request) => request,
        Err(error) => {
            response.error = Some(format_invalid_request_error(error));
            return Json(response);
        }
    };

    match signer_service_api_inner(request) {
        Ok(command_response) => {
            global_log::info!("Command executed successfully");
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            global_log::info!("Command failed with error: {:?}", rpc_error);
            response.error = Some(format_error(rpc_error));
        }
    };

    Json(response)
}

fn signer_service_api_inner(command: JsonCommandRequest) -> Result<JsonCommandResponse> {
    let response = match command {
        JsonCommandRequest::create_account {} => {
            let (mnemonic, account_info) = service::create_account();
            JsonCommandResponse::create_account {
                mnemonic: mnemonic.to_string(),
                account_info,
            }
        }
        JsonCommandRequest::get_account { mnemonic } => {
            let account_info = service::get_account(&mnemonic)?;
            JsonCommandResponse::get_account { account_info }
        }
        JsonCommandRequest::sign_tx {
            mnemonic,
            unsigned_tx_proposal,
        } => {
            let signed_tx = service::sign_tx(
                &mnemonic,
                (&unsigned_tx_proposal)
                    .try_into()
                    .map_err(|e: String| anyhow!(e))?,
            )?;
            JsonCommandResponse::sign_tx {
                tx_proposal: (&signed_tx).try_into().map_err(|e: String| anyhow!(e))?,
            }
        }
        JsonCommandRequest::sync_txos {
            mnemonic,
            txos_unsynced,
        } => {
            let txos_synced = service::sync_txos(&mnemonic, txos_unsynced)?;
            JsonCommandResponse::sync_txos { txos_synced }
        }
    };

    Ok(response)
}
