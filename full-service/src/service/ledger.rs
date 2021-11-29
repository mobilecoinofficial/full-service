// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

// For par_iter
use std::iter::empty;

use displaydoc::Display;
use mc_connection::{BlockchainConnection, RetryableBlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_ledger_sync::NetworkState;
use mc_transaction_core::{
    constants::MINIMUM_FEE,
    ring_signature::KeyImage,
    tx::{Tx, TxOut},
    Block, BlockContents,
};
use rayon::prelude::*;

use crate::{
    db::{
        models::{TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDbError,
    },
    WalletService,
};

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum LedgerServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding prost: {0}
    ProstDecode(prost::DecodeError),

    /// No transaction object associated with this transaction. Note, received
    /// transactions do not have transaction objects.
    NoTxInTransaction,

    /// Error decoding with mc_util_serial {0}
    Decode(mc_util_serial::DecodeError),
}

impl From<mc_ledger_db::Error> for LedgerServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<prost::DecodeError> for LedgerServiceError {
    fn from(src: prost::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<WalletDbError> for LedgerServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<mc_util_serial::DecodeError> for LedgerServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::Decode(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// ledger objects and interfaces.
pub trait LedgerService {
    /// Gets the network highest block index on the live MobileCoin consensus
    /// network.
    fn get_network_block_index(&self) -> Result<u64, LedgerServiceError>;

    fn get_transaction_object(&self, transaction_id_hex: &str) -> Result<Tx, LedgerServiceError>;

    fn get_txo_object(&self, txo_id_hex: &str) -> Result<TxOut, LedgerServiceError>;

    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(Block, BlockContents), LedgerServiceError>;

    fn contains_key_image(&self, key_image: &KeyImage) -> Result<bool, LedgerServiceError>;

    fn get_network_fee(&self) -> u64;
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

        let txo: TxOut = mc_util_serial::decode(&txo_details.txo.txo)?;
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
        Ok(self.ledger_db.contains_key_image(&key_image)?)
    }

    fn get_network_fee(&self) -> u64 {
        if self.peer_manager.is_empty() {
            MINIMUM_FEE
        } else {
            // Iterate an owned list of connections in parallel, get the block info for
            // each, and extract the fee. If no fees are returned, use the hard-coded
            // minimum.
            self.peer_manager
                .conns()
                .par_iter()
                .filter_map(|conn| conn.fetch_block_info(empty()).ok())
                .filter_map(|block_info| {
                    // Cleanup the protobuf default fee
                    if block_info.minimum_fee == 0 {
                        None
                    } else {
                        Some(block_info.minimum_fee)
                    }
                })
                .max()
                .unwrap_or(MINIMUM_FEE)
        }
    }
}
