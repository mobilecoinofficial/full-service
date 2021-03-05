// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Txo object.

use crate::service;
use serde::{Deserialize, Serialize};

/// An Proof for a Txo in the wallet.
///
/// A proof allows a sender to provide evidence that they were involved in the
/// construction of an associated Txo.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Proof {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    object: String,

    /// Unique identifier for the Txo.
    txo_id: String,

    /// The index of the Txo in the ledger.
    txo_index: String,

    /// A string with a proof that can be verified to confirm that another party
    /// constructed or had knowledge of the construction of the associated Txo.
    proof: String,
}

impl From<&service::proof::Proof> for Proof {
    fn from(src: &service::proof::Proof) -> Proof {
        Proof {
            object: "proof".to_string(),
            txo_id: src.txo_id.to_string(),
            txo_index: src.txo_index.to_string(),
            proof: hex::encode(mc_util_serial::encode(&src.proof)),
        }
    }
}
