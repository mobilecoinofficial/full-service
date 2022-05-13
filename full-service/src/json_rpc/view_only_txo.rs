// Copyright (c) 2020-2022 MobileCoin Inc.

//! API definition for the Txo object.

use crate::db;
use serde_derive::{Deserialize, Serialize};

/// An View Only Txo in the wallet.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ViewOnlyTxo {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// Unique identifier for the Txo. Constructed from the contents of the
    /// TxOut in the ledger representation.
    pub txo_id_hex: String,

    /// A fingerprint of the txo derived from your private spend key materials,
    /// required to spend a Txo.
    pub key_image: Option<String>,

    pub subaddress_index: Option<String>,

    /// Available pico MOB for this account at the current account_block_height.
    /// If the account is syncing, this value may change.
    pub value_pmob: String,

    /// The public key for this txo, can be used as an identifier to find the
    /// txo in the ledger.
    pub public_key: String,

    /// the view-only-account id for this txo
    pub view_only_account_id_hex: String,

    pub submitted_block_index: Option<String>,

    pub pending_tombstone_block_index: Option<String>,

    pub received_block_index: Option<String>,

    pub spent_block_index: Option<String>,
}

impl From<&db::models::ViewOnlyTxo> for ViewOnlyTxo {
    fn from(txo: &db::models::ViewOnlyTxo) -> ViewOnlyTxo {
        ViewOnlyTxo {
            object: "view_only_txo".to_string(),
            txo_id_hex: txo.txo_id_hex.clone(),
            key_image: txo.key_image.as_ref().map(|k| hex::encode(&k)),
            subaddress_index: txo.subaddress_index.as_ref().map(|i| i.to_string()),
            value_pmob: (txo.value as u64).to_string(),
            public_key: hex::encode(&txo.public_key),
            view_only_account_id_hex: txo.view_only_account_id_hex.to_string(),
            submitted_block_index: txo.submitted_block_index.as_ref().map(|i| i.to_string()),
            pending_tombstone_block_index: txo
                .pending_tombstone_block_index
                .as_ref()
                .map(|i| i.to_string()),
            received_block_index: txo.received_block_index.as_ref().map(|i| i.to_string()),
            spent_block_index: txo.spent_block_index.as_ref().map(|i| i.to_string()),
        }
    }
}
