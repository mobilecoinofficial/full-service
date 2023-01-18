// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for LedgerService-related objects.

use serde_derive::{Deserialize, Serialize};

/// A single search result from the ledger.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LedgerSearchResult {
    /// Query matched a TxOut
    TxOut {
        /// The block index that contains the TxOut
        block_index: u64,

        /// The index of the output inside the block contents
        block_contents_tx_out_index: u64,

        /// The global tx out index
        global_tx_out_index: u64,
    },

    /// Query matched a KeyImage
    KeyImage {
        /// The block index that contains the KeyImage
        block_index: u64,

        /// The index of the key image inside the block contents
        block_contents_key_image_index: u64,
    },
}
