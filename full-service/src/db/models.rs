// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB Models

use super::schema::{
    __diesel_schema_migrations, accounts, assigned_subaddresses, authenticated_sender_memos,
    gift_codes, post_migration_processes, transaction_input_txos, transaction_logs,
    transaction_output_txos, txos,
};

use mc_crypto_keys::CompressedRistrettoPublic;

use serde::Serialize;

/// An Account entity.
///
/// Contains the account private keys, subaddress configuration, and ...
#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = accounts)]
pub struct Account {
    /// Primary key, derived from the account data.
    pub id: String,
    pub account_key: Vec<u8>,
    pub entropy: Option<Vec<u8>>,
    pub key_derivation_version: i32,
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
    /// If true, this accounts private spend key is managed by a hardware wallet
    /// and is required in order to spend funds and generate key images for this
    /// account.
    pub managed_by_hardware_wallet: bool,
}

/// A structure that can be inserted to create a new entity in the `accounts`
/// table.
#[derive(Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub id: &'a str,
    pub account_key: &'a [u8],
    pub entropy: Option<&'a [u8]>,
    pub key_derivation_version: i32,
    pub first_block_index: i64,
    pub next_block_index: i64,
    pub import_block_index: Option<i64>,
    pub name: &'a str,
    pub fog_enabled: bool,
    pub view_only: bool,
    pub managed_by_hardware_wallet: bool,
}

/// A transaction output entity that either was received to an Account in this
/// wallet, or originated from an Account in this wallet. A transaction
/// output can be in one of many states with respect to multiple accounts.
/// Managing these relationships and states is one of the main goals of
/// the Full-Service wallet.
#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = txos)]
pub struct Txo {
    /// Primary key derived from the contents of the ledger TxOut
    pub id: String,
    pub account_id: Option<String>,
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
    /// The receiving subaddress, if known.
    pub subaddress_index: Option<i64>,
    /// Pre-computed key image for this Txo, or None if the Txo is orphaned.
    pub key_image: Option<Vec<u8>>,
    /// Block index containing this Txo.
    pub received_block_index: Option<i64>,
    pub spent_block_index: Option<i64>,
    pub confirmation: Option<Vec<u8>>,
    pub shared_secret: Option<Vec<u8>>,
    pub memo_type: Option<i32>,
}

impl Txo {
    pub fn public_key(&self) -> Result<CompressedRistrettoPublic, mc_util_serial::DecodeError> {
        let public_key: CompressedRistrettoPublic = mc_util_serial::decode(&self.public_key)?;
        Ok(public_key)
    }
}

/// A structure that can be inserted to create a new entity in the `txos` table.
#[derive(Insertable)]
#[diesel(table_name = txos)]
pub struct NewTxo<'a> {
    pub id: &'a str,
    pub account_id: Option<String>,
    pub value: i64,
    pub token_id: i64,
    pub target_key: &'a [u8],
    pub public_key: &'a [u8],
    pub e_fog_hint: &'a [u8],
    pub subaddress_index: Option<i64>,
    pub key_image: Option<&'a [u8]>,
    pub received_block_index: Option<i64>,
    pub spent_block_index: Option<i64>,
    pub confirmation: Option<&'a [u8]>,
    pub shared_secret: Option<&'a [u8]>,
    pub memo_type: Option<i32>,
}

/// A subaddress given to a particular contact, for the purpose of tracking
/// funds received from that contact.
#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(belongs_to(Account, foreign_key = account_id))]
#[diesel(primary_key(public_address_b58))]
#[diesel(table_name = assigned_subaddresses)]
pub struct AssignedSubaddress {
    pub public_address_b58: String,
    pub account_id: String,
    pub subaddress_index: i64,
    pub comment: String,
    pub spend_public_key: Vec<u8>,
}

/// A structure that can be inserted to create a new AssignedSubaddress entity.
#[derive(Insertable)]
#[diesel(table_name = assigned_subaddresses)]
pub struct NewAssignedSubaddress<'a> {
    pub public_address_b58: &'a str,
    pub account_id: &'a str,
    pub subaddress_index: i64,
    pub comment: &'a str,
    pub spend_public_key: &'a [u8],
}

/// The status of a sent transaction OR a received transaction output.
#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(belongs_to(Account, foreign_key = account_id))]
#[diesel(primary_key(id))]
#[diesel(table_name = transaction_logs)]
pub struct TransactionLog {
    pub id: String,
    pub account_id: String,
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
#[diesel(table_name = transaction_logs)]
pub struct NewTransactionLog<'a> {
    pub id: &'a str,
    pub account_id: &'a str,
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
#[diesel(belongs_to(TransactionLog, foreign_key = transaction_log_id))]
#[diesel(belongs_to(Txo, foreign_key = txo_id))]
#[diesel(table_name = transaction_input_txos)]
#[diesel(primary_key(transaction_log_id, txo_id))]
pub struct TransactionInputTxo {
    pub transaction_log_id: String,
    pub txo_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = transaction_input_txos)]
pub struct NewTransactionInputTxo<'a> {
    pub transaction_log_id: &'a str,
    pub txo_id: &'a str,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(belongs_to(TransactionLog, foreign_key = transaction_log_id))]
#[diesel(belongs_to(Txo, foreign_key = txo_id))]
#[diesel(table_name = transaction_output_txos)]
#[diesel(primary_key(transaction_log_id, txo_id))]
pub struct TransactionOutputTxo {
    pub transaction_log_id: String,
    pub txo_id: String,
    pub recipient_public_address_b58: String,
    pub is_change: bool,
}

#[derive(Insertable)]
#[diesel(table_name = transaction_output_txos)]
pub struct NewTransactionOutputTxo<'a> {
    pub transaction_log_id: &'a str,
    pub txo_id: &'a str,
    pub recipient_public_address_b58: &'a str,
    pub is_change: bool,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(belongs_to(Account, foreign_key = id))]
#[diesel(belongs_to(TransactionLog, foreign_key = id))]
#[diesel(table_name = gift_codes)]
#[diesel(primary_key(id))]
pub struct GiftCode {
    pub id: i32,
    pub gift_code_b58: String,
    pub value: i64,
}

#[derive(Insertable)]
#[diesel(table_name = gift_codes)]
pub struct NewGiftCode<'a> {
    pub gift_code_b58: &'a str,
    pub value: i64,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Eq, Debug)]
#[diesel(belongs_to(Txo, foreign_key = txo_id))]
#[diesel(table_name = authenticated_sender_memos)]
#[diesel(primary_key(txo_id))]
pub struct AuthenticatedSenderMemo {
    pub txo_id: String,
    pub sender_address_hash: String,
    pub payment_request_id: Option<i64>,
    pub payment_intent_id: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = authenticated_sender_memos)]
pub struct NewAuthenticatedSenderMemo<'a> {
    pub txo_id: &'a str,
    pub sender_address_hash: &'a str,
    pub payment_request_id: Option<i64>,
    pub payment_intent_id: Option<i64>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = __diesel_schema_migrations)]
pub struct Migration {
    pub version: String,
    pub run_on: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = __diesel_schema_migrations)]
pub struct NewMigration {
    pub version: String,
}

impl NewMigration {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
        }
    }
}

#[derive(Queryable)]
#[diesel(table_name = post_migration_processes)]
pub struct PostMigrationProcess {
    pub migration_version: String,
    pub has_run: bool,
}
