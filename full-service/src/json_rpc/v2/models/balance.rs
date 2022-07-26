// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Balance object.

use std::collections::BTreeMap;

use crate::service;

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct BalanceMap(pub BTreeMap<String, Balance>);

/// The balance for an account, as well as some information about syncing status
/// needed to interpret the balance correctly.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Balance {
    /// The max spendable amount in a single transaction.
    pub max_spendable: String,

    /// Unverified pico MOB. The Unverified value represents the Txos which were
    /// NOT view-key matched, but do have an assigned subaddress.
    pub unverified: String,

    /// Unspent pico MOB for this account at the current account_block_height.
    /// If the account is syncing, this value may change.
    pub unspent: String,

    /// Pending, out-going pico MOB. The pending value will clear once the
    /// ledger processes the outgoing txos. The available_pmob will reflect the
    /// change.
    pub pending: String,

    /// Spent pico MOB. This is the sum of all the Txos in the wallet which have
    /// been spent.
    pub spent: String,

    /// Secreted (minted) pico MOB. This is the sum of all the Txos which have
    /// been created in the wallet for outgoing transactions.
    pub secreted: String,

    /// Orphaned pico MOB. The orphaned value represents the Txos which were
    /// view-key matched, but which can not be spent until their subaddress
    /// index is recovered.
    pub orphaned: String,
}

impl From<&service::balance::Balance> for Balance {
    fn from(src: &service::balance::Balance) -> Balance {
        Balance {
            max_spendable: src.max_spendable.to_string(),
            unverified: src.unverified.to_string(),
            unspent: src.unspent.to_string(),
            pending: src.pending.to_string(),
            spent: src.spent.to_string(),
            secreted: src.secreted.to_string(),
            orphaned: src.orphaned.to_string(),
        }
    }
}
