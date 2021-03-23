// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Tx object.

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

impl From<&mc_transaction_core::tx::Tx> for Tx {
    fn from(_src: &mc_transaction_core::tx::Tx) -> Tx {
        Tx {
            object: "tx".to_string(),
        }
    }
}
