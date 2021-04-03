// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Address object.

use crate::db::models::AssignedSubaddress;
use serde_derive::{Deserialize, Serialize};

/// An address for an account in the wallet.
///
/// An account may have many addresses. This wallet implementation assumes
/// that an address has been "assigned" to an intended sender. In this way
/// the wallet can make sense of the anonymous MobileCoin ledger, by
/// determining the likely sender of the Txo is whomever was given that
/// address to which to send.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Address {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// A b58 encoding of the public address materials.
    ///
    /// The public_address is the unique identifier for the address.
    pub public_address: String,

    /// The account which owns this address.
    pub account_id: String,

    /// Additional data associated with this address.
    pub metadata: String,

    /// The index of this address in the subaddress space for the account.
    pub subaddress_index: String,

    /// The offset in the database (used for pagination).
    pub offset_count: String,
}

impl From<&AssignedSubaddress> for Address {
    fn from(src: &AssignedSubaddress) -> Address {
        Address {
            object: "address".to_string(),
            public_address: src.assigned_subaddress_b58.clone(),
            account_id: src.account_id_hex.clone(),
            metadata: src.comment.clone(),
            subaddress_index: (src.subaddress_index as u64).to_string(),
            offset_count: src.id.to_string(),
        }
    }
}
