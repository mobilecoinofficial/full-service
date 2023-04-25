// Copyright (c) 2020-2023 MobileCoin Inc.

//! API definition for the Wallet Status object.

use crate::{json_rpc::v2::models::balance::Balance, service};

use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::TryFrom};

/// The status of the wallet, including the sum of the balances for all
/// accounts.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct WalletStatus {
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

    pub balance_per_token: BTreeMap<String, Balance>,
}

impl TryFrom<&service::balance::WalletStatus> for WalletStatus {
    type Error = String;

    fn try_from(src: &service::balance::WalletStatus) -> Result<WalletStatus, String> {
        Ok(WalletStatus {
            network_block_height: src.network_block_height.to_string(),
            local_block_height: src.local_block_height.to_string(),
            is_synced_all: src.min_synced_block_index + 1 >= src.network_block_height,
            min_synced_block_index: src.min_synced_block_index.to_string(),
            balance_per_token: src
                .balance_per_token
                .iter()
                .map(|(k, v)| (k.to_string(), Balance::from(v)))
                .collect(),
        })
    }
}
