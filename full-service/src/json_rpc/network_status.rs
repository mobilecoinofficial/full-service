// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Network Status object.

use crate::service;

use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct NetworkStatus {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The highest index block on MobileCoin's distributed ledger. The
    /// local_block_index is synced when it reaches the network_block_index.
    pub network_block_index: String,

    /// The highest index block that has been downloaded from the ledger.
    pub local_block_index: String,

    /// The current network fee per transaction, in pmob.
    pub fee_pmob: String,
}

impl TryFrom<&service::balance::NetworkStatus> for NetworkStatus {
    type Error = String;

    fn try_from(src: &service::balance::NetworkStatus) -> Result<NetworkStatus, String> {
        Ok(NetworkStatus {
            object: "network_status".to_string(),
            network_block_index: src.network_block_index.to_string(),
            local_block_index: src.local_block_index.to_string(),
            fee_pmob: src.fee_pmob.to_string(),
        })
    }
}
