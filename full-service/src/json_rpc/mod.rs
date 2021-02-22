// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON RPC 2.0 API specification for the Full Service wallet.

#![warn(missing_docs)]

mod account;
mod account_key;
#[cfg(any(test, feature = "test_utils"))]
pub mod api_test_utils;
pub mod api_v1;
mod balance;
pub mod json_rpc_request;
pub mod json_rpc_response;
pub mod wallet;
mod wallet_status;
