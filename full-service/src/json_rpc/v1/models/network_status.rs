// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Network Status object.

use crate::service;

use mc_transaction_core::{tokens::Mob, Token};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct NetworkStatus {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The block count of MobileCoin's distributed ledger.
    pub network_block_height: String,

    /// The local block count downloaded from the ledger. The local database
    /// is synced when the local_block_height reaches the network_block_height.
    pub local_block_height: String,

    /// The current network fee per transaction, in pmob.
    pub fee_pmob: String,

    /// The current block version
    pub block_version: String,
}

impl TryFrom<&service::balance::NetworkStatus> for NetworkStatus {
    type Error = String;

    fn try_from(src: &service::balance::NetworkStatus) -> Result<NetworkStatus, String> {
        let fee = match src.fees.get(&Mob::ID) {
            Some(fee) => fee,
            None => {
                return Err(format!(
                    "Could not find fee for token {}",
                    Mob::ID.to_string()
                ))
            }
        };

        Ok(NetworkStatus {
            object: "network_status".to_string(),
            network_block_height: src.network_block_height.to_string(),
            local_block_height: src.local_block_height.to_string(),
            fee_pmob: fee.to_string(),
            block_version: src.block_version.to_string(),
        })
    }
}
