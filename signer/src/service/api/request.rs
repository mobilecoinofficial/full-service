use mc_full_service::json_rpc::{
    json_rpc_request::JsonRPCRequest, v2::models::tx_proposal::UnsignedTxProposal,
};

use mc_transaction_signer::types::TxoUnsynced;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::{EnumIter, IntoEnumIterator};

pub fn help_str() -> String {
    let mut help_str = "Please use json data to choose api commands. For example, \n\ncurl -s localhost:9090/wallet/v2 -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequest::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    help_str
}

impl TryFrom<&JsonRPCRequest> for JsonCommandRequest {
    type Error = String;

    fn try_from(src: &JsonRPCRequest) -> Result<JsonCommandRequest, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))
    }
}

/// Requests to the Transaction Signer Service.
#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequest {
    create_account {},
    get_account {
        mnemonic: String,
    },
    sign_tx {
        mnemonic: String,
        unsigned_tx: UnsignedTxProposal,
    },
    sync_txos {
        mnemonic: String,
        txos: Vec<TxoUnsynced>,
    },
}
