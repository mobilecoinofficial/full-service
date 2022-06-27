// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TransactionLog object.

use chrono::{offset::TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    db,
    db::{
        models::TX_DIRECTION_SENT,
        transaction_log::{AssociatedTxos, TransactionLogModel, TxStatus},
    },
};

/// A log of a transaction that occurred on the MobileCoin network, constructed
/// and/or submitted from an account in this wallet.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TransactionLog {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// Unique identifier for the transaction log. This value is not associated
    /// to the ledger, but derived from the tx.
    pub id: String,

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

    pub fee_value: String,

    pub fee_token_id: String,

    /// The block index of the highest block on the network at the time the
    /// transaction was submitted.
    pub submitted_block_index: Option<String>,

    pub tombstone_block_index: Option<String>,

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
}

impl TransactionLog {
    pub fn new(
        transaction_log: &db::models::TransactionLog,
        associated_txos: &AssociatedTxos,
    ) -> Self {
        Self {
            object: "transaction_log".to_string(),
            id: transaction_log.id.clone(),
            account_id: transaction_log.account_id_hex.clone(),
            submitted_block_index: transaction_log
                .submitted_block_index
                .map(|b| (b as u64).to_string()),
            tombstone_block_index: transaction_log
                .tombstone_block_index
                .map(|b| (b as u64).to_string()),
            finalized_block_index: transaction_log
                .finalized_block_index
                .map(|b| (b as u64).to_string()),
            status: transaction_log.status().to_string(),
            input_txos: associated_txos.inputs.iter().map(TxoAbbrev::new).collect(),
            output_txos: associated_txos.outputs.iter().map(TxoAbbrev::new).collect(),
            change_txos: associated_txos.change.iter().map(TxoAbbrev::new).collect(),
            fee_value: transaction_log.fee_value.to_string(),
            fee_token_id: transaction_log.fee_token_id.to_string(),
            sent_time: None,
            comment: transaction_log.comment.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TxoAbbrev {
    pub txo_id_hex: String,

    /// Unique identifier for the recipient associated account. Blank unless
    /// direction is "sent".
    pub recipient_address_id: String,

    /// Value of this txo.
    pub value: String,

    /// Token ID of this txo
    pub token_id: String,
}

impl TxoAbbrev {
    pub fn new(txo: &db::models::Txo) -> Self {
        Self {
            txo_id_hex: txo.txo_id_hex.clone(),
            recipient_address_id: txo.recipient_public_address_b58.clone(),
            value: (txo.value as u64).to_string(),
            token_id: (txo.token_id as u64).to_string(),
        }
    }
}
