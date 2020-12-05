// Copyright (c) 2020 MobileCoin Inc.

//! Decorated types for the service to return, with constructors from the database types.

use crate::db::models::{AccountTxoStatus, AssignedSubaddress, TransactionLog, Txo};
use mc_mobilecoind_json::data_types::{JsonTxOut, JsonTxOutMembershipElement};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonCreateAccountResponse {
    pub entropy: String,
    pub public_address_b58: String,
    pub account_id: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonImportAccountResponse {
    pub public_address_b58: String,
    pub account_id: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonAccount {
    pub account_id: String,
    pub name: String,
    pub synced_blocks: String,
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
    pub received_block_height: Option<String>,
    pub pending_tombstone_block_height: Option<String>,
    pub spent_block_height: Option<String>,
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
            received_block_height: txo.received_block_height.map(|x| x.to_string()),
            pending_tombstone_block_height: txo
                .pending_tombstone_block_height
                .map(|x| x.to_string()),
            spent_block_height: txo.spent_block_height.map(|x| x.to_string()),
            proof: txo.proof.as_ref().map(|x| hex::encode(x)),
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
    pub local_block_height: String,
    pub synced_blocks: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonAddress {
    pub public_address_b58: String,
    pub subaddress_index: String,
    pub address_book_entry_id: Option<String>,
    pub comment: String,
}

impl JsonAddress {
    pub fn new(assigned_subaddress: &AssignedSubaddress) -> Self {
        Self {
            public_address_b58: assigned_subaddress.assigned_subaddress_b58.clone(),
            subaddress_index: assigned_subaddress.subaddress_index.to_string(),
            address_book_entry_id: assigned_subaddress
                .address_book_entry
                .map(|x| x.to_string()),
            comment: assigned_subaddress.comment.clone(),
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
    pub block_height: String,
    pub comment: String,
    pub direction: String,
    pub input_txo_ids: Vec<String>,
    pub output_txo_ids: Vec<String>,
    pub change_txo_ids: Vec<String>,
}

impl JsonTransactionResponse {
    pub fn new(
        transaction_log: &TransactionLog,
        inputs: &Vec<String>,
        outputs: &Vec<String>,
        change: &Vec<String>,
    ) -> Self {
        Self {
            transaction_id: transaction_log.transaction_id_hex.clone(),
            account_id: transaction_log.account_id_hex.clone(),
            recipient_public_address: transaction_log.recipient_public_address_b58.clone(),
            assigned_subaddress: transaction_log.assigned_subaddress_b58.clone(),
            value: transaction_log.value.to_string(),
            fee: transaction_log.fee.map(|x| x.to_string()),
            status: transaction_log.status.clone(),
            sent_time: transaction_log.sent_time.clone(),
            block_height: transaction_log.block_height.to_string(),
            comment: transaction_log.comment.clone(),
            direction: transaction_log.direction.clone(),
            input_txo_ids: inputs.clone(),
            output_txo_ids: outputs.clone(),
            change_txo_ids: change.clone(),
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
