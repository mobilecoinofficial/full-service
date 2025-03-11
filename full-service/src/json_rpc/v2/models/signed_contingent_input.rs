// Copyright (c) 2020-2025 MobileCoin Inc.

//! API definition related to SCIs.

use super::amount::Amount;
use serde::{Deserialize, Serialize};

/// A result of a call to the validate_proof_of_reserve_sci JSON-RPC method.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "result")]
pub enum ValidateProofOfReserveSciResult {
    /// The signed contingent input is valid.
    Valid {
        /// Hex-encoded tx out public key.
        tx_out_public_key: String,

        /// Hex-encoded key image.
        key_image: String,

        /// Amount.
        amount: Amount,
    },

    /// The SCI validate method failed.
    InvalidSci { error: String },

    /// The SCI is valid but is not a proof of reserve SCI.
    NotProofOfReserveSci { error: String },

    /// The SCI is valid but the TxOut is not found in the ledger.
    TxOutNotFoundInLedger {
        /// Hex-encoded tx out public key.
        tx_out_public_key: String,
    },

    /// The TxOut in the SCI does not match the TxOut in the ledger.
    TxOutMismatch {
        /// Hex-encoded tx out public key.
        tx_out_public_key: String,
    },
}
