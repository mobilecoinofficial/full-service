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
        gift_code::GiftCode,
        proof::Proof,
        receiver_receipt::ReceiverReceipt,
        transaction_log::TransactionLog,
        tx_proposal::TxProposal,
        txo::Txo,
        wallet_status::WalletStatus,
    },
    service::{gift_code::GiftCodeStatus, receipt::ReceiptTransactionStatus},
};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use strum::AsStaticRef;
use strum_macros::AsStaticStr;

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
#[derive(Deserialize, Serialize, Debug, AsStaticStr)]
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
    let data: serde_json::Value =
        json!({"server_error": format!("{:?}", e), "details": e.to_string()}).into();
    // FIXME: wrap in JsonRPCResponse
    let json_resp = JsonRPCError::error {
        code: JsonRPCErrorCodes::InternalError as i32,
        message: JsonRPCErrorCodes::InternalError.as_static().to_string(),
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
    export_account_secrets {
        account_secrets: AccountSecrets,
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
    update_account_fog_details {
        account: Account,
    },
    remove_account {
        removed: bool,
    },
    get_balance_for_account {
        balance: Balance,
    },
    build_and_submit_transaction {
        transaction_log: TransactionLog,
    },
    build_transaction {
        tx_proposal: TxProposal,
    },
    submit_transaction {
        transaction_log: Option<TransactionLog>,
    },
    get_all_transaction_logs_for_account {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_transaction_log {
        transaction_log: TransactionLog,
    },
    get_all_transaction_logs_for_block {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_all_transaction_logs_ordered_by_block {
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_wallet_status {
        wallet_status: WalletStatus,
    },
    get_account_status {
        account: Account,
        balance: Balance,
    },
    assign_address_for_account {
        address: Address,
    },
    get_all_addresses_for_account {
        public_addresses: Vec<String>,
        address_map: Map<String, serde_json::Value>,
    },
    verify_address {
        verified: bool,
    },
    get_balance_for_address {
        balance: Balance,
    },
    get_all_txos_for_account {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_txo {
        txo: Txo,
    },
    get_all_txos_for_address {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_proofs {
        proofs: Vec<Proof>,
    },
    verify_proof {
        verified: bool,
    },
    get_mc_protocol_transaction {
        transaction: JsonTx,
    },
    get_mc_protocol_txo {
        txo: JsonTxOut,
    },
    get_block {
        block: Block,
        block_contents: BlockContents,
    },
    check_receiver_receipt_status {
        receipt_transaction_status: ReceiptTransactionStatus,
    },
    create_receiver_receipts {
        receiver_receipts: Vec<ReceiverReceipt>,
    },
    build_gift_code {
        tx_proposal: TxProposal,
        gift_code_b58: String,
        gift_code: GiftCode,
    },
    get_gift_code {
        gift_code: GiftCode,
    },
    get_all_gift_codes {
        gift_codes: Vec<GiftCode>,
    },
    check_gift_code_status {
        gift_code_status: GiftCodeStatus,
        gift_code: Option<GiftCode>,
    },
    claim_gift_code {
        transaction_log_id: String,
        gift_code: GiftCode,
    },
    remove_gift_code {
        removed: bool,
    },
}
