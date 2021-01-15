// Copyright (c) 2018-2020 MobileCoin Inc.

//! The signed JSON request sent from clients when operating in encrypted mode.

use serde_derive::Deserialize;

// The request object sent from clients when using the SignedRequest call.
#[derive(Deserialize)]
pub enum SignedJsonRequest {
    GetProcessedBlock {
        block: u64,
    },
    GetBlock {
        block: u64,
    },
    GetBlockInfo {
        block: u64,
    },
    GetLedgerInfo,
    GetBlockIndexByTxPubKey {
        // Hex encoded
        tx_public_key: String,
    },
}
