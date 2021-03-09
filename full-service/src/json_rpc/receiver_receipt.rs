// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the ReceiverReceipt object.

use crate::{
    db::{b58_decode, b58_encode},
    service,
};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_transaction_core::tx::TxOutConfirmationNumber;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// An receipt provided from the sender of a transaction for the receiver to use
/// in order to check the status of a transaction.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ReceiverReceipt {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The recipient of this Txo.
    pub recipient: String,

    /// The public key of the Txo sent to the recipient.
    pub txo_public_key: String,

    /// The hash of the Txo sent to the recipient.
    pub txo_hash: String,

    /// The tombstone block for the transaction.
    pub tombstone: String,

    /// The proof for this Txo, which links the sender to this Txo.
    pub proof: String,
}

impl TryFrom<&service::receipt::ReceiverReceipt> for ReceiverReceipt {
    type Error = String;

    fn try_from(src: &service::receipt::ReceiverReceipt) -> Result<ReceiverReceipt, String> {
        Ok(ReceiverReceipt {
            object: "receiver_receipt".to_string(),
            recipient: b58_encode(&src.recipient)
                .map_err(|err| format!("Could not encode public address: {:?}", err))?,
            txo_public_key: hex::encode(&mc_util_serial::encode(&src.txo_public_key)),
            txo_hash: hex::encode(&src.txo_hash),
            tombstone: src.tombstone.to_string(),
            proof: hex::encode(&mc_util_serial::encode(&src.proof)),
        })
    }
}

impl TryFrom<&ReceiverReceipt> for service::receipt::ReceiverReceipt {
    type Error = String;

    fn try_from(src: &ReceiverReceipt) -> Result<service::receipt::ReceiverReceipt, String> {
        let txo_public_key: CompressedRistrettoPublic = mc_util_serial::decode(
            &hex::decode(&src.txo_public_key)
                .map_err(|err| format!("Could not decode hex for txo_public_key: {:?}", err))?,
        )
        .map_err(|err| format!("Could not decode txo public key: {:?}", err))?;
        let proof: TxOutConfirmationNumber = mc_util_serial::decode(
            &hex::decode(&src.proof)
                .map_err(|err| format!("Could not decode hex for proof: {:?}", err))?,
        )
        .map_err(|err| format!("Could not decode proof: {:?}", err))?;
        Ok(service::receipt::ReceiverReceipt {
            recipient: b58_decode(&src.recipient)
                .map_err(|err| format!("Could not decode public address: {:?}", err))?,
            txo_public_key,
            txo_hash: hex::decode(&src.txo_hash)
                .map_err(|err| format!("Could not decode hex for txo_hash: {:?}", err))?,
            tombstone: src
                .tombstone
                .parse::<u64>()
                .map_err(|err| format!("Could not parse u64: {:?}", err))?,
            proof,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_account_keys::AccountKey;
    use mc_crypto_keys::RistrettoPrivate;
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::tx::TxOut;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_rpc_receipt_round_trip() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let account_key = AccountKey::random(&mut rng);
        let public_address = account_key.default_subaddress();
        let txo = TxOut::new(
            rng.next_u64(),
            &public_address,
            &RistrettoPrivate::from_random(&mut rng),
            Default::default(),
        )
        .expect("Could not make TxOut");
        let tombstone = rng.next_u64();
        let mut proof_bytes = [0u8; 32];
        rng.fill_bytes(&mut proof_bytes);
        let confirmation_number = TxOutConfirmationNumber::from(proof_bytes);

        let service_receipt = service::receipt::ReceiverReceipt {
            recipient: public_address,
            txo_public_key: txo.public_key,
            txo_hash: txo.hash().to_vec(),
            tombstone,
            proof: confirmation_number,
        };

        let json_rpc_receipt = ReceiverReceipt::try_from(&service_receipt)
            .expect("Could not get json receipt from service receipt");

        let service_receipt_from_json =
            service::receipt::ReceiverReceipt::try_from(&json_rpc_receipt)
                .expect("Could not get receipt from json");

        assert_eq!(service_receipt, service_receipt_from_json);
    }
}
