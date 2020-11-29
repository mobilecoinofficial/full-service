// Copyright (c) 2020 MobileCoin Inc.

//! Decorated types for the service to return, with constructors from the database types.

use crate::models::{AccountTxoStatus, TransactionLog, Txo};
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

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonBalanceResponse {
    pub unspent: String,
    pub pending: String,
    pub spent: String,
    pub unknown: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonCreateAddressResponse {
    pub public_address_b58: String,
    pub address_book_entry_id: Option<String>,
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
