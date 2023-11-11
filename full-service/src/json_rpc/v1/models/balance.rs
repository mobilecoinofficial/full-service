// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Balance object.

use crate::service;

use redact::{expose_secret, Secret};
use serde_derive::{Deserialize, Serialize};

/// The balance for an account, as well as some information about syncing status
/// needed to interpret the balance correctly.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Balance {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The block count of MobileCoin's distributed ledger.
    pub network_block_height: String,

    /// The local block count downloaded from the ledger. The local database
    /// is synced when the local_block_height reaches the network_block_height.
    /// The account_block_height can only sync up to local_block_height.
    pub local_block_height: String,

    /// The scanned local block count for this account. This value will never
    /// be greater than the local_block_height. At fully synced, it will match
    /// network_block_height.
    pub account_block_height: String,

    /// Whether the account is synced with the network_block_height. Balances
    /// may not appear correct if the account is still syncing.
    pub is_synced: bool,

    /// Unspent pico MOB for this account at the current account_block_height.
    /// If the account is syncing, this value may change.
    #[serde(serialize_with = "expose_secret")]
    pub unspent_pmob: Secret<String>,

    /// The maximum amount of pico MOB that can be sent in a single transaction.
    /// Equal to the sum of the 16 highest value txos - the network fee.
    /// If the account is syncing, this value may change.
    #[serde(serialize_with = "expose_secret")]
    pub max_spendable_pmob: Secret<String>,

    /// Pending, out-going pico MOB. The pending value will clear once the
    /// ledger processes the outgoing txos. The available_pmob will reflect the
    /// change.
    #[serde(serialize_with = "expose_secret")]
    pub pending_pmob: Secret<String>,

    /// Spent pico MOB. This is the sum of all the Txos in the wallet which have
    /// been spent.
    #[serde(serialize_with = "expose_secret")]
    pub spent_pmob: Secret<String>,

    /// Secreted (minted) pico MOB. This is the sum of all the Txos which have
    /// been created in the wallet for outgoing transactions.
    #[serde(serialize_with = "expose_secret")]
    pub secreted_pmob: Secret<String>,

    /// Orphaned pico MOB. The orphaned value represents the Txos which were
    /// view-key matched, but which can not be spent until their subaddress
    /// index is recovered.
    #[serde(serialize_with = "expose_secret")]
    pub orphaned_pmob: Secret<String>,
}

impl Balance {
    pub fn new(
        balance: &service::balance::Balance,
        account_block_height: u64,
        network_status: &service::balance::NetworkStatus,
    ) -> Self {
        Balance {
            object: "balance".to_string(),
            network_block_height: network_status.network_block_height.to_string(),
            local_block_height: network_status.local_block_height.to_string(),
            account_block_height: account_block_height.to_string(),
            is_synced: account_block_height == network_status.network_block_height,
            unspent_pmob: (balance.unspent + balance.unverified).to_string().into(),
            max_spendable_pmob: balance.max_spendable.to_string().into(),
            pending_pmob: balance.pending.to_string().into(),
            spent_pmob: balance.spent.to_string().into(),
            secreted_pmob: balance.secreted.to_string().into(),
            orphaned_pmob: balance.orphaned.to_string().into(),
        }
    }
}
