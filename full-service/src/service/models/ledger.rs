// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for LedgerService-related objects.

use mc_blockchain_types::{Block, BlockContents};
use serde_derive::{Deserialize, Serialize};

/// A single search result from the ledger.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LedgerSearchResult {
    /// Query matched a TxOut
    TxOut {
        /// The block that contains the TxOut
        block: Block,

        /// The block contents that contains the TxOut
        block_contents: BlockContents,

        /// The index of the output inside the block contents
        block_contents_tx_out_index: u64,

        /// The global index of the TxOut
        tx_out_global_index: u64,
    },

    /// Query matched a KeyImage
    KeyImage {
        /// The block that contains the TxOut
        block: Block,

        /// The block contents that contains the TxOut
        block_contents: BlockContents,

        /// The index of the key image inside the block contents
        block_contents_key_image_index: u64,
    },
}
