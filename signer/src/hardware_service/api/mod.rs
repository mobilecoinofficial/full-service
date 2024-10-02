// Copyright (c) 2020-2024 MobileCoin Inc.

use crate::{
    hardware_service,
    hardware_service::api::{request::JsonCommandRequest, response::JsonCommandResponse},
};
use anyhow::Result;
use mc_common::logger::global_log;
use mc_full_service::{
    json_rpc::{
        json_rpc_request::JsonRPCRequest, json_rpc_response::{format_error, format_invalid_request_error, JsonRPCResponse},
        v2::models::tx_proposal::TxProposal as TxProposalJSON,
    },
    service::models::tx_proposal::UnsignedTxProposal,
};
use rocket::{get, post, serde::json::Json};
use anyhow::anyhow;

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
pub async fn hardware_service_api(
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

    match hardware_service_api_inner(request).await {
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

async fn hardware_service_api_inner(command: JsonCommandRequest) -> Result<JsonCommandResponse> {
    let response = match command {
        JsonCommandRequest::get_account {} => {
            let account_info = hardware_service::get_account().await?;
            let hardware_account_id = hardware_service::get_account_id(account_info.clone());
            JsonCommandResponse::get_account { account_id:hardware_account_id, account_info }
        },
        JsonCommandRequest::sync_txos {
            account_id,
            txos_unsynced,
        } => {
            let synced_txos = hardware_service::sync_txos(account_id.clone(), txos_unsynced).await?;
            JsonCommandResponse::sync_txos { account_id, synced_txos }
        },
        JsonCommandRequest::sign_tx {
            account_id,
            unsigned_tx_proposal,
        } => {
            let unsigned_tx_proposal: UnsignedTxProposal = mc_full_service::service::models::tx_proposal::UnsignedTxProposal::try_from(&unsigned_tx_proposal).map_err(|e| anyhow!(e))?;
            let signed_tx_proposal = hardware_service::sign_tx(account_id.clone(), unsigned_tx_proposal).await?;
            let tx_proposal: TxProposalJSON = mc_full_service::json_rpc::v2::models::tx_proposal::TxProposal::try_from(&signed_tx_proposal).map_err(|e|anyhow!(e))?;
            JsonCommandResponse::sign_tx { tx_proposal }
        },
    };

    Ok(response)
}
