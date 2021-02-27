// Copyright (c) 2020-2021 MobileCoin Inc.

//! Decorated types for the service to return, with constructors from the
//! database types.

use crate::db::{
    models::{AssignedSubaddress, TransactionLog},
    transaction_log::AssociatedTxos,
    txo::TxoDetails,
};
use chrono::{TimeZone, Utc};
use mc_mobilecoind_json::data_types::{JsonTxOut, JsonTxOutMembershipElement};
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
            received_block_height: txo_details.txo.received_block_count.map(|x| x.to_string()),
            spent_block_height: txo_details.txo.spent_block_count.map(|x| x.to_string()),
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

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonTransactionLog {
    pub object: String,
    pub transaction_log_id: String,
    pub direction: String,
    pub is_sent_recovered: Option<bool>,
    pub account_id: String,
    pub recipient_address_id: Option<String>,
    pub assigned_address_id: Option<String>,
    pub value_pmob: String,
    pub fee_pmob: Option<String>,
    pub submitted_block_height: Option<String>,
    pub finalized_block_height: Option<String>,
    pub status: String,
    pub input_txo_ids: Vec<String>,
    pub output_txo_ids: Vec<String>,
    pub change_txo_ids: Vec<String>,
    pub sent_time: Option<String>,
    pub comment: String,
    pub failure_code: Option<i32>,
    pub failure_message: Option<String>,
    pub offset_count: i32,
}

impl JsonTransactionLog {
    pub fn new(transaction_log: &TransactionLog, associated_txos: &AssociatedTxos) -> Self {
        let recipient_address_id = transaction_log.recipient_public_address_b58.clone();
        let assigned_address_id = transaction_log.assigned_subaddress_b58.clone();
        Self {
            object: "transaction_log".to_string(),
            transaction_log_id: transaction_log.transaction_id_hex.clone(),
            direction: transaction_log.direction.clone(),
            is_sent_recovered: None, // FIXME: WS-16 "Is Sent Recovered"
            account_id: transaction_log.account_id_hex.clone(),
            recipient_address_id: if recipient_address_id == "" {
                None
            } else {
                Some(recipient_address_id)
            },
            assigned_address_id: if assigned_address_id == "" {
                None
            } else {
                Some(assigned_address_id)
            },
            value_pmob: transaction_log.value.to_string(),
            fee_pmob: transaction_log.fee.map(|x| x.to_string()),
            submitted_block_height: transaction_log.submitted_block_count.map(|b| b.to_string()),
            finalized_block_height: transaction_log.finalized_block_count.map(|b| b.to_string()),
            status: transaction_log.status.clone(),
            input_txo_ids: associated_txos.inputs.clone(),
            output_txo_ids: associated_txos.outputs.clone(),
            change_txo_ids: associated_txos.change.clone(),
            sent_time: transaction_log
                .sent_time
                .map(|t| Utc.timestamp(t, 0).to_string()),
            comment: transaction_log.comment.clone(),
            failure_code: None,    // FIXME: WS-17 Failiure code
            failure_message: None, // FIXME: WS-17 Failure message
            offset_count: transaction_log.id,
        }
    }
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
