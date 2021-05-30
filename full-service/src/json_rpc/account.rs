// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account object.

use crate::db;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// An account in the wallet.
///
/// An Account is associated with one AccountKey, containing a View keypair and
/// a Spend keypair.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Account {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// Unique identifier for the account. Constructed from the public key
    /// materials of the account key.
    pub account_id: String,

    /// Display name for the account.
    pub name: String,

    /// Key Derivation Version
    pub key_derivation_version: String,

    /// B58 Address Code for the account's main address. The main address is
    /// determined by the seed subaddress. It is not assigned to a single
    /// recipient, and should be consider a free-for-all address.
    pub main_address: String,

    /// This index represents the next subaddress to be assigned as an address.
    /// This is useful information in case the account is imported elsewhere.
    pub next_subaddress_index: String,

    /// Index of the first block when this account may have received funds.
    /// No transactions before this point will be synchronized.
    pub first_block_index: String,

    /// Index of the next block this account needs to sync.
    pub next_block_index: String,

    /// A flag that indicates this imported account is attempting to un-orphan
    /// found TXOs. It is recommended to move all MOB to another account after
    /// recovery if the user is unsure of the assigned addresses.
    pub recovery_mode: bool,
}

impl TryFrom<&db::models::Account> for Account {
    type Error = String;

    fn try_from(src: &db::models::Account) -> Result<Account, String> {
        let account_key: mc_account_keys::AccountKey = mc_util_serial::decode(&src.account_key)
            .map_err(|e| format!("Could not decode account key: {:?}", e))?;
        let main_address =
            db::b58::b58_encode(&account_key.subaddress(src.main_subaddress_index as u64))
                .map_err(|e| format!("Could not b58 encode public address {:?}", e))?;

        Ok(Account {
            object: "account".to_string(),
            account_id: src.account_id_hex.clone(),
            key_derivation_version: src.key_derivation_version.to_string(),
            name: src.name.clone(),
            main_address,
            next_subaddress_index: (src.next_subaddress_index as u64).to_string(),
            first_block_index: (src.first_block_index as u64).to_string(),
            next_block_index: (src.next_block_index as u64).to_string(),
            recovery_mode: false,
        })
    }
}
