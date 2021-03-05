// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and interfaces.

use crate::WalletService;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_sync::NetworkState;

use displaydoc::Display;

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum LedgerServiceError {
    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),
}

impl From<mc_ledger_db::Error> for LedgerServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// ledger objects and interfaces.
pub trait LedgerService {
    /// Gets the network highest block index on the live MobileCoin consensus
    /// network.
    fn get_network_block_index(&self) -> Result<u64, LedgerServiceError>;
}

impl<T, FPR> LedgerService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_network_block_index(&self) -> Result<u64, LedgerServiceError> {
        let network_state = self.network_state.read().expect("lock poisoned");
        Ok(network_state.highest_block_index_on_network().unwrap_or(0))
    }
}
