// Copyright (c) 2020-2024 MobileCoin Inc.

use mc_full_service::{
    json_rpc::{
        json_rpc_response::JsonCommandResponse as JsonCommandResponseTrait,
        v2::models::tx_proposal::TxProposal,
    },
    util::b58::b58_public_address::B58PublicAddress,
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
        default_public_address: B58PublicAddress,
        change_public_address: B58PublicAddress,
    },
    sync_txos {
        account_id: String,
        synced_txos: Vec<TxoSynced>,
    },
    sign_tx {
        tx_proposal: TxProposal,
    },
    sign_tx_blueprint {
        tx_proposal: TxProposal,
    },
}

impl JsonCommandResponseTrait for JsonCommandResponse {}
