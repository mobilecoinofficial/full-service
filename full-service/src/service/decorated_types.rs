// Copyright (c) 2020 MobileCoin Inc.

//! Decorated types for the service to return, with constructors from the database types.

use crate::db::{
    models::{AccountTxoStatus, AssignedSubaddress, TransactionLog, Txo},
    transaction_log::AssociatedTxos,
};
use chrono::{TimeZone, Utc};
use mc_mobilecoind_json::data_types::{JsonTxOut, JsonTxOutMembershipElement};
use serde_derive::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonCreateAccountResponse {
    pub entropy: String,
    pub account: JsonAccount,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonAccount {
    pub object: String,
    pub account_id: String,
    pub name: String,
    pub network_height: String,
    pub local_height: String,
    pub account_height: String,
    pub is_synced: bool,
    pub available_pmob: String,
    pub pending_pmob: String,
    pub main_address: String,
    pub next_subaddress_index: String,
    pub recovery_mode: bool,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonWalletStatus {
    pub object: String,
    pub network_height: String,
    pub local_height: String,
    pub is_synced_all: bool,
    pub total_available_pmob: String,
    pub total_pending_pmob: String,
    pub account_ids: Vec<String>,
    pub account_map: Map<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonListTxosResponse {
    pub txo_id: String,
    pub value: String,
    pub txo_type: String,
    pub txo_status: String,
}

impl JsonListTxosResponse {
    pub fn new(txo: &Txo, account_txo_status: &AccountTxoStatus) -> Self {
        Self {
            txo_id: txo.txo_id_hex.clone(),
            value: txo.value.to_string(),
            txo_type: account_txo_status.txo_type.clone(),
            txo_status: account_txo_status.txo_status.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonTxo {
    pub txo_id: String,
    pub value: String,
    pub assigned_subaddress: Option<String>,
    pub subaddress_index: Option<String>,
    pub key_image: Option<String>,
    pub txo_type: String,
    pub txo_status: String,
    pub received_block_count: Option<String>,
    pub pending_tombstone_block_count: Option<String>,
    pub spent_block_count: Option<String>,
    pub proof: Option<String>,
}

impl JsonTxo {
    pub fn new(
        txo: &Txo,
        account_txo_status: &AccountTxoStatus,
        assigned_subaddress: Option<&AssignedSubaddress>,
    ) -> Self {
        Self {
            txo_id: txo.txo_id_hex.clone(),
            value: txo.value.to_string(),
            assigned_subaddress: assigned_subaddress.map(|a| a.assigned_subaddress_b58.clone()),
            subaddress_index: txo.subaddress_index.map(|s| s.to_string()),
            key_image: txo.key_image.as_ref().map(|k| hex::encode(&k)),
            txo_type: account_txo_status.txo_type.clone(),
            txo_status: account_txo_status.txo_status.clone(),
            received_block_count: txo.received_block_count.map(|x| x.to_string()),
            pending_tombstone_block_count: txo.pending_tombstone_block_count.map(|x| x.to_string()),
            spent_block_count: txo.spent_block_count.map(|x| x.to_string()),
            proof: txo.proof.as_ref().map(hex::encode),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonBalanceResponse {
    pub unspent: String,
    pub pending: String,
    pub spent: String,
    pub secreted: String,
    pub orphaned: String,
    pub local_block_count: String,
    pub synced_blocks: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
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
    pub transaction_id: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonTransactionResponse {
    pub transaction_id: String,
    pub account_id: String,
    pub recipient_public_address: String,
    pub assigned_subaddress: String,
    pub value: String,
    pub fee: Option<String>,
    pub status: String,
    pub sent_time: String,
    pub submitted_block_count: Option<String>,
    pub finalized_block_count: Option<String>,
    pub comment: String,
    pub direction: String,
    pub input_txo_ids: Vec<String>,
    pub output_txo_ids: Vec<String>,
    pub change_txo_ids: Vec<String>,
}

impl JsonTransactionResponse {
    pub fn new(transaction_log: &TransactionLog, associated_txos: &AssociatedTxos) -> Self {
        Self {
            transaction_id: transaction_log.transaction_id_hex.clone(),
            account_id: transaction_log.account_id_hex.clone(),
            recipient_public_address: transaction_log.recipient_public_address_b58.clone(),
            assigned_subaddress: transaction_log.assigned_subaddress_b58.clone(),
            value: transaction_log.value.to_string(),
            fee: transaction_log.fee.map(|x| x.to_string()),
            status: transaction_log.status.clone(),
            sent_time: transaction_log
                .sent_time
                .map(|t| Utc.timestamp(t, 0).to_string())
                .unwrap_or_else(|| "".to_string()),
            submitted_block_count: transaction_log.submitted_block_count.map(|b| b.to_string()),
            finalized_block_count: transaction_log.finalized_block_count.map(|b| b.to_string()),
            comment: transaction_log.comment.clone(),
            direction: transaction_log.direction.clone(),
            input_txo_ids: associated_txos.inputs.clone(),
            output_txo_ids: associated_txos.outputs.clone(),
            change_txo_ids: associated_txos.change.clone(),
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
