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
    pub id: u32,
}

impl TryFrom<&JsonRPCRequest> for JsonCommandRequest {
    type Error = String;

    fn try_from(src: &JsonRPCRequest) -> Result<JsonCommandRequest, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

/// Requests to the Full Service Wallet Service.
#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequest {
    create_account {
        name: Option<String>,
        first_block_index: Option<String>,
    },
    import_account {
        entropy: String,
        name: Option<String>,
        first_block_index: Option<String>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
    },
    export_account_secrets {
        account_id: String,
    },
    get_all_accounts,
    get_account {
        account_id: String,
    },
    update_account_name {
        account_id: String,
        name: String,
    },
    remove_account {
        account_id: String,
    },
    get_balance_for_account {
        account_id: String,
    },
    build_and_submit_transaction {
        account_id: String,
        recipient_public_address: String,
        value_pmob: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    },
    build_transaction {
        account_id: String,
        recipient_public_address: String,
        value_pmob: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    },
    submit_transaction {
        tx_proposal: TxProposal,
        comment: Option<String>,
        account_id: Option<String>,
    },
    get_all_transaction_logs_for_account {
        account_id: String,
    },
    get_transaction_log {
        transaction_log_id: String,
    },
    get_all_transaction_logs_for_block {
        block_index: String,
    },
    get_all_transaction_logs_ordered_by_block,
    get_wallet_status,
    get_account_status {
        account_id: String,
    },
    assign_address_for_account {
        account_id: String,
        metadata: Option<String>,
    },
    get_all_addresses_for_account {
        account_id: String,
    },
    verify_address {
        address: String,
    },
    get_balance_for_address {
        address: String,
    },
    get_all_txos_for_account {
        account_id: String,
    },
    get_txo {
        txo_id: String,
    },
    get_all_txos_for_address {
        address: String,
    },
    get_proofs {
        transaction_log_id: String,
    },
    verify_proof {
        account_id: String,
        txo_id: String,
        proof: String,
    },
    get_mc_protocol_transaction {
        transaction_log_id: String,
    },
    get_mc_protocol_txo {
        txo_id: String,
    },
    get_block {
        block_index: String,
    },
    check_receiver_receipt_status {
        address: String,
        receiver_receipt: ReceiverReceipt,
    },
    create_receiver_receipts {
        tx_proposal: TxProposal,
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
    submit_gift_code {
        from_account_id: String,
        gift_code_b58: String,
        tx_proposal: TxProposal,
    },
    get_gift_code {
        gift_code_b58: String,
    },
    get_all_gift_codes,
    check_gift_code_status {
        gift_code_b58: String,
    },
    claim_gift_code {
        gift_code_b58: String,
        account_id: String,
        address: Option<String>,
    },
    remove_gift_code {
        gift_code_b58: String,
    },
}
