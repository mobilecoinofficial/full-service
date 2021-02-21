// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::service::{
    api_v1::{help_str_v1, wallet_api_inner_v1, JsonCommandRequestV1, JsonCommandResponseV1},
    wallet_impl::WalletService,
};
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;

pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    pub service: WalletService<T, FPR>,
}

#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
    command: Json<JsonCommandRequestV1>,
) -> Result<Json<JsonCommandResponseV1>, String> {
    wallet_api_inner_v1(&state.service, command)
}

#[get("/wallet")]
fn wallet_help() -> Result<String, String> {
    Ok(help_str_v1())
}

pub fn rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api, wallet_help])
        .manage(state)
}
