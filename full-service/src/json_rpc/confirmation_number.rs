// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Txo object.

use crate::service;
use mc_crypto_keys::ReprBytes;
use serde::{Deserialize, Serialize};

/// A confirmation number for a Txo in the wallet.
///
/// A confirmation number allows a sender to provide evidence that they were
/// involved in the construction of an associated Txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Confirmation {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    object: String,

    /// Unique identifier for the Txo.
    txo_id_hex: String,

    /// The index of the Txo in the ledger.
    txo_index: String,

    /// A string with a confirmation number that can be validated to confirm
    /// that another party constructed or had knowledge of the construction
    /// of the associated Txo.
    confirmation: String,
}

impl From<&service::confirmation_number::Confirmation> for Confirmation {
    fn from(src: &service::confirmation_number::Confirmation) -> Confirmation {
        Confirmation {
            object: "confirmation".to_string(),
            txo_id_hex: src.txo_id.to_string(),
            txo_index: src.txo_index.to_string(),
            confirmation: hex::encode(src.confirmation.to_bytes().to_vec().as_slice()),
        }
    }
}
