// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account object.

use crate::{db, util::b58::b58_encode_public_address};
use mc_account_keys::PublicAddress;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct AccountMap(pub BTreeMap<String, Account>);

/// An account in the wallet.
///
/// An Account is associated with one AccountKey, containing a View keypair and
/// a Spend keypair.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Account {
    /// Unique identifier for the account. Constructed from the public key
    /// materials of the account key.
    pub id: String,

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

    /// A flag that indicates if this account is FOG enabled, which means that
    /// it will send any change to it's main subaddress (index 0) instead of
    /// the default change subaddress (index 1). It also generates
    /// PublicAddressB58's with fog credentials.
    pub fog_enabled: bool,

    /// A flag that indicates if this account is a watch only account.
    pub view_only: bool,

    /// A flag that indicates if this account's private spend key is managed by
    /// a hardware wallet.
    pub managed_by_hardware_wallet: bool,
}

impl Account {
    pub fn new(
        src: &db::models::Account,
        main_public_address: &PublicAddress,
        next_subaddress_index: u64,
    ) -> Result<Self, String> {
        let main_public_address_b58 = b58_encode_public_address(main_public_address)
            .map_err(|e| format!("Could not b58 encode public address {e:?}"))?;

        Ok(Account {
            id: src.id.clone(),
            key_derivation_version: src.key_derivation_version.to_string(),
            name: src.name.clone(),
            main_address: main_public_address_b58,
            next_subaddress_index: next_subaddress_index.to_string(),
            first_block_index: (src.first_block_index as u64).to_string(),
            next_block_index: (src.next_block_index as u64).to_string(),
            recovery_mode: false,
            fog_enabled: src.fog_enabled,
            view_only: src.view_only,
            managed_by_hardware_wallet: src.managed_by_hardware_wallet,
        })
    }
}
