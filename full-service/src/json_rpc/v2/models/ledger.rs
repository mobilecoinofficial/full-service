// Copyright (c) 2020-2023 MobileCoin Inc.

//! API definition for ledger-related objects.

use crate::{
    json_rpc::v2::models::{
        block::{Block, BlockContents},
        watcher::WatcherBlockInfo,
    },
    service::models::ledger::LedgerSearchResult as ServiceLedgerSearchResult,
};
use serde_derive::{Deserialize, Serialize};

// TxOut search result
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LedgerTxOutSearchResult {
    pub block_contents_tx_out_index: String,
    pub global_tx_out_index: String,
}

// KeyImage search result
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LedgerKeyImageSearchResult {
    pub block_contents_key_image_index: String,
}

/// Information about a single search result
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LedgerSearchResult {
    pub result_type: String,
    pub block: Block,
    pub block_contents: BlockContents,
    pub tx_out: Option<LedgerTxOutSearchResult>,
    pub key_image: Option<LedgerKeyImageSearchResult>,
    pub watcher_info: Option<WatcherBlockInfo>,
}

impl From<&ServiceLedgerSearchResult> for LedgerSearchResult {
    fn from(src: &ServiceLedgerSearchResult) -> Self {
        match src {
            ServiceLedgerSearchResult::TxOut {
                block,
                block_contents,
                block_contents_tx_out_index,
                tx_out_global_index,
                watcher_info,
            } => Self {
                result_type: "TxOut".to_string(),
                block: block.into(),
                block_contents: block_contents.into(),
                tx_out: Some(LedgerTxOutSearchResult {
                    block_contents_tx_out_index: block_contents_tx_out_index.to_string(),
                    global_tx_out_index: tx_out_global_index.to_string(),
                }),
                watcher_info: watcher_info.as_ref().map(Into::into),
                ..Default::default()
            },
            ServiceLedgerSearchResult::KeyImage {
                block,
                block_contents,
                block_contents_key_image_index,
                watcher_info,
            } => Self {
                result_type: "KeyImage".to_string(),
                block: block.into(),
                block_contents: block_contents.into(),
                key_image: Some(LedgerKeyImageSearchResult {
                    block_contents_key_image_index: block_contents_key_image_index.to_string(),
                }),
                watcher_info: watcher_info.as_ref().map(Into::into),
                ..Default::default()
            },
        }
    }
}
