// Copyright (c) 2020-2021 MobileCoin Inc.

// use crate::db::models::TransactionLog;
// use crate::db::transaction_log::AssociatedTxos;
use crate::db;
use chrono::offset::TimeZone;
use chrono::Utc;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TransactionLog {
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

impl TransactionLog {
    pub fn new(transaction_log: &db::TransactionLog, associated_txos: &db::AssociatedTxos) -> Self {
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
