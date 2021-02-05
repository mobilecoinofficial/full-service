use crate::db::txo::TxoDetails;
use serde_derive::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonTxo {
    pub object: String,
    pub txo_id: String,
    pub value_pmob: String,
    pub received_block_height: Option<String>,
    pub spent_block_height: Option<String>,
    pub is_spent_recovered: bool, // FIXME: WS-16 is_spent_recovered
    pub received_account_id: Option<String>,
    pub minted_account_id: Option<String>,
    pub account_status_map: Map<String, serde_json::Value>,
    pub target_key: String,
    pub public_key: String,
    pub e_fog_hint: String,
    pub subaddress_index: Option<String>,
    pub assigned_subaddress: Option<String>,
    pub key_image: Option<String>,
    pub proof: Option<String>,
    pub offset_count: i32,
}

impl JsonTxo {
    pub fn new(txo_details: &TxoDetails) -> Self {
        let mut account_status_map: Map<String, serde_json::Value> = Map::new();

        if let Some(received) = txo_details.received_to_account.clone() {
            account_status_map.insert(
                received.account_id_hex,
                json!({"txo_type": received.txo_type, "txo_status": received.txo_status}).into(),
            );
        }

        if let Some(spent) = txo_details.spent_from_account.clone() {
            account_status_map.insert(
                spent.account_id_hex,
                json!({"txo_type": spent.txo_type, "txo_status": spent.txo_status}).into(),
            );
        }

        Self {
            object: "txo".to_string(),
            txo_id: txo_details.txo.txo_id_hex.clone(),
            value_pmob: txo_details.txo.value.to_string(),
            received_block_height: txo_details.txo.received_block_count.map(|x| x.to_string()),
            spent_block_height: txo_details.txo.spent_block_count.map(|x| x.to_string()),
            is_spent_recovered: false,
            received_account_id: txo_details
                .received_to_account
                .as_ref()
                .map(|a| a.account_id_hex.clone()),
            minted_account_id: txo_details
                .clone()
                .spent_from_account
                .as_ref()
                .map(|a| a.account_id_hex.clone()),
            account_status_map,
            target_key: hex::encode(&txo_details.txo.target_key),
            public_key: hex::encode(&txo_details.txo.public_key),
            e_fog_hint: hex::encode(&txo_details.txo.e_fog_hint),
            subaddress_index: txo_details.txo.subaddress_index.map(|s| s.to_string()),
            assigned_subaddress: txo_details
                .received_to_assigned_subaddress
                .clone()
                .map(|a| a.assigned_subaddress_b58),
            key_image: txo_details.txo.key_image.as_ref().map(|k| hex::encode(&k)),
            proof: txo_details.txo.proof.as_ref().map(hex::encode),
            offset_count: txo_details.txo.id,
        }
    }
}
