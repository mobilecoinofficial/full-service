// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON-RPC Responses from the Wallet API.
//!
//! API v2

use crate::{
    json_rpc::{
        account::Account,
        account_secrets::AccountSecrets,
        address::Address,
        balance::Balance,
        block::{Block, BlockContents},
        confirmation_number::Confirmation,
        gift_code::GiftCode,
        json_rpc_request::JsonRPCRequest,
        network_status::NetworkStatus,
        receiver_receipt::ReceiverReceipt,
        transaction_log::TransactionLog,
        tx_proposal::TxProposal as TxProposalJSON,
        txo::Txo,
        wallet_status::WalletStatus,
    },
    service::{gift_code::GiftCodeStatus, receipt::ReceiptTransactionStatus},
    util::b58::PrintableWrapperType,
};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::{BTreeMap, HashMap};
use strum::Display;

use crate::{fog_resolver::FullServiceFogResolver, unsigned_tx::UnsignedTx};

pub trait JsonCommandResponse {}

/// A JSON RPC 2.0 Response.
#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRPCResponse<Response>
where
    Response: JsonCommandResponse,
{
    /// The method which was invoked on the server.
    ///
    /// Optional because JSON RPC does not require returning the method invoked,
    /// as that can be determined by the id. We return it as a convenience.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// The result of invoking the method on the server.
    ///
    /// Optional: if error occurs, result is not returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Response>,

    /// The error that occurred when invoking the method on the server.
    ///
    /// Optional: if method was successful, error is not returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRPCError>,

    /// The JSON RPC version. Should always be 2.0.
    pub jsonrpc: String,

    /// The id of the Request object to which this response corresponds.
    pub id: serde_json::Value,
}

/// A JSON RPC Error.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum JsonRPCError {
    error {
        /// The error code associated with this error.
        code: i32,

        /// A string providing a short description of the error.
        message: String,

        /// Additional information about the error.
        data: serde_json::Value,
    },
}

/// JSON RPC Error codes.
#[derive(Deserialize, Serialize, Debug, Display)]
pub enum JsonRPCErrorCodes {
    /// Parse error.
    ParseError = -32700,

    /// Invalid request.
    InvalidRequest = -32600,

    /// Method not found.
    MethodNotFound = -32601,

    /// Invalid params.
    InvalidParams = -32602,

    /// Internal Error.
    InternalError = -32603,
    /* Server error.
     * ServerError(i32), // FIXME: WalletServiceError -> i32 between 32000 and 32099 */
}

/// Helper method to format displaydoc errors in JSON RPC 2.0 format.
pub fn format_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> JsonRPCError {
    let data: serde_json::Value =
        json!({"server_error": format!("{:?}", e), "details": e.to_string()}).into();
    JsonRPCError::error {
        code: JsonRPCErrorCodes::InternalError as i32,
        message: JsonRPCErrorCodes::InternalError.to_string(),
        data,
    }
}

/// Helper method to format displaydoc invalid request errors in JSON RPC 2.0
/// format.
pub fn format_invalid_request_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> JsonRPCError {
    let data: serde_json::Value =
        json!({"server_error": format!("{:?}", e), "details": e.to_string()}).into();
    JsonRPCError::error {
        code: JsonRPCErrorCodes::InvalidRequest as i32,
        message: JsonRPCErrorCodes::InvalidRequest.to_string(),
        data,
    }
}
