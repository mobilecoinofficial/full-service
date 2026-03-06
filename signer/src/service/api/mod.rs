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
        JsonCommandRequest::get_account {
            mnemonic,
            bip39_entropy,
        } => match (mnemonic, bip39_entropy) {
            (Some(mnemonic), None) => {
                let account_info = service::get_account_by_mnemonic(&mnemonic)?;
                JsonCommandResponse::get_account { account_info }
            }
            (None, Some(bip39_entropy)) => {
                let account_info = service::get_account_by_bip39_entropy(&bip39_entropy)?;
                JsonCommandResponse::get_account { account_info }
            }
            (None, None) => {
                return Err(anyhow!("Either mnemonic or bip39_entropy must be provided"));
            }
            _ => {
                return Err(anyhow!(
                    "Only one of mnemonic or bip39_entropy can be provided"
                ))
            }
        },
        JsonCommandRequest::sign_tx {
            mnemonic,
            bip39_entropy,
            unsigned_tx_proposal,
        } => match (mnemonic, bip39_entropy) {
            (Some(mnemonic), None) => {
                let signed_tx = service::sign_tx_with_mnemonic(
                    &mnemonic,
                    (&unsigned_tx_proposal)
                        .try_into()
                        .map_err(|e: String| anyhow!(e))?,
                )?;
                JsonCommandResponse::sign_tx {
                    tx_proposal: (&signed_tx).try_into().map_err(|e: String| anyhow!(e))?,
                }
            }
            (None, Some(bip39_entropy)) => {
                let signed_tx = service::sign_tx_with_bip39_entropy(
                    &bip39_entropy,
                    (&unsigned_tx_proposal)
                        .try_into()
                        .map_err(|e: String| anyhow!(e))?,
                )?;
                JsonCommandResponse::sign_tx {
                    tx_proposal: (&signed_tx).try_into().map_err(|e: String| anyhow!(e))?,
                }
            }
            (None, None) => {
                return Err(anyhow!("Either mnemonic or bip39_entropy must be provided"));
            }
            _ => {
                return Err(anyhow!(
                    "Only one of mnemonic or bip39_entropy can be provided"
                ))
            }
        },
        JsonCommandRequest::sign_tx_blueprint {
            mnemonic,
            bip39_entropy,
            tx_blueprint_proposal,
        } => match (mnemonic, bip39_entropy) {
            (Some(mnemonic), None) => {
                let signed_tx = service::sign_tx_blueprint_with_mnemonic(
                    &mnemonic,
                    (&tx_blueprint_proposal)
                        .try_into()
                        .map_err(|e: String| anyhow!(e))?,
                )?;
                JsonCommandResponse::sign_tx {
                    tx_proposal: (&signed_tx).try_into().map_err(|e: String| anyhow!(e))?,
                }
            }
            (None, Some(bip39_entropy)) => {
                let signed_tx = service::sign_tx_blueprint_with_bip39_entropy(
                    &bip39_entropy,
                    (&tx_blueprint_proposal)
                        .try_into()
                        .map_err(|e: String| anyhow!(e))?,
                )?;
                JsonCommandResponse::sign_tx {
                    tx_proposal: (&signed_tx).try_into().map_err(|e: String| anyhow!(e))?,
                }
            }
            (None, None) => {
                return Err(anyhow!("Either mnemonic or bip39_entropy must be provided"));
            }
            _ => {
                return Err(anyhow!(
                    "Only one of mnemonic or bip39_entropy can be provided"
                ))
            }
        },
        JsonCommandRequest::sync_txos {
            mnemonic,
            bip39_entropy,
            txos_unsynced,
        } => match (mnemonic, bip39_entropy) {
            (Some(mnemonic), None) => {
                let txos_synced = service::sync_txos_by_mnemonic(&mnemonic, txos_unsynced)?;
                JsonCommandResponse::sync_txos { txos_synced }
            }
            (None, Some(bip39_entropy)) => {
                let txos_synced =
                    service::sync_txos_by_bip39_entropy(&bip39_entropy, txos_unsynced)?;
                JsonCommandResponse::sync_txos { txos_synced }
            }
            (None, None) => {
                return Err(anyhow!("Either mnemonic or bip39_entropy must be provided"));
            }
            _ => {
                return Err(anyhow!(
                    "Only one of mnemonic or bip39_entropy can be provided"
                ))
            }
        },
    };

    Ok(response)
}
