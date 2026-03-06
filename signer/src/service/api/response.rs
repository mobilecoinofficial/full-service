// Copyright (c) 2020-2023 MobileCoin Inc.

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
    create_account {
        mnemonic: String,
        account_info: AccountInfo,
    },
    get_account {
        account_info: AccountInfo,
    },
    sign_tx {
        tx_proposal: TxProposal,
    },
    sign_tx_blueprint {
        tx_proposal: TxProposal,
    },
    sync_txos {
        txos_synced: Vec<TxoSynced>,
    },
}

impl JsonCommandResponseTrait for JsonCommandResponse {}
