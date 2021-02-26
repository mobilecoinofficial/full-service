// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON-RPC Responses from the Wallet API.
//!
//! API v2

use crate::json_rpc::{account::Account, balance::Balance, wallet_status::WalletStatus};

use serde::{Deserialize, Serialize};
use serde_json::Map;

/// A JSON RPC Response.
#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRPCResponse {
    /// The method which was invoked on the server.
    ///
    /// Optional because JSON RPC does not require returning the method invoked,
    /// as that can be determined by the id. We return it as a convenience.
    pub method: Option<String>,

    /// The result of invoking the method on the server.
    ///
    /// Optional: if error occurs, result is not returned.
    pub result: Option<serde_json::Value>,

    /// The error that occurred when invoking the method on the server.
    ///
    /// Optional: if method was successful, error is not returned.
    pub error: Option<JsonRPCError>,

    /// The JSON RPC version. Should always be 2.0.
    pub jsonrpc: String,

    /// The id of the Request object to which this response corresponds.
    pub id: u32,
}

// FIXME: unwraps -> TryFrom
impl From<JsonCommandResponseV2> for JsonRPCResponse {
    fn from(src: JsonCommandResponseV2) -> JsonRPCResponse {
        let json_response = json!(src);
        JsonRPCResponse {
            method: Some(
                json_response
                    .get("method")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            result: Some(json_response.get("result").unwrap().clone()),
            error: None, // FIXME: currently returning "error: null" but should be omitted
            jsonrpc: "2.0".to_string(),
            id: 1, // FIXME: must be the same as the request that was passed in
        }
    }
}

/// JSON RPC 2.0 Response.
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types)]
pub struct JsonCommandResponse {
    /// The method which was invoked on the server.
    ///
    /// Optional because JSON RPC does not require returning the method invoked,
    /// as that can be determined by the id. We return it as a convenience.
    pub method: Option<String>,

    /// The result of invoking the method on the server.
    ///
    /// Optional: if error occurs, result is not returned.
    pub result: Option<serde_json::Value>,

    /// The error that occurred when invoking the method on the server.
    ///
    /// Optional: if method was successful, error is not returned.
    pub error: Option<JsonRPCError>,

    /// The JSON RPC version. Should always be 2.0.
    pub jsonrpc: Option<String>,

    /// The id of the Request object to which this response corresponds.
    pub id: Option<u32>,

    /// The Full Service Wallet API version.
    ///
    /// Optional: If omitted, assumes V1.
    pub api_version: Option<String>,
}

/// A JSON RPC Error.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum JsonRPCError {
    error {
        /// The error code associated with this error.
        code: JsonRPCErrorCodes,

        /// A string providing a short description of the error.
        message: String,

        /// Additional information about the error.
        data: String, // FIXME: could be json structured value.
    },
}

/// JSON RPC Error codes.
#[derive(Deserialize, Serialize, Debug)]
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
pub fn format_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> String {
    let data = json!({"server_error": format!("{:?}", e), "details": e.to_string()}).to_string();
    // FIXME: wrap in JsonRPCResponse
    let json_resp = JsonRPCError::error {
        code: JsonRPCErrorCodes::InternalError,
        message: "Internal error".to_string(),
        data,
    };
    json!(json_resp).to_string()
}

/// Responses from the Full Service Wallet.
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
pub enum JsonCommandResponseV2 {
    create_account {
        account: Account,
    },
    import_account {
        account: Account,
    },
    get_all_accounts {
        account_ids: Vec<String>,
        account_map: Map<String, serde_json::Value>,
    },
    get_account {
        account: Account,
    },
    update_account_name {
        account: Account,
    },
    delete_account {
        account: Account,
    },
    get_balance_for_account {
        balance: Balance,
    },
    /*
    get_balance_for_subaddress {
        balance: Balance,
    },*/
    /* get_txos_for_subaddress {

    }
     */
    get_wallet_status {
        wallet_status: WalletStatus,
    },
    get_account_status {
        account: Account,
        balance: Balance,
    },
    /*
    get_all_txos_by_account {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_txo {
        txo: JsonTxo,
    },

    create_address {
        address: JsonAddress,
    },
    get_all_addresses_by_account {
        address_ids: Vec<String>,
        address_map: Map<String, serde_json::Value>,
    },
    send_transaction {
        transaction: JsonSubmitResponse,
    },
    build_transaction {
        tx_proposal: StringifiedJsonTxProposal,
    },
    submit_transaction {
        transaction: JsonSubmitResponse,
    },
    get_all_transactions_by_account {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_transaction {
        transaction: JsonTransactionLog,
    },
    get_transaction_object {
        transaction: JsonTx,
    },
    get_txo_object {
        txo: JsonTxOut,
    },
    get_block_object {
        block: JsonBlock,
        block_contents: JsonBlockContents,
    },
    get_proofs {
        proofs: Vec<JsonProof>,
    },
    verify_proof {
        verified: bool,
    },*/
}
