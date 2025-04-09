// Copyright (c) 2020-2024 MobileCoin Inc.

use mc_full_service::json_rpc::{
    json_rpc_request::JsonRPCRequest, v2::models::tx_proposal::UnsignedTxProposal,
};
use mc_transaction_signer::types::TxoUnsynced;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::{EnumIter, IntoEnumIterator};

pub fn help_str() -> String {
    let mut help_str =
        "Please use json data to choose api commands. Available commands are: \n".to_string();
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

/// Requests to the Signer Service.
#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequest {
    get_account,
    sync_txos {
        account_id: String,
        txos_unsynced: Vec<TxoUnsynced>,
    },
    sign_tx {
        account_id: String,
        unsigned_tx_proposal: UnsignedTxProposal,
    },
}
