use mc_full_service::json_rpc::json_rpc_response::JsonCommandResponse as JsonCommandResponseTrait;

use mc_transaction_signer::types::{AccountInfo, TxoSynced};

use serde::{Deserialize, Serialize};

/// Responses from the Full Service Wallet.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
#[allow(clippy::large_enum_variant)]
pub enum JsonCommandResponse {
    create_account {
        mnemonic: String,
    },
    get_account {
        info: AccountInfo,
    },
    sign_tx {},
    sync_txos {
        account_id: String,
        txos_synced: Vec<TxoSynced>,
    },
}

impl JsonCommandResponseTrait for JsonCommandResponse {}
