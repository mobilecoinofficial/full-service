// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the GiftCode object.

use crate::db;

use serde::{Deserialize, Serialize};

/// An gift code created by this wallet to share.
///
/// A gift code is a self-contained account which has been funded with MOB.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GiftCode {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The base58-encoded gift code string to share.
    pub gift_code_b58: String,

    /// The entropy for the account in this gift code.
    pub entropy: String,

    /// The amount of MOB contained in the gift code account.
    pub value_pmob: String,

    /// A memo associated with this gift code.
    pub memo: String,

    /// The account ID of the ephemeral account in this wallet which holds the
    /// Gift Code funds.
    pub account_id: String,

    /// The Txo ID of the Txo in the Gift Code.
    pub txo_id: String,
}

impl From<&db::models::GiftCode> for GiftCode {
    fn from(src: &db::models::GiftCode) -> GiftCode {
        GiftCode {
            object: "gift_code".to_string(),
            gift_code_b58: src.gift_code_b58.clone(),
            entropy: hex::encode(&src.entropy),
            value_pmob: src.value.to_string(),
            memo: src.memo.clone(),
            account_id: src.account_id_hex.to_string(),
            txo_id: src.txo_id_hex.to_string(),
        }
    }
}
