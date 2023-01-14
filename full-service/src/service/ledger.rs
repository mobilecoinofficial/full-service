// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use crate::{
    db::{
        models::{TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
    },
    WalletService,
};
use mc_connection::{
    BlockInfo, BlockchainConnection, RetryableBlockchainConnection, UserTxConnection,
    _retry::delay::Fibonacci,
};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_ledger_sync::NetworkState;
use mc_transaction_core::{
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut},
    Block, BlockContents, BlockVersion, BlockVersionError, Token,
};

use crate::db::WalletDbError;
use displaydoc::Display;
use rayon::prelude::*; // For par_iter
use std::convert::TryFrom;

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum LedgerServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /** No transaction object associated with this transaction. Note,
     * received transactions do not have transaction objects.
     */
    NoTxInTransaction,

    /// No node responded to the last block info request
    NoLastBlockInfo,

    /// Inconsistent last block info
    InconsistentLastBlockInfo,

    /// Block version: {0}
    BlockVersion(BlockVersionError),

    /// Fee for Mob is missing from all nodes
    FeeForMobIsMissing,
}

impl From<mc_ledger_db::Error> for LedgerServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_util_serial::DecodeError> for LedgerServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<WalletDbError> for LedgerServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<BlockVersionError> for LedgerServiceError {
    fn from(src: BlockVersionError) -> Self {
        Self::BlockVersion(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// ledger objects and interfaces.
pub trait LedgerService {
    /// Get the total number of blocks on the ledger.
    fn get_network_block_height(&self) -> Result<u64, LedgerServiceError>;

    fn get_transaction_object(&self, transaction_id_hex: &str) -> Result<Tx, LedgerServiceError>;

    fn get_txo_object(&self, txo_id_hex: &str) -> Result<TxOut, LedgerServiceError>;

    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(Block, BlockContents), LedgerServiceError>;

    fn contains_key_image(&self, key_image: &KeyImage) -> Result<bool, LedgerServiceError>;

    fn get_network_fee(&self) -> Result<u64, LedgerServiceError>;

    fn get_network_block_version(&self) -> Result<BlockVersion, LedgerServiceError>;

    fn get_latest_block_info(&self) -> Result<BlockInfo, LedgerServiceError>;
}

impl<T, FPR> LedgerService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_network_block_height(&self) -> Result<u64, LedgerServiceError> {
        let network_state = self.network_state.read().expect("lock poisoned");
        match network_state.highest_block_index_on_network() {
            Some(index) => Ok(index + 1),
            None => Ok(0),
        }
    }

    fn get_transaction_object(&self, transaction_id_hex: &str) -> Result<Tx, LedgerServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction = TransactionLog::get(transaction_id_hex, &conn)?;

        if let Some(tx_bytes) = transaction.tx {
            let tx: Tx = mc_util_serial::decode(&tx_bytes)?;
            Ok(tx)
        } else {
            Err(LedgerServiceError::NoTxInTransaction)
        }
    }

    fn get_txo_object(&self, txo_id_hex: &str) -> Result<TxOut, LedgerServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let txo_details = Txo::get(txo_id_hex, &conn)?;

        let txo: TxOut = mc_util_serial::decode(&txo_details.txo)?;
        Ok(txo)
    }

    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(Block, BlockContents), LedgerServiceError> {
        let block = self.ledger_db.get_block(block_index)?;
        let block_contents = self.ledger_db.get_block_contents(block_index)?;
        Ok((block, block_contents))
    }

    fn contains_key_image(&self, key_image: &KeyImage) -> Result<bool, LedgerServiceError> {
        Ok(self.ledger_db.contains_key_image(key_image)?)
    }

    fn get_latest_block_info(&self) -> Result<BlockInfo, LedgerServiceError> {
        // Get the last block information from all nodes we are aware of, in parallel.
        let last_block_infos = self
            .peer_manager
            .conns()
            .par_iter()
            .filter_map(|conn| {
                conn.fetch_block_info(Fibonacci::from_millis(10).take(5))
                    .ok()
            })
            .collect::<Vec<_>>();

        // Ensure that all nodes agree on the latest block version and network fees.
        if last_block_infos.windows(2).any(|window| {
            window[0].network_block_version != window[1].network_block_version
                || window[0].minimum_fees != window[1].minimum_fees
        }) {
            return Err(LedgerServiceError::InconsistentLastBlockInfo);
        }

        last_block_infos
            .first()
            .cloned()
            .ok_or(LedgerServiceError::NoLastBlockInfo)
    }

    fn get_network_fee(&self) -> Result<u64, LedgerServiceError> {
        Ok(*self
            .get_latest_block_info()?
            .minimum_fees
            .get(&Mob::ID)
            .ok_or(LedgerServiceError::FeeForMobIsMissing)?)
    }

    fn get_network_block_version(&self) -> Result<BlockVersion, LedgerServiceError> {
        Ok(BlockVersion::try_from(
            self.get_latest_block_info()?.network_block_version,
        )?)
    }
}
