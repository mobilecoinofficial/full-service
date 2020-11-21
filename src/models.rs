// Copyright (c) 2020 MobileCoin Inc.

use super::schema::accounts;
use serde::Serialize;

#[derive(Clone, Serialize, Queryable, PartialEq, Debug)]
pub struct Account {
    pub account_id_hex: String,
    pub encrypted_account_key: Vec<u8>,
    pub main_subaddress_index: String,
    pub change_subaddress_index: String,
    pub next_subaddress_index: String,
    pub first_block: String,
    pub next_block: String,
    pub name: Option<String>,
}

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount<'a> {
    pub account_id_hex: &'a str,
    pub encrypted_account_key: &'a Vec<u8>,
    pub main_subaddress_index: &'a str,
    pub change_subaddress_index: &'a str,
    pub next_subaddress_index: &'a str,
    pub first_block: &'a str,
    pub next_block: &'a str,
    pub name: Option<&'a str>,
}
