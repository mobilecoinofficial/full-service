// Copyright (c) 2020-2023 MobileCoin Inc.

//! API definition for the Wallet Status object.

use crate::service;

use mc_transaction_core::{tokens::Mob, Token};
use serde_derive::{Deserialize, Serialize};
use serde_json::Map;

/// The status of the wallet, including the sum of the balances for all
/// accounts.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct WalletStatus {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The block count of MobileCoin's distributed ledger.
    pub network_block_height: String,

    /// The local block count downloaded from the ledger. The local database
    /// is synced when the local_block_height reaches the network_block_height.
    /// The account_block_height can only sync up to local_block_height.
    pub local_block_height: String,

    /// Whether ALL accounts are synced up to the network_block_height. Balances
    /// may not appear correct if any account is still syncing.
    pub is_synced_all: bool,

    /// The minimum synced block across all accounts
    pub min_synced_block_index: String,

    /// Unspent pico mob for ALL accounts at the account_block_height. If the
    /// account is syncing, this value may change.
    pub total_unspent_pmob: String,

    /// Pending out-going pico mob from ALL accounts. Pending pico mobs will
    /// clear once the ledger processes the outgoing txo. The available_pmob
    /// will reflect the change.
    pub total_pending_pmob: String,

    /// Spent pico MOB. This is the sum of all the Txos in the wallet which have
    /// been spent.
    pub total_spent_pmob: String,

    /// Secreted (minted) pico MOB. This is the sum of all the Txos which have
    /// been created in the wallet for outgoing transactions.
    pub total_secreted_pmob: String,

    /// Orphaned pico MOB. The orphaned value represents the Txos which were
    /// view-key matched, but which can not be spent until their subaddress
    /// index is recovered.
    pub total_orphaned_pmob: String,

    /// A list of all account_ids imported into the wallet in order of import.
    pub account_ids: Vec<String>,

    /// A normalized hash mapping account_id to account objects.
    pub account_map: Map<String, serde_json::Value>,
}

impl WalletStatus {
    pub fn new(
        src: &service::balance::WalletStatus,
        account_map: Map<String, serde_json::Value>,
    ) -> Result<Self, String> {
        let balance_mob = src.balance_per_token.get(&Mob::ID).unwrap_or_default();

        Ok(WalletStatus {
            object: "wallet_status".to_string(),
            network_block_height: src.network_block_height.to_string(),
            local_block_height: src.local_block_height.to_string(),
            is_synced_all: src.min_synced_block_index + 1 >= src.network_block_height,
            min_synced_block_index: src.min_synced_block_index.to_string(),
            total_unspent_pmob: (balance_mob.unspent + balance_mob.unverified).to_string(),
            total_pending_pmob: balance_mob.pending.to_string(),
            total_spent_pmob: balance_mob.spent.to_string(),
            total_secreted_pmob: balance_mob.secreted.to_string(),
            total_orphaned_pmob: balance_mob.orphaned.to_string(),
            account_ids: src.account_ids.iter().map(|a| a.to_string()).collect(),
            account_map,
        })
    }
}
