// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Address object.

use crate::db::models::{AssignedSubaddress, ViewOnlySubaddress};
use serde_derive::{Deserialize, Serialize};

/// An address for an account in the wallet.
///
/// An account may have many addresses. This wallet implementation assumes
/// that an address has been "assigned" to an intended sender. In this way
/// the wallet can make sense of the anonymous MobileCoin ledger, by
/// determining the likely sender of the Txo is whomever was given that
/// address to which to send.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ViewOnlySubaddressJSON {
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
    pub comment: String,

    /// The index of this address in the subaddress space for the account.
    pub subaddress_index: String,

    pub public_spend_key: String,
}

pub type ViewOnlySubaddressesJSON = Vec<ViewOnlySubaddressJSON>;

impl From<&ViewOnlySubaddress> for ViewOnlySubaddressJSON {
    fn from(src: &ViewOnlySubaddress) -> ViewOnlySubaddressJSON {
        ViewOnlySubaddressJSON {
            object: "address".to_string(),
            public_address: src.public_address_b58.clone(),
            account_id: src.view_only_account_id_hex.clone(),
            comment: src.comment.clone(),
            subaddress_index: (src.subaddress_index as u64).to_string(),
            public_spend_key: hex::encode(src.public_spend_key.clone()),
        }
    }
}

impl From<&AssignedSubaddress> for ViewOnlySubaddressJSON {
    fn from(src: &AssignedSubaddress) -> ViewOnlySubaddressJSON {
        ViewOnlySubaddressJSON {
            object: "view_only_subaddress".to_string(),
            public_address: src.assigned_subaddress_b58.clone(),
            account_id: src.account_id_hex.clone(),
            comment: src.comment.clone(),
            subaddress_index: (src.subaddress_index as u64).to_string(),
            public_spend_key: hex::encode(src.subaddress_spend_key.clone()),
        }
    }
}
