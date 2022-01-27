// Copyright (c) 2020-2021 MobileCoin Inc.

//! The JSON RPC 2.0 Requests to the Wallet API for Full Service.

use crate::json_rpc::tx_proposal::TxProposal;

use crate::json_rpc::receiver_receipt::ReceiverReceipt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// FIXME: Update
/// Help string when invoking GET on the wallet endpoint.
pub fn help_str() -> String {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequest::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    help_str
}

/// JSON-RPC 2.0 Request.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct JsonRPCRequest {
    /// The method to be invoked on the server.
    pub method: String,

    /// The parameters to be provided to the method.
    ///
    /// Optional, as some methods do not take parameters.
    pub params: Option<serde_json::Value>,

    /// The JSON-RPC Version (Should always be 2.0)
    pub jsonrpc: String,

    /// The ID to be associated with this request.
    /// JSON-RPC Notification requests are not yet supported, so this field is
    /// not optional.
    pub id: serde_json::Value,
}

impl TryFrom<&JsonRPCRequest> for JsonCommandRequest {
    type Error = String;

    fn try_from(src: &JsonRPCRequest) -> Result<JsonCommandRequest, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))
    }
}

/// Requests to the Full Service Wallet Service.
#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequest {
    assign_address_for_account {
        account_id: String,
        metadata: Option<String>,
    },
    build_and_submit_transaction {
        account_id: String,
        addresses_and_values: Option<Vec<(String, String)>>,
        recipient_public_address: Option<String>,
        value_pmob: Option<String>,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    },
    build_gift_code {
        account_id: String,
        value_pmob: String,
        memo: Option<String>,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    },
    build_split_txo_transaction {
        txo_id: String,
        output_values: Vec<String>,
        destination_subaddress_index: Option<String>,
        fee: Option<String>,
        tombstone_block: Option<String>,
    },
    build_transaction {
        account_id: String,
        addresses_and_values: Option<Vec<(String, String)>>,
        recipient_public_address: Option<String>,
        value_pmob: Option<String>,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        log_tx_proposal: Option<bool>,
    },
    check_b58_type {
        b58_code: String,
    },
    check_gift_code_status {
        gift_code_b58: String,
    },
    check_receiver_receipt_status {
        address: String,
        receiver_receipt: ReceiverReceipt,
    },
    claim_gift_code {
        gift_code_b58: String,
        account_id: String,
        address: Option<String>,
    },
    create_account {
        name: Option<String>,
    },
    create_payment_request {
        account_id: String,
        subaddress_index: Option<i64>,
        amount_pmob: u64,
        memo: Option<String>,
    },
    create_receiver_receipts {
        tx_proposal: TxProposal,
    },
    export_account_secrets {
        account_id: String,
    },
    get_account {
        account_id: String,
    },
    get_account_status {
        account_id: String,
    },
    get_address_for_account {
        account_id: String,
        index: i64,
    },
    get_addresses_for_account {
        account_id: String,
        offset: String,
        limit: String,
    },
    get_all_accounts,
    get_all_addresses_for_account {
        account_id: String,
    },
    get_all_gift_codes,
    get_all_transaction_logs_for_account {
        account_id: String,
    },
    get_all_transaction_logs_for_block {
        block_index: String,
    },
    get_all_transaction_logs_ordered_by_block,
    get_all_txos_for_account {
        account_id: String,
    },
    get_all_txos_for_address {
        address: String,
    },
    get_balance_for_account {
        account_id: String,
    },
    get_balance_for_address {
        address: String,
    },
    get_block {
        block_index: String,
    },
    get_confirmations {
        transaction_log_id: String,
    },
    get_gift_code {
        gift_code_b58: String,
    },
    get_mc_protocol_transaction {
        transaction_log_id: String,
    },
    get_mc_protocol_txo {
        txo_id: String,
    },
    get_network_status,
    get_transaction_log {
        transaction_log_id: String,
    },
    get_transaction_logs_for_account {
        account_id: String,
        offset: String,
        limit: String,
    },
    get_txo {
        txo_id: String,
    },
    get_txos_for_account {
        account_id: String,
        offset: String,
        limit: String,
    },
    get_wallet_status,
    import_account {
        mnemonic: String,
        key_derivation_version: String,
        name: Option<String>,
        first_block_index: Option<String>,
        next_subaddress_index: Option<String>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
    },
    import_account_from_legacy_root_entropy {
        entropy: String,
        name: Option<String>,
        first_block_index: Option<String>,
        next_subaddress_index: Option<String>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
    },
    remove_account {
        account_id: String,
    },
    remove_gift_code {
        gift_code_b58: String,
    },
    submit_gift_code {
        from_account_id: String,
        gift_code_b58: String,
        tx_proposal: TxProposal,
    },
    submit_transaction {
        tx_proposal: TxProposal,
        comment: Option<String>,
        account_id: Option<String>,
    },
    update_account_name {
        account_id: String,
        name: String,
    },
    validate_confirmation {
        account_id: String,
        txo_id: String,
        confirmation: String,
    },
    verify_address {
        address: String,
    },
}
