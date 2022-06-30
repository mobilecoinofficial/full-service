// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB Models

use super::schema::{
    accounts, assigned_subaddresses, gift_codes, transaction_logs, transaction_txos, txos,
};

use serde::Serialize;

/// An Account entity.
///
/// Contains the account private keys, subaddress configuration, and ...
#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(id)]
pub struct Account {
    /// Primary key
    pub id: i32,
    /// An additional ID, derived from the account data.
    pub account_id_hex: String,
    pub account_key: Vec<u8>,
    pub entropy: Option<Vec<u8>>,
    pub key_derivation_version: i32,
    /// Default subadress that is given out to refer to this account.
    pub main_subaddress_index: i64,
    /// Subaddress used to return transaction "change" to self.
    pub change_subaddress_index: i64,
    /// The next unused subaddress index. (Assumes indices are used sequentially
    /// from 0).
    pub next_subaddress_index: i64,
    /// Index of the first block where this account may have held funds.
    pub first_block_index: i64,
    /// Index of the next block to inspect for transactions related to this
    /// account.
    pub next_block_index: i64,
    /// If the account was imported, account history prior to this block index
    /// is derived from the public ledger, and does not reflect client-side
    /// user events.
    pub import_block_index: Option<i64>,
    /// Name of this account.
    pub name: String, /* empty string for nullable */
    pub fog_enabled: bool,
    pub view_only: bool,
}

/// A structure that can be inserted to create a new entity in the `accounts`
/// table.
#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount<'a> {
    pub account_id_hex: &'a str,
    pub account_key: &'a [u8],
    pub entropy: Option<&'a [u8]>,
    pub key_derivation_version: i32,
    pub main_subaddress_index: i64,
    pub change_subaddress_index: i64,
    pub next_subaddress_index: i64,
    pub first_block_index: i64,
    pub next_block_index: i64,
    pub import_block_index: Option<i64>,
    pub name: &'a str,
    pub fog_enabled: bool,
    pub view_only: bool,
}

/// A transaction output entity that either was received to an Account in this
/// wallet, or originated from an Account in this wallet. A transaction
/// output can be in one of many states with respect to multiple accounts.
/// Managing these relationships and states is one of the main goals of
/// the Full-Service wallet.
#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(id)]
pub struct Txo {
    /// Primary key derived from the contents of the ledger TxOut
    pub id: String,
    pub account_id_hex: Option<String>,
    /// The value of this transaction output, in picoMob.
    pub value: i64,
    /// The token of this transaction output.
    pub token_id: i64,
    /// The serialized target_key of the TxOut.
    pub target_key: Vec<u8>,
    /// The serialized public_key of the TxOut.
    pub public_key: Vec<u8>,
    /// The serialized e_fog_hint of the TxOut.
    pub e_fog_hint: Vec<u8>,
    /// The serialized TxOut.
    pub txo: Vec<u8>,
    /// The receiving subaddress, if known.
    pub subaddress_index: Option<i64>,
    /// Pre-computed key image for this Txo, or None if the Txo is orphaned.
    pub key_image: Option<Vec<u8>>,
    /// Block index containing this Txo.
    pub received_block_index: Option<i64>,
    pub spent_block_index: Option<i64>,
    pub shared_secret: Option<Vec<u8>>,
}

/// A structure that can be inserted to create a new entity in the `txos` table.
#[derive(Insertable)]
#[table_name = "txos"]
pub struct NewTxo<'a> {
    pub id: &'a str,
    pub account_id_hex: Option<String>,
    pub value: i64,
    pub token_id: i64,
    pub target_key: &'a [u8],
    pub public_key: &'a [u8],
    pub e_fog_hint: &'a [u8],
    pub txo: &'a [u8],
    pub subaddress_index: Option<i64>,
    pub key_image: Option<&'a [u8]>,
    pub received_block_index: Option<i64>,
    pub spent_block_index: Option<i64>,
    pub shared_secret: Option<&'a [u8]>,
}

/// A subaddress given to a particular contact, for the purpose of tracking
/// funds received from that contact.
#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(Account, foreign_key = "account_id_hex")]
#[primary_key(id)]
#[table_name = "assigned_subaddresses"]
pub struct AssignedSubaddress {
    pub id: i32,
    pub assigned_subaddress_b58: String,
    pub account_id_hex: String,
    pub address_book_entry: Option<i64>,
    pub public_address: Vec<u8>,
    pub subaddress_index: i64,
    pub comment: String,               // empty string for nullable
    pub subaddress_spend_key: Vec<u8>, // FIXME: WS-28 - Index on subaddress_spend_key?
}

/// A structure that can be inserted to create a new AssignedSubaddress entity.
#[derive(Insertable)]
#[table_name = "assigned_subaddresses"]
pub struct NewAssignedSubaddress<'a> {
    pub assigned_subaddress_b58: &'a str,
    pub account_id_hex: &'a str,
    pub address_book_entry: Option<i64>,
    pub public_address: &'a [u8],
    pub subaddress_index: i64,
    pub comment: &'a str,
    pub subaddress_spend_key: &'a [u8],
}

/// The status of a sent transaction OR a received transaction output.
#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(Account, foreign_key = "account_id_hex")]
#[primary_key(id)]
#[table_name = "transaction_logs"]
pub struct TransactionLog {
    pub id: String,
    pub account_id_hex: String,
    pub fee_value: i64,
    pub fee_token_id: i64,
    pub submitted_block_index: Option<i64>,
    pub tombstone_block_index: Option<i64>,
    pub finalized_block_index: Option<i64>,
    pub comment: String,
    pub tx: Vec<u8>,
    pub failed: bool,
}

/// A structure that can be inserted to create a new TransactionLog entity.
#[derive(Insertable)]
#[table_name = "transaction_logs"]
pub struct NewTransactionLog<'a> {
    pub id: &'a str,
    pub account_id_hex: &'a str,
    pub fee_value: i64,
    pub fee_token_id: i64,
    pub submitted_block_index: Option<i64>,
    pub tombstone_block_index: Option<i64>,
    pub finalized_block_index: Option<i64>,
    pub comment: &'a str,
    pub tx: &'a [u8],
    pub failed: bool,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(TransactionLog, foreign_key = "transaction_log_id")]
#[belongs_to(Txo, foreign_key = "txo_id")]
#[table_name = "transaction_txos"]
#[primary_key(transaction_log_id, txo_id)]
pub struct TransactionTxo {
    pub transaction_log_id: String,
    pub txo_id: String,
    pub used_as: String,
}

#[derive(Insertable)]
#[table_name = "transaction_txos"]
pub struct NewTransactionTxo<'a> {
    pub transaction_log_id: &'a str,
    pub txo_id: &'a str,
    pub used_as: &'a str,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(Account, foreign_key = "id")]
#[belongs_to(TransactionLog, foreign_key = "id")]
#[table_name = "gift_codes"]
#[primary_key(id)]
pub struct GiftCode {
    pub id: i32,
    pub gift_code_b58: String,
    pub value: i64,
}

#[derive(Insertable)]
#[table_name = "gift_codes"]
pub struct NewGiftCode<'a> {
    pub gift_code_b58: &'a str,
    pub value: i64,
}
