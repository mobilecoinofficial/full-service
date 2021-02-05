// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::db::models::AssignedSubaddress;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct JsonAddress {
    pub object: String,
    pub address_id: String,
    pub public_address: String,
    pub account_id: String,
    pub address_book_entry_id: Option<String>,
    pub comment: String,
    pub subaddress_index: String,
    pub offset_count: i32,
}

impl JsonAddress {
    pub fn new(assigned_subaddress: &AssignedSubaddress) -> Self {
        Self {
            object: "assigned_address".to_string(),
            address_id: assigned_subaddress.assigned_subaddress_b58.clone(),
            account_id: assigned_subaddress.account_id_hex.to_string(),
            public_address: assigned_subaddress.assigned_subaddress_b58.clone(),
            address_book_entry_id: assigned_subaddress
                .address_book_entry
                .map(|x| x.to_string()),
            comment: assigned_subaddress.comment.clone(),
            subaddress_index: assigned_subaddress.subaddress_index.to_string(),
            offset_count: assigned_subaddress.id,
        }
    }
}
