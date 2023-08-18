// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Balance object.

use std::collections::BTreeMap;

use crate::service;

use redact::{expose_secret, Secret};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct BalanceMap(pub BTreeMap<String, Balance>);

/// The balance for an account, as well as some information about syncing status
/// needed to interpret the balance correctly.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Balance {
    /// The max spendable amount in a single transaction.
    #[serde(serialize_with = "expose_secret")]
    pub max_spendable: Secret<String>,

    /// Unverified pico MOB. The Unverified value represents the Txos which were
    /// NOT view-key matched, but do have an assigned subaddress.
    #[serde(serialize_with = "expose_secret")]
    pub unverified: Secret<String>,

    /// Unspent pico MOB for this account at the current account_block_height.
    /// If the account is syncing, this value may change.
    #[serde(serialize_with = "expose_secret")]
    pub unspent: Secret<String>,

    /// Pending, out-going pico MOB. The pending value will clear once the
    /// ledger processes the outgoing txos. The available_pmob will reflect the
    /// change.
    #[serde(serialize_with = "expose_secret")]
    pub pending: Secret<String>,

    /// Spent pico MOB. This is the sum of all the Txos in the wallet which have
    /// been spent.
    #[serde(serialize_with = "expose_secret")]
    pub spent: Secret<String>,

    /// Secreted (minted) pico MOB. This is the sum of all the Txos which have
    /// been created in the wallet for outgoing transactions.
    #[serde(serialize_with = "expose_secret")]
    pub secreted: Secret<String>,

    /// Orphaned pico MOB. The orphaned value represents the Txos which were
    /// view-key matched, but which can not be spent until their subaddress
    /// index is recovered.
    #[serde(serialize_with = "expose_secret")]
    pub orphaned: Secret<String>,
}

impl From<&service::balance::Balance> for Balance {
    fn from(src: &service::balance::Balance) -> Balance {
        Balance {
            max_spendable: src.max_spendable.to_string().into(),
            unverified: src.unverified.to_string().into(),
            unspent: src.unspent.to_string().into(),
            pending: src.pending.to_string().into(),
            spent: src.spent.to_string().into(),
            secreted: src.secreted.to_string().into(),
            orphaned: src.orphaned.to_string().into(),
        }
    }
}
