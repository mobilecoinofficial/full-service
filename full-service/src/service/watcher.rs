// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for accessing the watcher database.
use crate::{service::models::watcher::WatcherBlockInfo, WalletService};
use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_watcher::error::WatcherDBError;

/// Errors for the Watcher Service.
#[derive(Display, Debug)]
pub enum WatcherServiceError {
    /// Error interacting with watcher database: {0}
    WatcherDb(WatcherDBError),
}

impl From<WatcherDBError> for WatcherServiceError {
    fn from(src: WatcherDBError) -> Self {
        Self::WatcherDb(src)
    }
}

/// Trait defining the ways in which the service can interact with the watcher.
pub trait WatcherService {
    fn get_watcher_block_info(
        &self,
        block_index: u64,
    ) -> Result<Option<WatcherBlockInfo>, WatcherServiceError>;
}

impl<T, FPR> WatcherService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_watcher_block_info(
        &self,
        block_index: u64,
    ) -> Result<Option<WatcherBlockInfo>, WatcherServiceError> {
        match &self.watcher_db {
            Some(watcher_db) => {
                let (timestamp, timestamp_result_code) =
                    watcher_db.get_block_timestamp(block_index)?;
                let signatures = watcher_db.get_block_signatures(block_index)?;

                Ok(Some(WatcherBlockInfo {
                    timestamp,
                    timestamp_result_code,
                    signatures,
                }))
            }
            None => Ok(None),
        }
    }
}
