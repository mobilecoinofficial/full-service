// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TransactionLog object.

use redact::{expose_secret, Secret};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    db,
    db::transaction_log::{AssociatedTxos, TransactionLogModel},
};

pub enum TxStatus {
    Built,
    Pending,
    Succeeded,
    Failed,
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxStatus::Built => write!(f, "tx_status_built"),
            TxStatus::Pending => write!(f, "tx_status_pending"),
            TxStatus::Succeeded => write!(f, "tx_status_succeeded"),
            TxStatus::Failed => write!(f, "tx_status_failed"),
        }
    }
}

impl From<&db::transaction_log::TxStatus> for TxStatus {
    fn from(tx_status: &db::transaction_log::TxStatus) -> Self {
        match tx_status {
            db::transaction_log::TxStatus::Built => TxStatus::Built,
            db::transaction_log::TxStatus::Signed => TxStatus::Built,
            db::transaction_log::TxStatus::Pending => TxStatus::Pending,
            db::transaction_log::TxStatus::Succeeded => TxStatus::Succeeded,
            db::transaction_log::TxStatus::Failed => TxStatus::Failed,
        }
    }
}

impl From<&TxStatus> for db::transaction_log::TxStatus {
    fn from(tx_status: &TxStatus) -> Self {
        match tx_status {
            TxStatus::Built => db::transaction_log::TxStatus::Built,
            TxStatus::Pending => db::transaction_log::TxStatus::Pending,
            TxStatus::Succeeded => db::transaction_log::TxStatus::Pending,
            TxStatus::Failed => db::transaction_log::TxStatus::Pending,
        }
    }
}

pub enum TxDirection {
    Received,
    Sent,
}

impl fmt::Display for TxDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxDirection::Received => write!(f, "tx_direction_received"),
            TxDirection::Sent => write!(f, "tx_direction_sent"),
        }
    }
}

/// A log of a transaction that occurred on the MobileCoin network, constructed
/// and/or submitted from an account in this wallet.
#[derive(Deserialize, PartialEq, Serialize, Default, Debug, Clone)]
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
    #[serde(serialize_with = "expose_secret")]
    pub value_pmob: Secret<String>,

    /// Fee in pico MOB associated to this transaction log. Only on outgoing
    /// transaction logs. Only available if direction is "sent".
    #[serde(serialize_with = "expose_secret")]
    pub fee_pmob: Secret<Option<String>>,

    /// The block index of the highest block on the network at the time the
    /// transaction was submitted.
    #[serde(serialize_with = "expose_secret")]
    pub submitted_block_index: Secret<Option<String>>,

    ///  The scanned block block index in which this transaction occurred.
    #[serde(serialize_with = "expose_secret")]
    pub finalized_block_index: Secret<Option<String>>,

    /// String representing the transaction log status. On "sent", valid
    /// statuses are "built", "pending", "succeeded", "failed".  On "received",
    /// the status is "succeeded".
    #[serde(serialize_with = "expose_secret")]
    pub status: Secret<String>,

    /// Time at which sent transaction log was created. Only available if
    /// direction is "sent". This value is null if "received" or if the sent
    /// transactions were recovered from the ledger (is_sent_recovered = true).
    #[serde(serialize_with = "expose_secret")]
    pub sent_time: Secret<Option<String>>,

    /// An arbitrary string attached to the object.
    pub comment: String,

    /// Code representing the cause of "failed" status.
    pub failure_code: Option<i32>,

    /// Human parsable explanation of "failed" status.
    pub failure_message: Option<String>,
}

