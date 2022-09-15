// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TransactionLog object.

use std::collections::BTreeMap;

use mc_common::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    db,
    db::transaction_log::{AssociatedTxos, TransactionLogModel, ValueMap},
};

use super::amount::Amount;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TransactionLogMap(pub BTreeMap<String, TransactionLog>);

/// A log of a transaction that occurred on the MobileCoin network, constructed
/// and/or submitted from an account in this wallet.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TransactionLog {
    /// Unique identifier for the transaction log. This value is not associated
    /// to the ledger, but derived from the tx.
    pub id: String,

    /// Unique identifier for the assigned associated account. If the
    /// transaction is outgoing, this account is from whence the txo came. If
    /// received, this is the receiving account.
    pub account_id: String,

    /// A list of the Txos which were inputs to this transaction.
    pub input_txos: Vec<InputTxo>,

    /// A list of the Txos which were outputs from this transaction.
    pub output_txos: Vec<OutputTxo>,

    /// A list of the Txos which were change in this transaction.
    pub change_txos: Vec<OutputTxo>,

    pub value_map: HashMap<String, String>,

    pub fee_amount: Amount,

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
        value_map: &ValueMap,
    ) -> Self {
        let values = value_map
            .0
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Self {
            id: transaction_log.id.clone(),
            account_id: transaction_log.account_id.clone(),
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
            input_txos: associated_txos.inputs.iter().map(InputTxo::new).collect(),
            output_txos: associated_txos
                .outputs
                .iter()
                .map(|(txo, recipient)| OutputTxo::new(txo, recipient.to_string()))
                .collect(),
            change_txos: associated_txos
                .change
                .iter()
                .map(|(txo, recipient)| OutputTxo::new(txo, recipient.to_string()))
                .collect(),
            value_map: values,
            fee_amount: Amount::from(&transaction_log.fee_amount()),
            sent_time: None,
            comment: transaction_log.comment.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct InputTxo {
    pub txo_id: String,

    /// Amount of this Txo
    pub amount: Amount,
}

impl InputTxo {
    pub fn new(txo: &db::models::Txo) -> Self {
        Self {
            txo_id: txo.id.clone(),
            amount: Amount::from(&txo.amount()),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct OutputTxo {
    pub txo_id_hex: String,

    pub amount: Amount,

    pub recipient_public_address_b58: String,
}

impl OutputTxo {
    pub fn new(txo: &db::models::Txo, recipient_public_address_b58: String) -> Self {
        Self {
            txo_id_hex: txo.id.clone(),
            amount: Amount::from(&txo.amount()),
            recipient_public_address_b58,
        }
    }
}
