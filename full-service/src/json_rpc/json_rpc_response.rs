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
        network_status::NetworkStatus,
        receiver_receipt::ReceiverReceipt,
        transaction_log::TransactionLog,
        tx_proposal::TxProposal,
        txo::Txo,
        wallet_status::WalletStatus,
    },
    service::{gift_code::GiftCodeStatus, receipt::ReceiptTransactionStatus},
    util::b58::PrintableWrapperType,
};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::HashMap;
use strum::AsStaticRef;
use strum_macros::AsStaticStr;

/// A JSON RPC 2.0 Response.
#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRPCResponse {
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
    pub result: Option<JsonCommandResponse>,

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
pub fn format_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> JsonRPCError {
    let data: serde_json::Value =
        json!({"server_error": format!("{:?}", e), "details": e.to_string()}).into();
    JsonRPCError::error {
        code: JsonRPCErrorCodes::InternalError as i32,
        message: JsonRPCErrorCodes::InternalError.as_static().to_string(),
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
        message: JsonRPCErrorCodes::InvalidRequest.as_static().to_string(),
        data,
    }
}

/// Responses from the Full Service Wallet.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
#[allow(clippy::large_enum_variant)]
pub enum JsonCommandResponse {
    assign_address_for_account {
        address: Address,
    },
    build_and_submit_transaction {
        transaction_log: TransactionLog,
        tx_proposal: TxProposal,
    },
    build_gift_code {
        tx_proposal: TxProposal,
        gift_code_b58: String,
    },
    build_split_txo_transaction {
        tx_proposal: TxProposal,
        transaction_log_id: String,
    },
    build_transaction {
        tx_proposal: TxProposal,
        transaction_log_id: String,
    },
    check_b58_type {
        b58_type: PrintableWrapperType,
        data: HashMap<String, String>,
    },
    check_gift_code_status {
        gift_code_status: GiftCodeStatus,
        gift_code_value: Option<i64>,
        gift_code_memo: String,
    },
    check_receiver_receipt_status {
        receipt_transaction_status: ReceiptTransactionStatus,
        txo: Option<Txo>,
    },
    claim_gift_code {
        txo_id: String,
    },
    create_account {
        account: Account,
    },
    create_payment_request {
        payment_request_b58: String,
    },
    create_receiver_receipts {
        receiver_receipts: Vec<ReceiverReceipt>,
    },
    export_account_secrets {
        account_secrets: AccountSecrets,
    },
    get_account {
        account: Account,
    },
    get_account_status {
        account: Account,
        balance: Balance,
    },
    get_address_for_account {
        address: Address,
    },
    get_addresses_for_account {
        public_addresses: Vec<String>,
        address_map: Map<String, serde_json::Value>,
    },
    get_all_accounts {
        account_ids: Vec<String>,
        account_map: Map<String, serde_json::Value>,
    },
    get_all_gift_codes {
        gift_codes: Vec<GiftCode>,
    },
    get_all_transaction_logs_for_account {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_all_transaction_logs_for_block {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_all_transaction_logs_ordered_by_block {
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_all_txos_for_account {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_all_txos_for_address {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_balance_for_account {
        balance: Balance,
    },
    get_balance_for_address {
        balance: Balance,
    },
    get_block {
        block: Block,
        block_contents: BlockContents,
    },
    get_confirmations {
        confirmations: Vec<Confirmation>,
    },
    get_gift_code {
        gift_code: GiftCode,
    },
    get_mc_protocol_transaction {
        transaction: JsonTx,
    },
    get_mc_protocol_txo {
        txo: JsonTxOut,
    },
    get_network_status {
        network_status: NetworkStatus,
    },
    get_transaction_log {
        transaction_log: TransactionLog,
    },
    get_transaction_logs_for_account {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_txo {
        txo: Txo,
    },
    get_txos_for_account {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_wallet_status {
        wallet_status: WalletStatus,
    },
    import_account {
        account: Account,
    },
    import_account_from_legacy_root_entropy {
        account: Account,
    },
    remove_account {
        removed: bool,
    },
    remove_gift_code {
        removed: bool,
    },
    submit_gift_code {
        gift_code: GiftCode,
    },
    submit_transaction {
        transaction_log: Option<TransactionLog>,
    },
    update_account_name {
        account: Account,
    },
    validate_confirmation {
        validated: bool,
    },
    verify_address {
        verified: bool,
    },
}
