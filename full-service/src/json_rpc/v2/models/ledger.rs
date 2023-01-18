// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for ledger-related objects.

use crate::service::models::ledger::LedgerSearchResult as ServiceLedgerSearchResult;
use serde_derive::{Deserialize, Serialize};

// TxOut search result
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LedgerTxOutSearchResult {
    pub block_index: String,
    pub block_contents_tx_out_index: String,
    pub global_tx_out_index: String,
}

// KeyImage search result
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LedgerKeyImageSearchResult {
    pub block_index: String,
    pub block_contents_key_image_index: String,
}

/// Information about a single search result
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LedgerSearchResult {
    pub result_type: String,
    pub tx_out: Option<LedgerTxOutSearchResult>,
    pub key_image: Option<LedgerKeyImageSearchResult>,
}

impl From<&ServiceLedgerSearchResult> for LedgerSearchResult {
    fn from(src: &ServiceLedgerSearchResult) -> Self {
        match src {
            ServiceLedgerSearchResult::TxOut {
                block_index,
                block_contents_tx_out_index,
                global_tx_out_index,
            } => Self {
                result_type: "TxOut".to_string(),
                tx_out: Some(LedgerTxOutSearchResult {
                    block_index: block_index.to_string(),
                    block_contents_tx_out_index: block_contents_tx_out_index.to_string(),
                    global_tx_out_index: global_tx_out_index.to_string(),
                }),
                key_image: None,
            },
            ServiceLedgerSearchResult::KeyImage {
                block_index,
                block_contents_key_image_index,
            } => Self {
                result_type: "KeyImage".to_string(),
                tx_out: None,
                key_image: Some(LedgerKeyImageSearchResult {
                    block_index: block_index.to_string(),
                    block_contents_key_image_index: block_contents_key_image_index.to_string(),
                }),
            },
        }
    }
}
