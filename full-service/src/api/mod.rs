// Copyright (c) 2020-2021 MobileCoin Inc.

//! MobileCoin FullService Wallet API Layer.

pub mod decorated_types;
mod json_rpc_request;
mod json_rpc_response;
mod wallet_api;

pub use wallet_api::{rocket, WalletApiState};
