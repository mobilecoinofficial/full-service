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

    /// Available pico MOB for this account at the current account_block_height.
    /// If the account is syncing, this value may change.
    pub value_pmob: String,

    /// the view-only-account id for this txo
    pub view_only_account_id_hex: String,

    /// Flag that indicates if the the TXO has been manually marked as spent
    pub spent: bool,
}

impl From<&db::models::ViewOnlyTxo> for ViewOnlyTxo {
    fn from(txo: &db::models::ViewOnlyTxo) -> ViewOnlyTxo {
        ViewOnlyTxo {
            object: "view_only_txo".to_string(),
            txo_id_hex: txo.txo_id_hex.clone(),
            value_pmob: (txo.value as u64).to_string(),
            view_only_account_id_hex: txo.view_only_account_id_hex.to_string(),
            spent: txo.spent,
        }
    }
}
