// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::JsonRPCResponse,
        v1::api::{
            request::help_str as help_str_v1,
            response::JsonCommandResponse as JsonCommandResponse_v1,
            wallet::generic_wallet_api as generic_wallet_api_v1,
        },
        v2::api::{
            request::help_str as help_str_v2,
            response::JsonCommandResponse as JsonCommandResponse_v2,
            wallet::generic_wallet_api as generic_wallet_api_v2,
        },
    },
    service::WalletService,
};
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_resolver::FogResolver;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_validator_connection::ValidatorConnection;
use rocket::{
    self, get, http::Status, outcome::Outcome, post, request::FromRequest, routes,
    serde::json::Json, Request, State,
};

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
    ApiKeyStateConfigInvalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKeyGuard {
    type Error = ApiKeyError;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> Outcome<Self, (rocket::http::Status, Self::Error), ()> {
        let client_key = req.headers().get_one(API_KEY_HEADER).unwrap_or_default();
        // let outcome = req.guard::<State<APIKeyState>>().await;
        let local_key = match req.guard::<&State<APIKeyState>>().await {
            Outcome::Success(api_key_state) => api_key_state.0.clone(),
            Outcome::Failure(_) => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    ApiKeyError::ApiKeyStateConfigInvalid,
                ))
            }
            Outcome::Forward(_) => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    ApiKeyError::ApiKeyStateConfigInvalid,
                ))
            }
        };

        if local_key == client_key {
            Outcome::Success(ApiKeyGuard {})
        } else {
            Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid))
        }
    }
}

#[get("/health")]
fn health() -> Result<(), ()> {
    Ok(())
}

#[get("/wallet")]
fn wallet_help_v1() -> Result<String, String> {
    Ok(help_str_v1())
}

/// The route for the Full Service Wallet API.
#[post("/wallet", format = "json", data = "<command>")]
fn consensus_backed_wallet_api_v1(
    _api_key_guard: ApiKeyGuard,
    state: &rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse_v1>>, String> {
    generic_wallet_api_v1(_api_key_guard, state, command)
}

#[post("/wallet", format = "json", data = "<command>")]
fn validator_backed_wallet_api_v1(
    _api_key_guard: ApiKeyGuard,
    state: &rocket::State<WalletState<ValidatorConnection, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse_v1>>, String> {
    generic_wallet_api_v1(_api_key_guard, state, command)
}

#[get("/wallet/v2")]
fn wallet_help_v2() -> Result<String, String> {
    Ok(help_str_v2())
}

/// The route for the Full Service Wallet API.
#[post("/wallet/v2", format = "json", data = "<command>")]
fn consensus_backed_wallet_api_v2(
    _api_key_guard: ApiKeyGuard,
    state: &rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse_v2>>, String> {
    generic_wallet_api_v2(_api_key_guard, state, command)
}

#[post("/wallet/v2", format = "json", data = "<command>")]
fn validator_backed_wallet_api_v2(
    _api_key_guard: ApiKeyGuard,
    state: &rocket::State<WalletState<ValidatorConnection, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse_v2>>, String> {
    generic_wallet_api_v2(_api_key_guard, state, command)
}

/// Returns an instance of a Rocket server.
pub fn consensus_backed_rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket<rocket::Build> {
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![
                consensus_backed_wallet_api_v1,
                consensus_backed_wallet_api_v2,
                wallet_help_v1,
                wallet_help_v2,
                health
            ],
        )
        .manage(state)
}

pub fn validator_backed_rocket(
    rocket_config: rocket::Config,
    state: WalletState<ValidatorConnection, FogResolver>,
) -> rocket::Rocket<rocket::Build> {
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![
                validator_backed_wallet_api_v1,
                validator_backed_wallet_api_v2,
                wallet_help_v1,
                wallet_help_v2,
                health
            ],
        )
        .manage(state)
}
