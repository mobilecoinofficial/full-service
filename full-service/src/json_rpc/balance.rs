// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Balance object.

use crate::service;

use serde_derive::{Deserialize, Serialize};

/// The balance for an account, as well as some information about syncing status
/// needed to interpret the balance correctly.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Balance {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The block count of MobileCoin's distributed ledger. The
    /// local_block_index is synced when it reaches the network_block_index.
    pub network_block_index: String,

    /// The local block count downloaded from the ledger. The local database
    /// will sync up to the network_block_index. The account_block_index can
    /// only sync up to local_block_index.
    pub local_block_index: String,

    /// The scanned local block count for this account. This value will never
    /// be greater than the local_block_index. At fully synced, it will match
    /// network_block_index.
    pub account_block_index: String,

    /// Whether the account is synced with the network_block_index. Balances may
    /// not appear correct if the account is still syncing.
    pub is_synced: bool,

    /// Unspent pico MOB for this account at the current account_block_index. If
    /// the account is syncing, this value may change.
    pub unspent_pmob: String,

    /// Pending, out-going pico MOB. The pending value will clear once the
    /// ledger processes the outgoing txos. The available_pmob will reflect the
    /// change.
    pub pending_pmob: String,

    /// Spent pico MOB. This is the sum of all the Txos in the wallet which have
    /// been spent.
    pub spent_pmob: String,

    /// Secreted (minted) pico MOB. This is the sum of all the Txos which have
    /// been created in the wallet for outgoing transactions.
    pub secreted_pmob: String,

    /// Orphaned pico MOB. The orphaned value represents the Txos which were
    /// view-key matched, but which can not be spent until their subaddress
    /// index is recovered.
    pub orphaned_pmob: String,
}

impl From<&service::balance::Balance> for Balance {
    fn from(src: &service::balance::Balance) -> Balance {
        Balance {
            object: "balance".to_string(),
            network_block_index: src.network_block_index.to_string(),
            local_block_index: src.local_block_index.to_string(),
            account_block_index: src.synced_blocks.to_string(),
            is_synced: src.synced_blocks == src.network_block_index,
            unspent_pmob: src.unspent.to_string(),
            pending_pmob: src.pending.to_string(),
            spent_pmob: src.spent.to_string(),
            secreted_pmob: src.secreted.to_string(),
            orphaned_pmob: src.orphaned.to_string(),
        }
    }
}
