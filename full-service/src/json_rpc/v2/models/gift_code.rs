// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the GiftCode object.

use crate::service::gift_code::DecodedGiftCode;

use serde::{Deserialize, Serialize};

/// An gift code created by this wallet to share.
///
/// A gift code is a self-contained account which has been funded with MOB.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GiftCode {
    /// The base58-encoded gift code string to share.
    pub gift_code_b58: String,

    /// The root entropy for the account in this gift code.
    pub root_entropy: String,

    /// The entropy mnemonic for the account in this gift code.
    pub bip39_entropy: String,

    /// The amount of MOB contained in the gift code account.
    pub value_pmob: String,

    /// A memo associated with this gift code.
    pub memo: String,
}

impl From<&DecodedGiftCode> for GiftCode {
    fn from(src: &DecodedGiftCode) -> GiftCode {
        GiftCode {
            gift_code_b58: src.gift_code_b58.clone(),
            root_entropy: src
                .root_entropy
                .as_ref()
                .map(hex::encode)
                .unwrap_or_default(),
            bip39_entropy: src
                .bip39_entropy
                .as_ref()
                .map(hex::encode)
                .unwrap_or_default(),
            value_pmob: src.value.to_string(),
            memo: src.memo.clone(),
        }
    }
}
