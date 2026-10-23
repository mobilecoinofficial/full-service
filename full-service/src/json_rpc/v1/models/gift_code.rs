// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the GiftCode object.

use crate::service::gift_code::DecodedGiftCode;

use redact::{expose_secret, Secret};
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
    #[serde(serialize_with = "expose_secret")]
    pub gift_code_b58: Secret<String>,

    /// The root entropy for the account in this gift code.
    #[serde(serialize_with = "expose_secret")]
    pub root_entropy: Secret<String>,

    /// The entropy mnemonic for the account in this gift code.
    #[serde(serialize_with = "expose_secret")]
    pub bip39_entropy: Secret<String>,

    /// The amount of MOB contained in the gift code account.
    pub value_pmob: String,

    /// A memo associated with this gift code.
    pub memo: String,
}

impl From<&DecodedGiftCode> for GiftCode {
    fn from(src: &DecodedGiftCode) -> GiftCode {
        GiftCode {
            object: "gift_code".to_string(),
            gift_code_b58: src.gift_code_b58.clone().into(),
            root_entropy: src
                .root_entropy
                .as_ref()
                .map(hex::encode)
                .unwrap_or_default()
                .into(),
            bip39_entropy: src
                .bip39_entropy
                .as_ref()
                .map(hex::encode)
                .unwrap_or_default()
                .into(),
            value_pmob: src.value.to_string(),
            memo: src.memo.clone(),
        }
    }
}
