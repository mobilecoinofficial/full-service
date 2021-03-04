// Copyright (c) 2020-2021 MobileCoin Inc.

//! Decorated types for the service to return, with constructors from the
//! database types.

use crate::db::{models::AssignedSubaddress, txo::TxoDetails};
use mc_mobilecoind_json::data_types::{JsonTxOut, JsonTxOutMembershipElement};
use serde_derive::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonTxo {
    pub object: String,
    pub txo_id: String,
    pub value_pmob: String,
    pub received_block_index: Option<String>,
    pub spent_block_index: Option<String>,
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

        if let Some(spent) = txo_details.secreted_from_account.clone() {
            account_status_map.insert(
                spent.account_id_hex,
                json!({"txo_type": spent.txo_type, "txo_status": spent.txo_status}).into(),
            );
        }

        Self {
            object: "txo".to_string(),
            txo_id: txo_details.txo.txo_id_hex.clone(),
            value_pmob: txo_details.txo.value.to_string(),
            received_block_index: txo_details.txo.received_block_index.map(|x| x.to_string()),
            spent_block_index: txo_details.txo.spent_block_index.map(|x| x.to_string()),
            is_spent_recovered: false,
            received_account_id: txo_details
                .received_to_account
                .as_ref()
                .map(|a| a.account_id_hex.clone()),
            minted_account_id: txo_details
                .clone()
                .secreted_from_account
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

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonAddress {
    pub object: String,
    pub address_id: String,
    pub public_address: String,
    pub account_id: String,
    pub address_book_entry_id: Option<String>,
    pub comment: String,
    pub subaddress_index: String,
    pub offset_count: i32,
}

impl JsonAddress {
    pub fn new(assigned_subaddress: &AssignedSubaddress) -> Self {
        Self {
            object: "assigned_address".to_string(),
            address_id: assigned_subaddress.assigned_subaddress_b58.clone(),
            account_id: assigned_subaddress.account_id_hex.to_string(),
            public_address: assigned_subaddress.assigned_subaddress_b58.clone(),
            address_book_entry_id: assigned_subaddress
                .address_book_entry
                .map(|x| x.to_string()),
            comment: assigned_subaddress.comment.clone(),
            subaddress_index: assigned_subaddress.subaddress_index.to_string(),
            offset_count: assigned_subaddress.id,
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonSubmitResponse {
    pub transaction_id: Option<String>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonBlock {
    pub id: String,
    pub version: String,
    pub parent_id: String,
    pub index: String,
    pub cumulative_txo_count: String,
    pub root_element: JsonTxOutMembershipElement,
    pub contents_hash: String,
}

impl JsonBlock {
    pub fn new(block: &mc_transaction_core::Block) -> Self {
        let membership_element_proto =
            mc_api::external::TxOutMembershipElement::from(&block.root_element);
        Self {
            id: hex::encode(block.id.clone()),
            version: block.version.to_string(),
            parent_id: hex::encode(block.parent_id.clone()),
            index: block.index.to_string(),
            cumulative_txo_count: block.cumulative_txo_count.to_string(),
            root_element: JsonTxOutMembershipElement::from(&membership_element_proto),
            contents_hash: hex::encode(block.contents_hash.0),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonBlockContents {
    pub key_images: Vec<String>,
    pub outputs: Vec<JsonTxOut>,
}

impl JsonBlockContents {
    pub fn new(block_contents: &mc_transaction_core::BlockContents) -> Self {
        Self {
            key_images: block_contents
                .key_images
                .iter()
                .map(|k| hex::encode(mc_util_serial::encode(k)))
                .collect::<Vec<String>>(),
            outputs: block_contents
                .outputs
                .iter()
                .map(|txo| {
                    let proto_txo = mc_api::external::TxOut::from(txo);
                    JsonTxOut::from(&proto_txo)
                })
                .collect::<Vec<JsonTxOut>>(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonProof {
    pub object: String,
    pub txo_id: String,
    pub proof: String,
}
