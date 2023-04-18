use mc_common::logger::global_log;
use mc_full_service::json_rpc::{
    json_rpc_request::JsonRPCRequest,
    json_rpc_response::{format_invalid_request_error, JsonRPCError, JsonRPCResponse},
};

use rocket::{post, serde::json::Json};

use crate::{
    service,
    service::api::{request::JsonCommandRequest, response::JsonCommandResponse},
};

pub mod request;
pub mod response;

/// The route for the Transaction Signer Service API.
#[post("/api", format = "json", data = "<command>")]
pub fn signer_service_api(
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse>>, String> {
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
            return Ok(Json(response));
        }
    };

    match signer_service_api_inner(request) {
        Ok(command_response) => {
            global_log::info!("Command executed successfully");
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            global_log::info!("Command failed with error: {:?}", rpc_error);
            response.error = Some(rpc_error);
        }
    };

    Ok(Json(response))
}

fn signer_service_api_inner(
    command: JsonCommandRequest,
) -> Result<JsonCommandResponse, JsonRPCError> {
    global_log::info!("Running command {:?}", command);

    let response = match command {
        JsonCommandRequest::create_account {} => {
            let (mnemonic, account_info) = service::create_account();
            JsonCommandResponse::create_account {
                mnemonic: mnemonic.to_string(),
                account_info: account_info,
            }
        }
        JsonCommandRequest::get_account { mnemonic } => {
            let account_info = service::get_account(&mnemonic);
            JsonCommandResponse::get_account { info: account_info }
        }
        JsonCommandRequest::sign_tx {
            mnemonic,
            unsigned_tx,
        } => {
            let signed_tx = service::sign_tx(&mnemonic, unsigned_tx.try_into().unwrap());
            JsonCommandResponse::sign_tx {
                tx_proposal: (&signed_tx).try_into().unwrap(),
            }
        }
        JsonCommandRequest::sync_txos { mnemonic, txos } => {
            let txos_synced = service::sync_txos(&mnemonic, txos);
            JsonCommandResponse::sync_txos { txos_synced }
        }
    };

    Ok(response)
}
