// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account object.

use mc_transaction_core::TokenId;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The value and token_id of a txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct Amount {
    /// The value of a Txo
    pub value: String,

    /// The token_id of a Txo
    pub token_id: String,
}

impl Amount {
    pub fn new(value: u64, token_id: TokenId) -> Self {
        Self {
            value: value.to_string(),
            token_id: token_id.to_string(),
        }
    }
}

impl From<&mc_transaction_core::Amount> for Amount {
    fn from(src: &mc_transaction_core::Amount) -> Self {
        Self::new(src.value, src.token_id)
    }
}

impl TryFrom<&Amount> for mc_transaction_core::Amount {
    type Error = String;

    fn try_from(src: &Amount) -> Result<Self, String> {
        Ok(Self {
            value: src
                .value
                .parse::<u64>()
                .map_err(|err| format!("Could not parse value u64: {:?}", err))?,
            token_id: TokenId::from(
                src.token_id
                    .parse::<u64>()
                    .map_err(|err| format!("Could not parse token_id u64: {:?}", err))?,
            ),
        })
    }
}
