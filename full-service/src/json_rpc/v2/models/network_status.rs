// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Network Status object.

use crate::{config::NetworkConfig, service};
use mc_transaction_core::constants;
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::TryFrom};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct NetworkStatus {
    /// The block count of MobileCoin's distributed ledger.
    pub network_block_height: String,

    /// The local block count downloaded from the ledger. The local database
    /// is synced when the local_block_height reaches the network_block_height.
    pub local_block_height: String,

    /// The number of TxOuts in the local ledger.
    pub local_num_txos: String,

    /// The current network fee per token_id.
    pub fees: BTreeMap<String, String>,

    /// The current block version
    pub block_version: String,

    pub max_tombstone_blocks: String,

    /// How we're connecting to the network
    pub network_info: NetworkConfig,
}

impl TryFrom<&service::balance::NetworkStatus> for NetworkStatus {
    type Error = String;

    fn try_from(src: &service::balance::NetworkStatus) -> Result<NetworkStatus, String> {
        Ok(NetworkStatus {
            network_block_height: src.network_block_height.to_string(),
            local_block_height: src.local_block_height.to_string(),
            local_num_txos: src.local_num_txos.to_string(),
            fees: src
                .fees
                .iter()
                .map(|(token_id, fee)| (token_id.to_string(), fee.to_string()))
                .collect(),
            block_version: src.block_version.to_string(),
            max_tombstone_blocks: constants::MAX_TOMBSTONE_BLOCKS.to_string(),
            network_info: src.network_info.clone(),
        })
    }
}
