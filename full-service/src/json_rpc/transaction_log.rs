// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TransactionLog object.

use chrono::{offset::TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::{db, db::transaction_log::AssociatedTxos};

/// A log of a transaction that occurred on the MobileCoin network, constructed
/// and/or submitted from an account in this wallet.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TransactionLog {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// Unique identifier for the transaction log. This value is not associated
    /// to the ledger.
    pub transaction_log_id: String,

    /// A string that identifies if this transaction log was sent or received.
    /// Valid values are "sent" or "received".
    pub direction: String,

    /// Flag that indicates if the sent transaction log was recovered from the
    /// ledger. This value is null for "received" transaction logs. If true,
    /// some information may not be available on the transaction log and its
    /// txos without user input. If true, the fee receipient_address_id, fee,
    /// and sent_time will be null without user input.
    pub is_sent_recovered: Option<bool>,

    /// Unique identifier for the assigned associated account. If the
    /// transaction is outgoing, this account is from whence the txo came. If
    /// received, this is the receiving account.
    pub account_id: String,

    /// A list of the Txos which were inputs to this transaction.
    pub input_txos: Vec<TxoAbbrev>,

    /// A list of the Txos which were outputs from this transaction.
    pub output_txos: Vec<TxoAbbrev>,

    /// A list of the Txos which were change in this transaction.
    pub change_txos: Vec<TxoAbbrev>,

    /// Unique identifier for the assigned associated account. Only available if
    /// direction is "received".
    pub assigned_address_id: Option<String>,

    /// Value in pico MOB associated to this transaction log.
    pub value_pmob: String,

    /// Fee in pico MOB associated to this transaction log. Only on outgoing
    /// transaction logs. Only available if direction is "sent".
    pub fee_pmob: Option<String>,

    /// The block index of the highest block on the network at the time the
    /// transaction was submitted.
    pub submitted_block_index: Option<String>,

    ///  The scanned block block index in which this transaction occurred.
    pub finalized_block_index: Option<String>,

    /// String representing the transaction log status. On "sent", valid
    /// statuses are "built", "pending", "succeeded", "failed".  On "received",
    /// the status is "succeeded".
    pub status: String,

    /// Time at which sent transaction log was created. Only available if
    /// direction is "sent". This value is null if "received" or if the sent
    /// transactions were recovered from the ledger (is_sent_recovered = true).
    pub sent_time: Option<String>,

    /// An arbitrary string attached to the object.
    pub comment: String,

    /// Code representing the cause of "failed" status.
    pub failure_code: Option<i32>,

    /// Human parsable explanation of "failed" status.
    pub failure_message: Option<String>,
}

impl TransactionLog {
    pub fn new(
        transaction_log: &db::models::TransactionLog,
        associated_txos: &AssociatedTxos,
    ) -> Self {
        let assigned_address_id = transaction_log.assigned_subaddress_b58.clone();
        Self {
            object: "transaction_log".to_string(),
            transaction_log_id: transaction_log.transaction_id_hex.clone(),
            direction: transaction_log.direction.clone(),
            is_sent_recovered: None, // FIXME: WS-16 "Is Sent Recovered"
            account_id: transaction_log.account_id_hex.clone(),
            assigned_address_id,
            value_pmob: (transaction_log.value as u64).to_string(),
            fee_pmob: transaction_log.fee.map(|x| (x as u64).to_string()),
            submitted_block_index: transaction_log
                .submitted_block_index
                .map(|b| (b as u64).to_string()),
            finalized_block_index: transaction_log
                .finalized_block_index
                .map(|b| (b as u64).to_string()),
            status: transaction_log.status.clone(),
            input_txos: associated_txos
                .inputs
                .iter()
                .map(|t| TxoAbbrev::new(t))
                .collect(),
            output_txos: associated_txos
                .outputs
                .iter()
                .map(|t| TxoAbbrev::new(t))
                .collect(),
            change_txos: associated_txos
                .change
                .iter()
                .map(|t| TxoAbbrev::new(t))
                .collect(),
            sent_time: transaction_log
                .sent_time
                .map(|t| Utc.timestamp(t, 0).to_string()),
            comment: transaction_log.comment.clone(),
            failure_code: None,    // FIXME: WS-17 Failiure code
            failure_message: None, // FIXME: WS-17 Failure message
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TxoAbbrev {
    pub txo_id_hex: String,

    /// Unique identifier for the recipient associated account. Blank unless
    /// direction is "sent".
    pub recipient_address_id: String,

    /// Available pico MOB for this Txo.
    /// If the account is syncing, this value may change.
    pub value_pmob: String,
}

impl TxoAbbrev {
    pub fn new(txo: &db::models::Txo) -> Self {
        Self {
            txo_id_hex: txo.txo_id_hex.clone(),
            recipient_address_id: txo.recipient_public_address_b58.clone(),
            value_pmob: txo.value.to_string(),
        }
    }
}

// FIXME: Test display for >i64::MAX
