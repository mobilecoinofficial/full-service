// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the ReceiverReceipt object.

use crate::{json_rpc::v2::models::masked_amount::MaskedAmount, service};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_transaction_extra::TxOutConfirmationNumber;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// An receipt provided from the sender of a transaction for the receiver to use
/// in order to check the status of a transaction.
///
/// Note: This should stay in line wth the Receipt defined in external.proto
/// https://github.com/mobilecoinfoundation/mobilecoin/blob/master/api/proto/external.proto#L255
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ReceiverReceipt {
    /// The public key of the Txo sent to the recipient.
    pub public_key: String,

    /// The confirmation proof for this Txo, which links the sender to this Txo.
    pub confirmation: String,

    /// The tombstone block for the transaction.
    pub tombstone_block: String,

    /// The amount of the Txo.
    /// Note: This value is self-reported by the sender and is unverifiable.
    pub amount: MaskedAmount,
}

impl TryFrom<&service::receipt::ReceiverReceipt> for ReceiverReceipt {
    type Error = String;

    fn try_from(src: &service::receipt::ReceiverReceipt) -> Result<ReceiverReceipt, String> {
        Ok(ReceiverReceipt {
            public_key: hex::encode(&mc_util_serial::encode(&src.public_key)),
            tombstone_block: src.tombstone_block.to_string(),
            confirmation: hex::encode(&mc_util_serial::encode(&src.confirmation)),
            amount: MaskedAmount::from(&src.amount),
        })
    }
}

impl TryFrom<&ReceiverReceipt> for service::receipt::ReceiverReceipt {
    type Error = String;

    fn try_from(src: &ReceiverReceipt) -> Result<service::receipt::ReceiverReceipt, String> {
        let txo_public_key: CompressedRistrettoPublic = mc_util_serial::decode(
            &hex::decode(&src.public_key)
                .map_err(|err| format!("Could not decode hex for txo_public_key: {err:?}"))?,
        )
        .map_err(|err| format!("Could not decode txo public key: {err:?}"))?;

        let proof: TxOutConfirmationNumber = mc_util_serial::decode(
            &hex::decode(&src.confirmation)
                .map_err(|err| format!("Could not decode hex for proof: {err:?}"))?,
        )
        .map_err(|err| format!("Could not decode proof: {err:?}"))?;

        let amount = mc_transaction_core::MaskedAmount::try_from(&src.amount)
            .map_err(|err| format!("Could not convert amount: {err:?}"))?;

        Ok(service::receipt::ReceiverReceipt {
            public_key: txo_public_key,
            tombstone_block: src
                .tombstone_block
                .parse::<u64>()
                .map_err(|err| format!("Could not parse u64: {err:?}"))?,
            confirmation: proof,
            amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_account_keys::AccountKey;
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::{tokens::Mob, tx::TxOut, Amount, Token};
    use mc_transaction_types::BlockVersion;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_rpc_receipt_round_trip() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let account_key = AccountKey::random(&mut rng);
        let public_address = account_key.default_subaddress();
        let txo = TxOut::new(
            BlockVersion::MAX,
            Amount::new(rng.next_u64(), Mob::ID),
            &public_address,
            &RistrettoPrivate::from_random(&mut rng),
            Default::default(),
        )
        .expect("Could not make TxOut");
        let tombstone = rng.next_u64();
        let mut proof_bytes = [0u8; 32];
        rng.fill_bytes(&mut proof_bytes);
        let confirmation_number = TxOutConfirmationNumber::from(proof_bytes);
        let amount = mc_transaction_core::MaskedAmount::new(
            BlockVersion::MAX,
            Amount::new(rng.next_u64(), Mob::ID),
            &RistrettoPublic::from_random(&mut rng),
        )
        .expect("Could not create amount");

        let service_receipt = service::receipt::ReceiverReceipt {
            public_key: txo.public_key,
            tombstone_block: tombstone,
            confirmation: confirmation_number,
            amount,
        };

        let json_rpc_receipt = ReceiverReceipt::try_from(&service_receipt)
            .expect("Could not get json receipt from service receipt");

        let service_receipt_from_json =
            service::receipt::ReceiverReceipt::try_from(&json_rpc_receipt)
                .expect("Could not get receipt from json");

        assert_eq!(service_receipt, service_receipt_from_json);
    }
}
