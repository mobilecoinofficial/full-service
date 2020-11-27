// Copyright (c) 2020 MobileCoin Inc.

//! DB Models

use super::schema::{account_txo_statuses, accounts, assigned_subaddresses, txos};
use serde::Serialize;

#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(account_id_hex)]
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

#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(txo_id_hex)]
pub struct Txo {
    pub txo_id_hex: String,
    pub value: i64,
    pub target_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub e_fog_hint: Vec<u8>,
    pub subaddress_index: i64,
    pub key_image: Option<Vec<u8>>,
    pub received_block_height: Option<i64>,
    pub spent_tombstone_block_height: Option<i64>,
    pub spent_block_height: Option<i64>,
    pub proof: Option<Vec<u8>>,
}

#[derive(Insertable)]
#[table_name = "txos"]
pub struct NewTxo<'a> {
    pub txo_id_hex: &'a str,
    pub value: i64,
    pub target_key: &'a Vec<u8>,
    pub public_key: &'a Vec<u8>,
    pub e_fog_hint: &'a Vec<u8>,
    pub subaddress_index: i64,
    pub key_image: Option<&'a Vec<u8>>,
    pub received_block_height: Option<i64>,
    pub spent_tombstone_block_height: Option<i64>,
    pub spent_block_height: Option<i64>,
    pub proof: Option<&'a Vec<u8>>,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(Account, foreign_key = "account_id_hex")]
#[belongs_to(Txo, foreign_key = "txo_id_hex")]
#[table_name = "account_txo_statuses"]
#[primary_key(account_id_hex, txo_id_hex)]
pub struct AccountTxoStatus {
    pub account_id_hex: String,
    pub txo_id_hex: String,
    // Statuses: unspent, pending, spent, unknown
    pub txo_status: String,
    // Types: minted, received
    pub txo_type: String,
}

#[derive(Insertable)]
#[table_name = "account_txo_statuses"]
pub struct NewAccountTxoStatus<'a> {
    pub account_id_hex: &'a str,
    pub txo_id_hex: &'a str,
    pub txo_status: &'a str,
    pub txo_type: &'a str,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(Account, foreign_key = "account_id_hex")]
#[primary_key(assigned_subaddress_b58)]
#[table_name = "assigned_subaddresses"]
pub struct AssignedSubaddress {
    pub assigned_subaddress_b58: String,
    pub account_id_hex: String,
    pub address_book_entry: Option<i64>,
    pub public_address: Vec<u8>,
    pub subaddress_index: i64,
    pub comment: String,
    pub expected_value: Option<i64>,
    pub subaddress_spend_key: Vec<u8>, // FIXME: should we be indexing on this col? We do a lot of lookups by this
}

#[derive(Insertable)]
#[table_name = "assigned_subaddresses"]
pub struct NewAssignedSubaddress<'a> {
    pub assigned_subaddress_b58: &'a str,
    pub account_id_hex: &'a str,
    pub address_book_entry: Option<i64>,
    pub public_address: &'a Vec<u8>,
    pub subaddress_index: i64,
    pub comment: &'a str,
    pub expected_value: Option<i64>,
    pub subaddress_spend_key: &'a Vec<u8>,
}
