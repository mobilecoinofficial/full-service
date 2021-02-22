// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Balance object.

use serde_derive::{Deserialize, Serialize};

/// The balance for an account, as well as some information about syncing status
/// needed to interpret the balance correctly.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Balance {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The block count of MobileCoin's distributed ledger. The
    /// local_block_count is synced when it reaches the network_block_count.
    pub network_block_count: String,

    /// The local block count downloaded from the ledger. The local database
    /// will sync up to the network_block_count. The account_block_count can
    /// only sync up to local_block_count.
    pub local_block_count: String,

    /// The scanned local block count for this account. This value will never
    /// be greater than the local_block_count. At fully synced, it will match
    /// network_block_count.
    pub account_block_count: String,

    /// Whether the account is synced with the network_block_count. Balances may
    /// not appear correct if the account is still syncing.
    pub is_synced: bool,

    /// Unspent pico MOB for this account at the current account_block_count. If
    /// the account is syncing, this value may change.
    pub unspent_pmob: String,

    /// Pending, out-going pico MOB. The pending value will clear once the
    /// ledger processes the outgoing txos. The available_pmob will reflect the
    /// change.
    pub pending_pmob: String,
}
