// Copyright (c) 2020-2023 MobileCoin Inc.

//! API definition for watcher-related objects.

use super::block::BlockSignature;
use mc_watcher_api::TimestampResultCode;
use serde_derive::{Deserialize, Serialize};

/// Information about a single block signature
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct BlockSignatureData {
    pub src_url: String,
    pub archive_filename: String,
    pub block_signature: BlockSignature,
}

impl From<&mc_watcher::watcher_db::BlockSignatureData> for BlockSignatureData {
    fn from(src: &mc_watcher::watcher_db::BlockSignatureData) -> Self {
        Self {
            src_url: src.src_url.clone(),
            archive_filename: src.archive_filename.clone(),
            block_signature: (&src.block_signature).into(),
        }
    }
}

/// Information about a block provided by the Watcher.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct WatcherBlockInfo {
    pub timestamp: String,
    pub timestamp_result_code: String,
    pub signatures: Vec<BlockSignatureData>,
}

impl From<&crate::service::models::watcher::WatcherBlockInfo> for WatcherBlockInfo {
    fn from(src: &crate::service::models::watcher::WatcherBlockInfo) -> Self {
        Self {
            timestamp: src.timestamp.to_string(),
            timestamp_result_code: match src.timestamp_result_code {
                TimestampResultCode::TimestampFound => "TimestampFound".to_string(),
                TimestampResultCode::WatcherBehind => "WatcherBehind".to_string(),
                TimestampResultCode::Unavailable => "Unavailable".to_string(),
                TimestampResultCode::WatcherDatabaseError => "WatcherDatabaseError".to_string(),
                TimestampResultCode::BlockIndexOutOfBounds => "BlockIndexOutOfBounds".to_string(),
            },
            signatures: src.signatures.iter().map(Into::into).collect(),
        }
    }
}
