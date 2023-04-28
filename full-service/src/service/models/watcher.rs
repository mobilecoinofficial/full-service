// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for watcher-related objects.

use mc_watcher::watcher_db::BlockSignatureData;
use mc_watcher_api::TimestampResultCode;

use serde_derive::{Deserialize, Serialize};

/// Information about a block provided by the Watcher.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WatcherBlockInfo {
    pub timestamp: u64,
    pub timestamp_result_code: TimestampResultCode,
    pub signatures: Vec<BlockSignatureData>,
}
