// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use crate::{
    db::{
        models::{TransactionLog, Txo},
        transaction_log::{TransactionID, TransactionLogModel},
        txo::TxoModel,
    },
    service::models::ledger::LedgerSearchResult,
    WalletService,
};
use mc_blockchain_types::{Block, BlockContents, BlockVersion, BlockVersionError};
use mc_common::HashSet;
use mc_connection::{
    BlockInfo, BlockchainConnection, RetryableBlockchainConnection, UserTxConnection,
};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Error as LedgerError, Ledger, LedgerDB};
use mc_ledger_sync::NetworkState;
use mc_transaction_core::{
    ring_signature::KeyImage,
    tx::{Tx, TxOut, TxOutMembershipProof},
    TokenId,
};
use rand::Rng;

use crate::db::WalletDbError;
use displaydoc::Display;
use rayon::prelude::*; // For par_iter
use std::{collections::BTreeMap, convert::TryFrom};

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

    /// Error converting from hex string to bytes.
    FromHex(hex::FromHexError),

    /// Key Error from mc_crypto_keys
    Key(mc_crypto_keys::KeyError),

    /// Invalid Argument: {0}
    InvalidArgument(String),

    /// Insufficient Tx Outs
    InsufficientTxOuts,

    /// No node responded to the last block info request
    NoLastBlockInfo,

    /// Inconsistent last block info
    InconsistentLastBlockInfo,

    /// Block version: {0}
    BlockVersion(BlockVersionError),

    /// Ledger inconsistent
    LedgerInconsistent,
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

impl From<hex::FromHexError> for LedgerServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::FromHex(src)
    }
}

