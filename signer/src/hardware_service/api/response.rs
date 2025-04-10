// Copyright (c) 2020-2024 MobileCoin Inc.

use mc_full_service::json_rpc::{
    json_rpc_response::JsonCommandResponse as JsonCommandResponseTrait,
    v2::models::tx_proposal::TxProposal,
};
use mc_transaction_signer::types::{AccountInfo, TxoSynced};
use serde::{Deserialize, Serialize};

/// Responses from the Signer Service.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
#[allow(clippy::large_enum_variant)]
pub enum JsonCommandResponse {
    get_account {
        account_id: String,
        account_info: AccountInfo,
    },
    sync_txos {
        account_id: String,
        synced_txos: Vec<TxoSynced>,
    },
    sign_tx {
        tx_proposal: TxProposal,
    },
}

impl JsonCommandResponseTrait for JsonCommandResponse {}
