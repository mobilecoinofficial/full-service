// Copyright (c) 2020-2023 MobileCoin Inc.

//! API definition for the Txo object.

use crate::service;
use serde::{Deserialize, Serialize};

/// A confirmation number for a Txo in the wallet.
///
/// A confirmation number allows a sender to provide evidence that they were
/// involved in the construction of an associated Txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Confirmation {
    /// Unique identifier for the Txo.
    txo_id: String,

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
            txo_id: src.txo_id.to_string(),
            txo_index: src.txo_index.to_string(),
            confirmation: hex::encode(mc_util_serial::encode(&src.confirmation)),
        }
    }
}
