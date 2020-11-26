// Copyright (c) 2020 MobileCoin Inc.

use super::schema::accounts;
use serde::Serialize;

#[derive(Clone, Serialize, Queryable, PartialEq, Debug)]
pub struct Account {
    pub account_id_hex: String,
    pub encrypted_account_key: Vec<u8>,
    pub main_subaddress_index: i64,
    pub change_subaddress_index: i64,
    pub next_subaddress_index: i64,
    pub first_block: i64,
    pub next_block: i64,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount<'a> {
    pub account_id_hex: &'a str,
    pub encrypted_account_key: &'a Vec<u8>,
    pub main_subaddress_index: i64,
    pub change_subaddress_index: i64,
    pub next_subaddress_index: i64,
    pub first_block: i64,
    pub next_block: i64,
    pub name: &'a str,
}
