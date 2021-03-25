// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account object.

use mc_crypto_keys::ReprBytes;
use mc_transaction_core::CompressedCommitment;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The encrypted amount of pMOB in a Txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Amount {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// A Pedersen commitment `v*G + s*H`
    pub commitment: String,

    /// The masked value of pMOB in a Txo.
    ///
    /// The private view key is required to decrypt the amount, via:
    /// `masked_value = value XOR_8 Blake2B("value_mask" || shared_secret)`
    pub masked_value: String,
}

impl From<&mc_api::external::Amount> for Amount {
    fn from(src: &mc_api::external::Amount) -> Self {
        Self {
            object: "amount".to_string(),
            commitment: hex::encode(src.get_commitment().get_data()),
            masked_value: src.get_masked_value().to_string(),
        }
    }
}

impl From<&mc_transaction_core::Amount> for Amount {
    fn from(src: &mc_transaction_core::Amount) -> Self {
        Self {
            object: "amount".to_string(),
            commitment: hex::encode(src.commitment.to_bytes()),
            masked_value: src.masked_value.to_string(),
        }
    }
}

impl TryFrom<&Amount> for mc_transaction_core::Amount {
    type Error = String;

    fn try_from(src: &Amount) -> Result<Self, String> {
        let mut commitment_bytes = [0u8; 32];
        commitment_bytes[0..32].copy_from_slice(
            &hex::decode(&src.commitment)
                .map_err(|err| format!("Could not decode hex for amount commitment: {:?}", err))?,
        );
        Ok(Self {
            commitment: CompressedCommitment::from(&commitment_bytes),
            masked_value: src
                .masked_value
                .parse::<u64>()
                .map_err(|err| format!("Could not parse masked value u64: {:?}", err))?,
        })
    }
}
