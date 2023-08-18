// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account object.

use mc_transaction_core::TokenId;
use redact::{expose_secret, Secret};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The value and token_id of a txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct Amount {
    /// The value of a Txo
    #[serde(serialize_with = "expose_secret")]
    pub value: Secret<String>,

    /// The token_id of a Txo
    #[serde(serialize_with = "expose_secret")]
    pub token_id: Secret<String>,
}

impl Amount {
    pub fn new(value: u64, token_id: TokenId) -> Self {
        Self {
            value: Secret::new(value.to_string()),
            token_id: Secret::new(token_id.to_string()),
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
                .expose_secret()
                .parse::<u64>()
                .map_err(|err| format!("Could not parse value u64: {err:?}"))?,
            token_id: TokenId::from(
                src.token_id
                    .expose_secret()
                    .parse::<u64>()
                    .map_err(|err| format!("Could not parse token_id u64: {err:?}"))?,
            ),
        })
    }
}
