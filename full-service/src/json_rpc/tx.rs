// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the GiftCode object.


use serde::{Deserialize, Serialize};

/// An gift code created by this wallet to share.
///
/// A gift code is a self-contained account which has been funded with MOB.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Tx {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,
}

/*
pub fn tx_hash(&self) -> TxHash {
        TxHash::from(self.digest32::<MerlinTranscript>(b"mobilecoin-tx"))
    }

    /// Key images "spent" by this transaction.
    pub fn key_images(&self) -> Vec<KeyImage> {
        self.signature.key_images()
    }

    /// Get the highest index of each membership proof referenced by the transaction.
    pub fn get_membership_proof_highest_indices(&self) -> Vec<u64> {
        self.prefix.get_membership_proof_highest_indices()
    }

    /// Output public keys contained in this transaction.
    pub fn output_public_keys(&self) -> Vec<CompressedRistrettoPublic> {
        self.prefix
            .outputs
            .iter()
            .map(|tx_out| tx_out.public_key)
            .collect()
    }
*/


impl From<&mc_transaction_core::tx::Tx> for Tx {
    fn from(src: &mc_transaction_core::tx::Tx) -> Tx {
        Tx {
            object: "tx".to_string(),
        }
    }
}
