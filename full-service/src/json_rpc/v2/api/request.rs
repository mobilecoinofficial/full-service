// Copyright (c) 2020-2021 MobileCoin Inc.

//! The JSON RPC 2.0 Requests to the Wallet API for Full Service.

use crate::json_rpc::{
    json_rpc_request::JsonRPCRequest,
    v2::models::{
        account_key::FogInfo, amount::Amount, receiver_receipt::ReceiverReceipt,
        tx_proposal::TxProposal,
    },
};

use mc_mobilecoind_json::data_types::JsonTxOut;
use mc_transaction_signer::types::TxoSynced;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub fn help_str() -> String {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet/v2 -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequest::iter() {
        help_str.push_str(&format!("{e:?}\n\n"));
    }
    help_str
}

impl TryFrom<&JsonRPCRequest> for JsonCommandRequest {
    type Error = String;

    fn try_from(src: &JsonRPCRequest) -> Result<JsonCommandRequest, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        serde_json::from_value(src_json).map_err(|e| format!("Could not get value {e:?}"))
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
        addresses_and_amounts: Option<Vec<(String, Amount)>>,
        recipient_public_address: Option<String>,
        amount: Option<Amount>,
        input_txo_ids: Option<Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
        block_version: Option<String>,
        sender_memo_credential_subaddress_index: Option<String>,
        payment_request_id: Option<String>,
        spend_subaddress: Option<String>,
    },
    build_burn_transaction {
        account_id: String,
        amount: Amount,
        redemption_memo_hex: Option<String>,
        input_txo_ids: Option<Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        block_version: Option<String>,
        spend_subaddress: Option<String>,
    },
    build_transaction {
        account_id: String,
        addresses_and_amounts: Option<Vec<(String, Amount)>>,
        recipient_public_address: Option<String>,
        amount: Option<Amount>,
        input_txo_ids: Option<Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        block_version: Option<String>,
        sender_memo_credential_subaddress_index: Option<String>,
        payment_request_id: Option<String>,
        spend_subaddress: Option<String>,
    },
    build_unsigned_burn_transaction {
        account_id: String,
        amount: Amount,
        redemption_memo_hex: Option<String>,
        input_txo_ids: Option<Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        block_version: Option<String>,
        spend_subaddress: Option<String>,
    },
    build_unsigned_transaction {
        account_id: String,
        addresses_and_amounts: Option<Vec<(String, Amount)>>,
        recipient_public_address: Option<String>,
        amount: Option<Amount>,
        input_txo_ids: Option<Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        block_version: Option<String>,
        spend_subaddress: Option<String>,
    },
    check_b58_type {
        b58_code: String,
    },
    check_receiver_receipt_status {
        address: String,
        receiver_receipt: ReceiverReceipt,
    },
    create_account {
        name: Option<String>,
        fog_info: Option<FogInfo>,
        #[serde(default = "bool::default")] // default is false
        require_spend_subaddress: bool,
    },
    create_payment_request {
        account_id: String,
        subaddress_index: Option<i64>,
        amount: Amount,
        memo: Option<String>,
    },
    create_receiver_receipts {
        tx_proposal: TxProposal,
    },
    create_view_only_account_import_request {
        account_id: String,
    },
    create_view_only_account_sync_request {
        account_id: String,
    },
    export_account_secrets {
        account_id: String,
    },
    get_account_status {
        account_id: String,
    },
    get_accounts {
        offset: Option<u64>,
        limit: Option<u64>,
    },
    get_address_details {
        address: String,
    },
    get_address_for_account {
        account_id: String,
        index: i64,
    },
    get_address_status {
        address: String,
    },
    get_address {
        public_address_b58: String,
    },
    get_addresses {
        account_id: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
    },
    get_balance {
        account_id: String,
    },
    get_block {
        block_index: Option<String>,
        txo_public_key: Option<String>,
    },
    get_blocks {
        first_block_index: String,
        limit: usize,
    },
    get_recent_blocks {
        limit: Option<usize>,
    },
    get_confirmations {
        transaction_log_id: String,
    },
    get_mc_protocol_transaction {
        transaction_log_id: String,
    },
    get_mc_protocol_txo {
        txo_id: String,
    },
    get_network_status,
    get_token_metadata,
    get_transaction_log {
        transaction_log_id: String,
    },
    get_transaction_logs {
        account_id: Option<String>,
        min_block_index: Option<String>,
        max_block_index: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
    },
    get_txo_block_index {
        public_key: String,
    },
    get_txo_membership_proofs {
        outputs: Vec<JsonTxOut>,
    },
    get_txo {
        txo_id: String,
    },
    get_txos {
        account_id: Option<String>,
        address: Option<String>,
        status: Option<String>,
        token_id: Option<String>,
        min_received_block_index: Option<String>,
        max_received_block_index: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
    },
    get_wallet_status,
    import_account_from_legacy_root_entropy {
        entropy: String,
        name: Option<String>,
        first_block_index: Option<String>,
        next_subaddress_index: Option<String>,
        fog_info: Option<FogInfo>,
        #[serde(default = "bool::default")] // default is false
        require_spend_subaddress: bool,
    },
    import_account {
        mnemonic: String,
        name: Option<String>,
        first_block_index: Option<String>,
        next_subaddress_index: Option<String>,
        fog_info: Option<FogInfo>,
        #[serde(default = "bool::default")] // default is false
        require_spend_subaddress: bool,
    },
    import_view_only_account {
        view_private_key: String,
        spend_public_key: String,
        name: Option<String>,
        first_block_index: Option<String>,
        next_subaddress_index: Option<String>,
        #[serde(default = "bool::default")] // default is false
        require_spend_subaddress: bool,
    },
    import_view_only_account_from_hardware_wallet {
        name: Option<String>,
        first_block_index: Option<String>,
        fog_info: Option<FogInfo>,
        #[serde(default = "bool::default")] // default is false
        require_spend_subaddress: bool,
    },
    remove_account {
        account_id: String,
    },
    resync_account {
        account_id: String,
    },
    sample_mixins {
        num_mixins: u64,
        excluded_outputs: Vec<JsonTxOut>,
    },
    search_ledger {
        query: String,
    },
    set_require_spend_subaddress {
        account_id: String,
        require_spend_subaddress: bool,
    },
    submit_transaction {
        tx_proposal: TxProposal,
        comment: Option<String>,
        account_id: Option<String>,
    },
    sync_view_only_account {
        account_id: String,
        synced_txos: Option<Vec<TxoSynced>>,
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
    validate_sender_memo {
        txo_id: String,
        sender_address: String,
    },
    validate_proof_of_reserve_sci {
        sci_proto: String,
    },
    verify_address {
        address: String,
    },
    version,
}
