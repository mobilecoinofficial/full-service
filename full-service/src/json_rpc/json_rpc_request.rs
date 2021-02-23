// Copyright (c) 2020-2021 MobileCoin Inc.

//! The JSON RPC 2.0 Requests to the Wallet API for Full Service.
//!
//! API v2

use crate::json_rpc::api_v1::wallet_api::JsonCommandRequestV1;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// FIXME: Update
/// Help string when invoking GET on the wallet endpoint.
pub fn help_str_v2() -> String {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequestV2::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    help_str
}

/// JSON RPC 2.0 Request.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct JsonCommandRequest {
    /// The method to be invoked on the server.
    pub method: String,

    /// The parameters to be provided to the method.
    ///
    /// Optional, as some methods do not take parameters.
    pub params: Option<serde_json::Value>,

    /// The JSON RPC Version (Should always be 2.0)
    ///
    /// Optional for backwards compatibility because the previous version of
    /// this API (v1) did not require the jsonrpc parameter.
    pub jsonrpc: Option<String>,

    /// The ID to be associated with this request.
    ///
    /// Optional because a "notify" method does not need to correlate an ID on
    /// the response.
    pub id: Option<u32>,

    /// The Full Service Wallet API version.
    ///
    /// Optional: If omitted, assumes V1.
    pub api_version: Option<String>,
}

impl TryFrom<&JsonCommandRequest> for JsonCommandRequestV1 {
    type Error = String;

    fn try_from(src: &JsonCommandRequest) -> Result<JsonCommandRequestV1, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

impl TryFrom<&JsonCommandRequest> for JsonCommandRequestV2 {
    type Error = String;

    fn try_from(src: &JsonCommandRequest) -> Result<JsonCommandRequestV2, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

/// Requests to the Full Service Wallet Service.
#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequestV2 {
    create_account {
        name: Option<String>,
        first_block: Option<String>,
    },
    import_account {
        entropy: String,
        name: Option<String>,
        first_block: Option<String>,
    },
    get_all_accounts,
    get_account {
        account_id: String,
    },
    update_account_name {
        account_id: String,
        name: String,
    },
    delete_account {
        account_id: String,
    },
    get_balance_for_account {
        account_id: String,
    },
    /*
    get_balance_for_subaddress {
        account_id: String,
        subaddress_index: String,
    },*/
    get_wallet_status,
    get_account_status {
        account_id: String,
    },
    /*
    get_all_txos_by_account {
        account_id: String,
    },
    get_txo {
        txo_id: String,
    },
    create_address {
        account_id: String,
        comment: Option<String>,
    },
    get_all_addresses_by_account {
        account_id: String,
    },
    send_transaction {
        account_id: String,
        recipient_public_address: String,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    },
    build_transaction {
        account_id: String,
        recipient_public_address: String,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    },
    submit_transaction {
        tx_proposal: StringifiedJsonTxProposal,
        comment: Option<String>,
        account_id: Option<String>,
    },
    get_all_transactions_by_account {
        account_id: String,
    },
    get_transaction {
        transaction_log_id: String,
    },
    get_transaction_object {
        transaction_log_id: String,
    },
    get_txo_object {
        txo_id: String,
    },
    get_block_object {
        block_index: String,
    },
    get_proofs {
        transaction_log_id: String,
    },
    verify_proof {
        account_id: String,
        txo_id: String,
        proof: String,
    },*/
}