impl From<mc_crypto_keys::KeyError> for LedgerServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::Key(src)
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

    fn get_block_objects(
        &self,
        first_block_index: u64,
        limit: usize,
    ) -> Result<Vec<(Block, BlockContents)>, LedgerServiceError>;

    fn get_recent_block_objects(
        &self,
        limit: usize,
    ) -> Result<Vec<(Block, BlockContents)>, LedgerServiceError>;

    fn contains_key_image(&self, key_image: &KeyImage) -> Result<bool, LedgerServiceError>;

    fn get_latest_block_info(&self) -> Result<BlockInfo, LedgerServiceError>;

    fn get_network_fees(&self) -> Result<BTreeMap<TokenId, u64>, LedgerServiceError>;

    fn get_network_block_version(&self) -> Result<BlockVersion, LedgerServiceError>;

    fn get_tx_out_proof_of_memberships(
        &self,
        indices: &[u64],
    ) -> Result<Vec<TxOutMembershipProof>, LedgerServiceError>;

    fn get_indices_from_txo_public_keys(
        &self,
        public_keys: &[CompressedRistrettoPublic],
    ) -> Result<Vec<u64>, LedgerServiceError>;

    fn sample_mixins(
        &self,
        num_mixins: usize,
        excluded_indices: &[u64],
    ) -> Result<(Vec<TxOut>, Vec<TxOutMembershipProof>), LedgerServiceError>;

    fn get_block_index_from_txo_public_key(
        &self,
        public_key: &CompressedRistrettoPublic,
    ) -> Result<u64, LedgerServiceError>;

    fn search_ledger(&self, query: &str) -> Result<Vec<LedgerSearchResult>, LedgerServiceError>;
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
        let conn = self.get_conn()?;
        let transaction_log =
            TransactionLog::get(&TransactionID(transaction_id_hex.to_string()), &conn)?;
        let tx: Tx = mc_util_serial::decode(&transaction_log.tx)?;
        Ok(tx)
    }

    fn get_txo_object(&self, txo_id_hex: &str) -> Result<TxOut, LedgerServiceError> {
        let conn = self.get_conn()?;
        let txo_details = Txo::get(txo_id_hex, &conn)?;
        let txo = self.ledger_db.get_tx_out_by_index(
            self.ledger_db
                .get_tx_out_index_by_public_key(&txo_details.public_key()?)?,
        )?;
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

    fn get_block_objects(
        &self,
        first_block_index: u64,
        limit: usize,
    ) -> Result<Vec<(Block, BlockContents)>, LedgerServiceError> {
        let mut results = vec![];

        let last_block_index = first_block_index.saturating_add(limit as u64);

        for block_index in first_block_index..last_block_index {
            let block = match self.ledger_db.get_block(block_index) {
                Ok(block) => block,
                Err(LedgerError::NotFound) => break,
                Err(err) => return Err(LedgerServiceError::from(err)),
            };
            let block_contents = self.ledger_db.get_block_contents(block_index)?;
            results.push((block, block_contents));
        }

        Ok(results)
    }

    fn get_recent_block_objects(
        &self,
        limit: usize,
    ) -> Result<Vec<(Block, BlockContents)>, LedgerServiceError> {
        let latest_block_index = self.ledger_db.num_blocks()?.checked_sub(1).ok_or_else(|| {
            LedgerServiceError::InvalidArgument("No blocks in ledger".to_string())
        })?;
        let mut block_index = latest_block_index;
        let mut results = vec![];

        loop {
            if results.len() >= limit {
                break;
            }

            let block = self.ledger_db.get_block(block_index)?;
            let block_contents = self.ledger_db.get_block_contents(block_index)?;
            results.push((block, block_contents));

            if block_index == 0 {
                break;
            }

            block_index -= 1;
        }

        Ok(results)
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
            .filter_map(|conn| conn.fetch_block_info(std::iter::empty()).ok())
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

    fn get_network_fees(&self) -> Result<BTreeMap<TokenId, u64>, LedgerServiceError> {
        Ok(self.get_latest_block_info()?.minimum_fees)
    }

    fn get_network_block_version(&self) -> Result<BlockVersion, LedgerServiceError> {
        Ok(BlockVersion::try_from(
            self.get_latest_block_info()?.network_block_version,
        )?)
    }

    fn get_tx_out_proof_of_memberships(
        &self,
        indices: &[u64],
    ) -> Result<Vec<TxOutMembershipProof>, LedgerServiceError> {
        Ok(self.ledger_db.get_tx_out_proof_of_memberships(indices)?)
    }

    fn get_indices_from_txo_public_keys(
        &self,
        public_keys: &[CompressedRistrettoPublic],
    ) -> Result<Vec<u64>, LedgerServiceError> {
        let indices = public_keys
            .iter()
            .map(|public_key| self.ledger_db.get_tx_out_index_by_public_key(public_key))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(indices)
    }

    fn sample_mixins(
        &self,
        num_mixins: usize,
        excluded_indices: &[u64],
    ) -> Result<(Vec<TxOut>, Vec<TxOutMembershipProof>), LedgerServiceError> {
        let num_txos = self.ledger_db.num_txos()?;

        // Check that the ledger contains enough tx outs.
        if excluded_indices.len() as u64 > num_txos {
            return Err(LedgerServiceError::InvalidArgument(
                "excluded_tx_out_indices exceeds amount of tx outs in ledger".to_string(),
            ));
        }

        if num_mixins > (num_txos as usize - excluded_indices.len()) {
            return Err(LedgerServiceError::InsufficientTxOuts);
        }

        let mut rng = rand::thread_rng();
        let mut sampled_indices: HashSet<u64> = HashSet::default();
        while sampled_indices.len() < num_mixins {
            let index = rng.gen_range(0..num_txos);
            if excluded_indices.contains(&index) {
                continue;
            }
            sampled_indices.insert(index);
        }
        let sampled_indices_vec: Vec<u64> = sampled_indices.into_iter().collect();

        // Get proofs for all of those indexes.
        let proofs = self
            .ledger_db
            .get_tx_out_proof_of_memberships(&sampled_indices_vec)?;

        let tx_outs = sampled_indices_vec
            .iter()
            .map(|index| self.ledger_db.get_tx_out_by_index(*index))
            .collect::<Result<Vec<TxOut>, _>>()?;

        Ok((tx_outs, proofs))
    }

    fn get_block_index_from_txo_public_key(
        &self,
        public_key: &CompressedRistrettoPublic,
    ) -> Result<u64, LedgerServiceError> {
        let index = self.ledger_db.get_tx_out_index_by_public_key(public_key)?;
        Ok(self.ledger_db.get_block_index_by_tx_out_index(index)?)
    }

    fn search_ledger(&self, query: &str) -> Result<Vec<LedgerSearchResult>, LedgerServiceError> {
        let mut results = vec![];

        // Try intepreting the query as a hex string.
        if let Some(mut query_bytes) = hex::decode(query).ok() {
            // Hack - strip away the protobuf header if we have one. This hack
            // is needed because sometimes full-service returns protobuf-encoded
            // bytes instead of just returning the raw bytes.
            // See https://github.com/mobilecoinofficial/full-service/issues/201
            // The protobuf encoded bytes start with 0x0a20
            if query_bytes.len() == 34 && query_bytes[0] == 0x0a && query_bytes[1] == 0x20 {
                query_bytes.remove(0);
                query_bytes.remove(0);
            }

            // Try and search for a tx out by public key.
            if let Some(result) = search_ledger_by_tx_out_pub_key(&self.ledger_db, &query_bytes)? {
                results.push(result);
            }

            // Try and search for a key image.
            if let Some(result) = search_ledger_by_key_image(&self.ledger_db, &query_bytes)? {
                results.push(result);
            }
        }

        Ok(results)
    }
}

pub fn get_tx_out_by_public_key(
    ledger_db: &LedgerDB,
    public_key: &CompressedRistrettoPublic,
) -> Result<TxOut, LedgerServiceError> {
    let txo_index = ledger_db.get_tx_out_index_by_public_key(public_key)?;
    let txo = ledger_db.get_tx_out_by_index(txo_index)?;
    Ok(txo)
}

fn search_ledger_by_tx_out_pub_key(
    ledger_db: &LedgerDB,
    query_bytes: &[u8],
) -> Result<Option<LedgerSearchResult>, LedgerServiceError> {
    let public_key = if let Ok(pk) = CompressedRistrettoPublic::try_from(&query_bytes[..]) {
        pk
    } else {
        return Ok(None);
    };

    let tx_out_global_index = match ledger_db.get_tx_out_index_by_public_key(&public_key) {
        Ok(index) => index,
        Err(LedgerError::NotFound) => return Ok(None),
        Err(e) => return Err(LedgerServiceError::from(e)),
    };

    let block_index = ledger_db.get_block_index_by_tx_out_index(tx_out_global_index)?;

    let block = ledger_db.get_block(block_index)?;

    let block_contents = ledger_db.get_block_contents(block_index)?;

    let block_contents_tx_out_index = block_contents
        .outputs
        .iter()
        .position(|tx_out| tx_out.public_key == public_key)
        .ok_or(LedgerServiceError::LedgerInconsistent)?
        as u64;

    Ok(Some(LedgerSearchResult::TxOut {
        block,
        block_contents,
        block_contents_tx_out_index,
        tx_out_global_index,
    }))
}

fn search_ledger_by_key_image(
    ledger_db: &LedgerDB,
    query_bytes: &[u8],
) -> Result<Option<LedgerSearchResult>, LedgerServiceError> {
    let key_image = if let Ok(ki) = KeyImage::try_from(&query_bytes[..]) {
        ki
    } else {
        return Ok(None);
    };

    let block_index = if let Some(idx) = ledger_db.check_key_image(&key_image)? {
        idx
    } else {
        return Ok(None);
    };

    let block = ledger_db.get_block(block_index)?;

    let block_contents = ledger_db.get_block_contents(block_index)?;

    let block_contents_key_image_index = block_contents
        .key_images
        .iter()
        .position(|key_image2| key_image2 == &key_image)
        .ok_or(LedgerServiceError::LedgerInconsistent)?
        as u64;

    Ok(Some(LedgerSearchResult::KeyImage {
        block,
        block_contents,
        block_contents_key_image_index,
    }))
}
