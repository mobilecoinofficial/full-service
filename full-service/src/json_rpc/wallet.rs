// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    db::{
        self,
        account::AccountID,
        transaction_log::TransactionID,
        txo::{TxoID, TxoStatus},
    },
    json_rpc::{
        self,
        account_secrets::AccountSecrets,
        address::Address,
        balance::Balance,
        block::{Block, BlockContents},
        confirmation_number::Confirmation,
        gift_code::GiftCode,
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::{
            format_error, format_invalid_request_error, JsonRPCError, JsonRPCResponse,
        },
        network_status::NetworkStatus,
        receiver_receipt::ReceiverReceipt,
        tx_proposal::TxProposal as TxProposalJSON,
        txo::Txo,
        v2::{
            api_request::{help_str as help_str_v2, JsonCommandRequest as JsonCommandRequestV2},
            api_response::JsonCommandResponse as JsonCommandResponseV2,
            wallet::generic_wallet_api as generic_wallet_api_v2,
        },
        wallet_status::WalletStatus,
    },
    service,
    service::{
        account::AccountService,
        address::AddressService,
        balance::BalanceService,
        confirmation_number::ConfirmationService,
        gift_code::{EncodedGiftCode, GiftCodeService},
        ledger::LedgerService,
        models::tx_proposal::TxProposal,
        payment_request::PaymentRequestService,
        receipt::ReceiptService,
        transaction::TransactionService,
        transaction_log::TransactionLogService,
        txo::TxoService,
        WalletService,
    },
    util::b58::{
        b58_decode_payment_request, b58_encode_public_address, b58_printable_wrapper_type,
        PrintableWrapperType,
    },
};
use mc_common::logger::global_log;
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use mc_validator_connection::ValidatorConnection;
use rocket::{
    self, get, http::Status, outcome::Outcome, post, request::FromRequest, routes, Request, State,
};
use rocket_contrib::json::Json;
use serde_json::Map;
use std::{collections::HashMap, convert::TryFrom, iter::FromIterator, str::FromStr};

/// State managed by rocket.
pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    /// The Wallet Service implementation.
    pub service: WalletService<T, FPR>,
}

pub const API_KEY_HEADER: &str = "X-API-KEY";

pub struct APIKeyState(pub String);

/// Ensures check for a pre-shared symmetric API key for the JsonRPC loop on the
/// Mobilecoin wallet.
pub struct ApiKeyGuard {}

#[derive(Debug)]
pub enum ApiKeyError {
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKeyGuard {
    type Error = ApiKeyError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> Outcome<Self, (rocket::http::Status, Self::Error), ()> {
        let client_key = req.headers().get_one(API_KEY_HEADER).unwrap_or_default();
        let local_key = &req
            .guard::<State<APIKeyState>>()
            .expect("api key state config is bad. see main.rs")
            .0;
        if local_key == client_key {
            Outcome::Success(ApiKeyGuard {})
        } else {
            Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid))
        }
    }
}

// fn generic_wallet_api<T, FPR>(
//     _api_key_guard: ApiKeyGuard,
//     state: rocket::State<WalletState<T, FPR>>,
//     command: Json<JsonRPCRequest>,
// ) -> Result<Json<JsonRPCResponse<JsonCommandResponseV1>>, String>
// where
//     T: BlockchainConnection + UserTxConnection + 'static,
//     FPR: FogPubkeyResolver + Send + Sync + 'static,
// {
//     let req: JsonRPCRequest = command.0.clone();

//     let mut response: JsonRPCResponse<JsonCommandResponseV1> =
// JsonRPCResponse {         method: Some(command.0.method),
//         result: None,
//         error: None,
//         jsonrpc: "2.0".to_string(),
//         id: command.0.id,
//     };

//     let request = match JsonCommandRequest::try_from(&req) {
//         Ok(request) => request,
//         Err(error) => {
//             response.error = Some(format_invalid_request_error(error));
//             return Ok(Json(response));
//         }
//     };

//     match wallet_api_inner(&state.service, request) {
//         Ok(command_response) => {
//             response.result = Some(command_response);
//         }
//         Err(rpc_error) => {
//             response.error = Some(rpc_error);
//         }
//     };

//     Ok(Json(response))
// }

// /// The route for the Full Service Wallet API.
// #[post("/wallet", format = "json", data = "<command>")]
// pub fn consensus_backed_wallet_api(
//     _api_key_guard: ApiKeyGuard,
//     state:
// rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>,
// FogResolver>>,     command: Json<JsonRPCRequest>,
// ) -> Result<Json<JsonRPCResponse<JsonCommandResponseV1>>, String> {
//     generic_wallet_api(_api_key_guard, state, command)
// }

// #[post("/wallet", format = "json", data = "<command>")]
// pub fn validator_backed_wallet_api(
//     _api_key_guard: ApiKeyGuard,
//     state: rocket::State<WalletState<ValidatorConnection, FogResolver>>,
//     command: Json<JsonRPCRequest>,
// ) -> Result<Json<JsonRPCResponse<JsonCommandResponseV1>>, String> {
//     generic_wallet_api(_api_key_guard, state, command)
// }

/// The route for the Full Service Wallet API.
#[post("/wallet/v2", format = "json", data = "<command>")]
pub fn consensus_backed_wallet_api_v2(
    _api_key_guard: ApiKeyGuard,
    state: rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponseV2>>, String> {
    generic_wallet_api_v2(_api_key_guard, state, command)
}

#[post("/wallet/v2", format = "json", data = "<command>")]
pub fn validator_backed_wallet_api_v2(
    _api_key_guard: ApiKeyGuard,
    state: rocket::State<WalletState<ValidatorConnection, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponseV2>>, String> {
    generic_wallet_api_v2(_api_key_guard, state, command)
}

#[get("/wallet/v2")]
fn wallet_help() -> Result<String, String> {
    Ok(help_str_v2())
}

#[get("/health")]
fn health() -> Result<(), ()> {
    Ok(())
}

/// Returns an instance of a Rocket server.
pub fn consensus_backed_rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![
                // consensus_backed_wallet_api,
                consensus_backed_wallet_api_v2,
                wallet_help,
                health
            ],
        )
        .manage(state)
}

pub fn validator_backed_rocket(
    rocket_config: rocket::Config,
    state: WalletState<ValidatorConnection, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![
                // validator_backed_wallet_api,
                validator_backed_wallet_api_v2,
                wallet_help,
                health
            ],
        )
        .manage(state)
}
