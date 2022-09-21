// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account object.

use mc_crypto_keys::ReprBytes;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MaskedAmountVersion {
    V1,
    V2,
}

impl Default for MaskedAmountVersion {
    fn default() -> Self {
        MaskedAmountVersion::V1
    }
}

/// The encrypted amount of pMOB in a Txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct MaskedAmount {
    /// A Pedersen commitment `v*G + s*H`
    pub commitment: String,

    /// The masked value of pMOB in a Txo.
    ///
    /// The private view key is required to decrypt the amount, via:
    /// `masked_value = value XOR_8 Blake2B("value_mask" || shared_secret)`
    pub masked_value: String,

    /// `masked_token_id = token_id XOR_8 Blake2B(token_id_mask |
    /// shared_secret)` 8 bytes long when used, 0 bytes for older amounts
    /// that don't have this.
    pub masked_token_id: String,

    /// The version of the masked amount.
    pub version: MaskedAmountVersion,
}

impl From<&mc_transaction_core::MaskedAmount> for MaskedAmount {
    fn from(src: &mc_transaction_core::MaskedAmount) -> Self {
        let version = match src {
            mc_transaction_core::MaskedAmount::V1(_) => MaskedAmountVersion::V1,
            mc_transaction_core::MaskedAmount::V2(_) => MaskedAmountVersion::V2,
        };

        Self {
            commitment: hex::encode(src.commitment().to_bytes()),
            masked_value: src.get_masked_value().to_string(),
            masked_token_id: hex::encode(&src.masked_token_id()),
            version,
        }
    }
}

impl TryFrom<&MaskedAmount> for mc_transaction_core::MaskedAmount {
    type Error = String;

    fn try_from(src: &MaskedAmount) -> Result<Self, String> {
        let mut commitment_bytes = [0u8; 32];
        commitment_bytes[0..32].copy_from_slice(
            &hex::decode(&src.commitment)
                .map_err(|err| format!("Could not decode hex for amount commitment: {:?}", err))?,
        );

        let commitment = (&commitment_bytes).into();
        let masked_value = src
            .masked_value
            .parse::<u64>()
            .map_err(|err| format!("Could not parse masked value u64: {:?}", err))?;
        let masked_token_id = hex::decode(&src.masked_token_id)
            .map_err(|err| format!("Could not decode hex for masked token id: {:?}", err))?;

        match src.version {
            MaskedAmountVersion::V1 => {
                let masked_amount = mc_transaction_core::MaskedAmountV1 {
                    commitment,
                    masked_value,
                    masked_token_id,
                };

                Ok(mc_transaction_core::MaskedAmount::V1(masked_amount))
            }
            MaskedAmountVersion::V2 => {
                let masked_amount = mc_transaction_core::MaskedAmountV2 {
                    commitment,
                    masked_value,
                    masked_token_id,
                };

                Ok(mc_transaction_core::MaskedAmount::V2(masked_amount))
            }
        }
    }
}