impl TransactionLog {
    pub fn new_from_received_txo(
        txo: &db::models::Txo,
        assigned_address: Option<String>,
    ) -> Result<Self, String> {
        Ok(TransactionLog {
            object: "transaction_log".to_string(),
            transaction_log_id: txo.id.clone(),
            direction: TxDirection::Received.to_string(),
            is_sent_recovered: None,
            account_id: txo
                .clone()
                .account_id
                .ok_or("Txo has no account_id but it is required for a transaction log")?,
            input_txos: vec![],
            output_txos: vec![TxoAbbrev {
                txo_id_hex: txo.id.to_string(),
                recipient_address_id: "".to_string(),
                value_pmob: (txo.value as u64).to_string().into(),
                public_key: hex::encode(
                    txo.public_key()
                        .map_err(|_| "failed to decode txo public key")?
                        .as_bytes(),
                ),
            }],
            change_txos: vec![],
            assigned_address_id: assigned_address,
            value_pmob: (txo.value as u64).to_string().into(),
            fee_pmob: Secret::new(None),
            submitted_block_index: Secret::new(None),
            finalized_block_index: Secret::new(
                txo.received_block_index.map(|index| index.to_string()),
            ),
            status: TxStatus::Succeeded.to_string().into(),
            sent_time: Secret::new(None),
            comment: "".to_string(),
            failure_code: None,
            failure_message: None,
        })
    }

    pub fn new(
        transaction_log: &db::models::TransactionLog,
        associated_txos: &AssociatedTxos,
    ) -> Self {
        let input_txos: Vec<TxoAbbrev> = associated_txos
            .inputs
            .iter()
            .map(|txo| TxoAbbrev::new(txo, "".to_string()))
            .collect();

        let output_txos: Vec<TxoAbbrev> = associated_txos
            .outputs
            .iter()
            .map(|(txo, recipient_address_id)| TxoAbbrev::new(txo, recipient_address_id.clone()))
            .collect();

        let value_pmob = associated_txos
            .outputs
            .iter()
            .map(|(txo, _)| txo.value as u64)
            .sum::<u64>()
            .to_string();

        let change_txos: Vec<TxoAbbrev> = associated_txos
            .change
            .iter()
            .map(|(txo, recipient_address_id)| TxoAbbrev::new(txo, recipient_address_id.clone()))
            .collect();

        let assigned_address_id = output_txos
            .first()
            .map(|txo| txo.recipient_address_id.clone());

        Self {
            object: "transaction_log".to_string(),
            transaction_log_id: transaction_log.id.clone(),
            direction: TxDirection::Sent.to_string(),
            is_sent_recovered: None,
            account_id: transaction_log.account_id.clone(),
            input_txos,
            output_txos,
            change_txos,
            assigned_address_id,
            value_pmob: value_pmob.into(),
            fee_pmob: Secret::new(Some(transaction_log.fee_value.to_string())),
            submitted_block_index: Secret::new(
                transaction_log.submitted_block_index.map(|i| i.to_string()),
            ),
            finalized_block_index: Secret::new(
                transaction_log.finalized_block_index.map(|i| i.to_string()),
            ),
            status: TxStatus::from(&transaction_log.status()).to_string().into(),
            sent_time: Secret::new(None),
            comment: transaction_log.comment.clone(),
            failure_code: None,
            failure_message: None,
        }
    }
}

#[derive(Deserialize, PartialEq, Serialize, Default, Debug, Clone)]
pub struct TxoAbbrev {
    pub txo_id_hex: String,

    /// Unique identifier for the recipient associated account. Blank unless
    /// direction is "sent".
    pub recipient_address_id: String,

    /// Available pico MOB for this Txo.
    /// If the account is syncing, this value may change.
    #[serde(serialize_with = "expose_secret")]
    pub value_pmob: Secret<String>,

    /// Hex encoded bytes of the public key of this txo, which can be found on
    /// the blockchain.
    pub public_key: String,
}

impl TxoAbbrev {
    pub fn new(txo: &db::models::Txo, recipient_address_id: String) -> Self {
        let public_key_hex = match txo.public_key() {
            Ok(pk) => hex::encode(pk.as_bytes()),
            Err(_) => "".to_string(),
        };

        Self {
            txo_id_hex: txo.id.clone(),
            recipient_address_id,
            value_pmob: (txo.value as u64).to_string().into(),
            public_key: public_key_hex,
        }
    }
}
