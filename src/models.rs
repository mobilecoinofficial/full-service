// Copyright (c) 2020 MobileCoin Inc.

//! DB Models

use super::schema::{account_txo_statuses, accounts, txos};
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
pub struct AccountTxoStatuses {
    pub account_id_hex: String,
    pub txo_id_hex: String,
    pub txo_status: String,
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
