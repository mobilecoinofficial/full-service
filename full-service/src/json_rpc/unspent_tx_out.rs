// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the UnspentTxOut object.

use mc_mobilecoind_json::data_types::JsonTxOut;

use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct UnspentTxOut {
    pub tx_out: JsonTxOut,
    pub subaddress_index: String,
    pub key_image: String,
    pub value: u64,
    pub attempted_spend_height: String,
    pub attempted_spend_tombstone: String,
    pub monitor_id: String,
}

impl TryFrom<&mc_mobilecoind_json::data_types::JsonUnspentTxOut> for UnspentTxOut {
    type Error = String;

    fn try_from(src: &mc_mobilecoind_json::data_types::JsonUnspentTxOut) -> Result<Self, String> {
        Ok(Self {
            tx_out: src.tx_out.clone(),
            subaddress_index: src.subaddress_index.to_string(),
            key_image: src.key_image.clone(),
            value: src
                .value
                .parse::<u64>()
                .map_err(|err| format!("Failed to parse u64 from value: {}", err))?,
            attempted_spend_height: src.attempted_spend_height.to_string(),
            attempted_spend_tombstone: src.attempted_spend_tombstone.to_string(),
            monitor_id: src.monitor_id.clone(),
        })
    }
}

impl TryFrom<&UnspentTxOut> for mc_mobilecoind_json::data_types::JsonUnspentTxOut {
    type Error = String;

    fn try_from(
        src: &UnspentTxOut,
    ) -> Result<mc_mobilecoind_json::data_types::JsonUnspentTxOut, String> {
        Ok(Self {
            tx_out: src.tx_out.clone(),
            subaddress_index: src
                .subaddress_index
                .parse::<u64>()
                .map_err(|err| format!("Failed to parse u64 from subaddress_index: {}", err))?,
            key_image: src.key_image.clone(),
            value: src.value.to_string(),
            attempted_spend_height: src.attempted_spend_height.parse::<u64>().map_err(|err| {
                format!("Failed to parse u64 from attempted_spend_height: {}", err)
            })?,
            attempted_spend_tombstone: src.attempted_spend_tombstone.parse::<u64>().map_err(
                |err| {
                    format!(
                        "Failed to parse u64 from attempted_spend_tombstone: {}",
                        err
                    )
                },
            )?,
            monitor_id: src.monitor_id.clone(),
        })
    }
}
