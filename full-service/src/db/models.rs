// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB Models

use super::schema::{
    account_txo_statuses, accounts, assigned_subaddresses, transaction_logs, transaction_txo_types,
    txos,
};

use serde::Serialize;

/// A TXO owned by an account in this wallet that has not yet been spent.
pub const TXO_STATUS_UNSPENT: &str = "txo_status_unspent";

/// A TXO owned by an account in this wallet that is used by a pending
/// transaction.
pub const TXO_STATUS_PENDING: &str = "txo_status_pending";

/// A TXO owned by an account in this wallet that has been spent.
pub const TXO_STATUS_SPENT: &str = "txo_status_spent";

/// A TXO created by an account in this wallet for use as an output in an
/// outgoing transaction.
pub const TXO_STATUS_SECRETED: &str = "txo_status_secreted";

/// The TXO is owned by this wallet, but not yet spendable (i.e., receiving
/// subaddress is unknown).
pub const TXO_STATUS_ORPHANED: &str = "txo_status_orphaned";

/// A Txo that has been created locally, but is not yet in the ledger.
pub const TXO_TYPE_MINTED: &str = "minted";

/// A Txo in the ledger that belongs to an account in this wallet.
pub const TXO_TYPE_RECEIVED: &str = "received";

/// A transaction that has been built locally.
pub const TX_STATUS_BUILT: &str = "built";

/// A transaction that has been submitted to the MobileCoin network.
pub const TX_STATUS_PENDING: &str = "pending";

/// A transaction that appears to have been processed by the MobileCoin network.
pub const TX_STATUS_SUCCEEDED: &str = "succeeded";

/// A transaction that was rejected by the MobileCoin network, or that expired
/// before it could be processed.
pub const TX_STATUS_FAILED: &str = "failed";

/// A transaction created by an account in this wallet.
pub const TX_DIR_SENT: &str = "sent";

/// A TxOut received by an account in this wallet.
pub const TX_DIR_RECEIVED: &str = "received";

/// A transaction output that is used as an input to a new transaction.
pub const TXO_USED_AS_INPUT: &str = "input";

/// A transaction output that is used as an output of a new transaction.
pub const TXO_USED_AS_OUTPUT: &str = "output";

/// A transaction output used as a change output of a new transaction.
pub const TXO_USED_AS_CHANGE: &str = "change";

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
    /// Private keys for viewing and spending the MobileCoin belonging to an
    /// account.
    pub account_key: Vec<u8>,
    /// Default subadress that is given out to refer to this account.
    pub main_subaddress_index: i64,
    /// Subaddress used to return transaction "change" to self.
    pub change_subaddress_index: i64,
    /// The next unused subaddress index. (Assumes indices are used sequentially
    /// from 0).
    pub next_subaddress_index: i64,
    /// Index of the first block where this account may have held funds.
    pub first_block: i64,
    /// Index of the next block to inspect for transactions related to this
    /// account.
    pub next_block: i64,
    /// If the account was imported, account history prior to this block index
    /// is derived from the public ledger, and does not reflect client-side
    /// user events.
    pub import_block: Option<i64>,
    /// Name of this account.
    pub name: String, /* empty string for nullable */
}

/// A structure that can be inserted to create a new entity in the `accounts`
/// table.
#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount<'a> {
    pub account_id_hex: &'a str,
    pub account_key: &'a [u8],
    pub main_subaddress_index: i64,
    pub change_subaddress_index: i64,
    pub next_subaddress_index: i64,
    pub first_block: i64,
    pub next_block: i64,
    pub import_block: Option<i64>,
    pub name: &'a str,
}

/// A received transaction output entity that belongs to a an Account in this
/// wallet. Also maybe a transaction output that hasn't been sent yet?
#[derive(Clone, Serialize, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(id)]
pub struct Txo {
    /// Primary key
    pub id: i32,
    /// An additional ID derived from the contents of the ledger TxOut.
    pub txo_id_hex: String,
    /// The value of this transaction output, in picoMob.
    pub value: i64,
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
    pub received_block_count: Option<i64>,
    pub pending_tombstone_block_count: Option<i64>,
    pub spent_block_count: Option<i64>,
    pub proof: Option<Vec<u8>>,
}

/// A structure that can be inserted to create a new entity in the `txos` table.
#[derive(Insertable)]
#[table_name = "txos"]
pub struct NewTxo<'a> {
    pub txo_id_hex: &'a str,
    pub value: i64,
    pub target_key: &'a [u8],
    pub public_key: &'a [u8],
    pub e_fog_hint: &'a [u8],
    pub txo: &'a [u8],
    pub subaddress_index: Option<i64>,
    pub key_image: Option<&'a [u8]>,
    pub received_block_count: Option<i64>,
    pub pending_tombstone_block_count: Option<i64>,
    pub spent_block_count: Option<i64>,
    pub proof: Option<&'a [u8]>,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(Account, foreign_key = "account_id_hex")]
#[belongs_to(Txo, foreign_key = "txo_id_hex")]
#[table_name = "account_txo_statuses"]
#[primary_key(account_id_hex, txo_id_hex)]
pub struct AccountTxoStatus {
    pub account_id_hex: String,
    pub txo_id_hex: String,
    // Statuses: unspent, pending, spent, secreted, orphaned
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
#[belongs_to(AssignedSubaddress, foreign_key = "assigned_subaddress_b58")]
#[primary_key(id)]
#[table_name = "transaction_logs"]
pub struct TransactionLog {
    pub id: i32,
    pub transaction_id_hex: String,
    pub account_id_hex: String,
    pub recipient_public_address_b58: String, // empty string for nullable
    pub assigned_subaddress_b58: String,      // empty string for nullable
    pub value: i64,
    pub fee: Option<i64>,
    // Statuses: built, pending, succeeded, failed
    pub status: String,
    pub sent_time: Option<i64>, // empty string for nullable
    pub submitted_block_count: Option<i64>,
    pub finalized_block_count: Option<i64>,
    pub comment: String, // empty string for nullable
    // Directions: sent, received
    pub direction: String,
    pub tx: Option<Vec<u8>>,
}

/// A structure that can be inserted to create a new TransactionLog entity.
#[derive(Insertable)]
#[table_name = "transaction_logs"]
pub struct NewTransactionLog<'a> {
    pub transaction_id_hex: &'a str,
    pub account_id_hex: &'a str,
    pub recipient_public_address_b58: &'a str,
    pub assigned_subaddress_b58: &'a str,
    pub value: i64,
    pub fee: Option<i64>,
    pub status: &'a str,
    pub sent_time: Option<i64>,
    pub submitted_block_count: Option<i64>,
    pub finalized_block_count: Option<i64>,
    pub comment: &'a str,
    pub direction: &'a str,
    pub tx: Option<&'a [u8]>,
}

#[derive(Clone, Serialize, Associations, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(TransactionLog, foreign_key = "transaction_id_hex")]
#[belongs_to(Txo, foreign_key = "txo_id_hex")]
#[table_name = "transaction_txo_types"]
#[primary_key(transaction_id_hex, txo_id_hex)]
pub struct TransactionTxoType {
    pub transaction_id_hex: String,
    pub txo_id_hex: String,
    // Statuses: input, output, change
    pub transaction_txo_type: String,
}

#[derive(Insertable)]
#[table_name = "transaction_txo_types"]
pub struct NewTransactionTxoType<'a> {
    pub transaction_id_hex: &'a str,
    pub txo_id_hex: &'a str,
    pub transaction_txo_type: &'a str,
}
