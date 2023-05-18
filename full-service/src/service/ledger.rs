// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use crate::{
    db::{
        models::{TransactionLog, Txo},
        transaction_log::{TransactionId, TransactionLogModel},
        txo::TxoModel,
    },
    service::{
        models::ledger::LedgerSearchResult,
        watcher::{WatcherService, WatcherServiceError},
    },
    WalletService,
};
use mc_blockchain_types::{Block, BlockContents, BlockVersion, BlockVersionError};
use mc_common::HashSet;
use mc_connection::{
    BlockInfo, BlockchainConnection, RetryableBlockchainConnection, UserTxConnection,
    _retry::delay::Fibonacci,
};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Error as LedgerError, Ledger, LedgerDB};
use mc_ledger_sync::NetworkState;
use mc_transaction_core::{
    ring_signature::KeyImage,
    tx::{Tx, TxOut, TxOutMembershipProof},
    FeeMap, FeeMapError,
};
use mc_watcher::error::WatcherDBError;
use rand::Rng;

use crate::db::WalletDbError;
use displaydoc::Display;
use rayon::prelude::*; // For par_iter
use std::{convert::{TryFrom, TryInto}, ops::DerefMut};

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

    /// Error with FeeMap: {0}
    FeeMap(FeeMapError),

    /// Error interacting with watcher database: {0}
    WatcherDb(WatcherDBError),
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

impl From<FeeMapError> for LedgerServiceError {
    fn from(src: FeeMapError) -> Self {
        Self::FeeMap(src)
    }
}

impl From<WatcherServiceError> for LedgerServiceError {
    fn from(src: WatcherServiceError) -> Self {
        match src {
            WatcherServiceError::WatcherDb(err) => Self::WatcherDb(err),
        }
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// ledger objects and interfaces.
#[rustfmt::skip]
pub trait LedgerService {
    /// Get the total number of blocks on the ledger.
    fn get_network_block_height(&self) -> Result<u64, LedgerServiceError>;

    /// Get the JSON representation of the TXO object in the transaction log
    ///
    /// # Arguments
    ///
    ///| Name                 | Purpose                        | Notes                                     |
    ///|----------------------|--------------------------------|-------------------------------------------|
    ///| `transaction_id_hex` | The transaction log ID to get. | Transaction log must exist in the wallet. |
    ///
    fn get_transaction_object(
        &self, 
        transaction_id_hex: &str
    ) -> Result<Tx, LedgerServiceError>;
    
    /// Get details of a given TXO.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                              | Notes |
    ///|--------------|--------------------------------------|-------|
    ///| `txo_id_hex` | The TXO ID for which to get details. |       |
    ///
    fn get_txo_object(
        &self, 
        txo_id_hex: &str
    ) -> Result<TxOut, LedgerServiceError>;

    /// Get block contents for a given block.
    ///
    /// # Arguments
    ///
    ///| Name          | Purpose                                    | Notes                           |
    ///|---------------|--------------------------------------------|---------------------------------|
    ///| `block_index` | The block on which to perform this action. | Block must exist in the wallet. |
    ///
    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(Block, BlockContents), LedgerServiceError>;

    /// Get block contents for a list of blocks starting from a given block.
    ///
    /// # Arguments
    ///
    ///| Name                | Purpose                                            | Notes |
    ///|---------------------|----------------------------------------------------|-------|
    ///| `first_block_index` | The block from which to start scanning the ledger. |       |
    ///| `limit`             | Limit for the number of results.                   |       |
    ///
    fn get_block_objects(
        &self,
        first_block_index: u64,
        limit: usize,
    ) -> Result<Vec<(Block, BlockContents)>, LedgerServiceError>;

    /// Get block contents for a list of blocks starting from the most recent block.
    ///
    /// # Arguments
    ///
    ///| Name                | Purpose                          | Notes |
    ///|---------------------|----------------------------------|-------|
    ///| `limit`             | Limit for the number of results. |       |
    ///
    fn get_recent_block_objects(
        &self,
        limit: usize,
    ) -> Result<Vec<(Block, BlockContents)>, LedgerServiceError>;

    /// Returns true if the Ledger contains the given key image.
    ///
    /// # Arguments
    ///
    ///| Name                | Purpose | Notes |
    ///|---------------------|---------|-------|
    ///| `key_image`         |         |       |
    ///
    fn contains_key_image(
        &self, 
        key_image: &KeyImage
    ) -> Result<bool, LedgerServiceError>;
    
    /// Get the last block information cross all nodes
    fn get_latest_block_info(&self) -> Result<BlockInfo, LedgerServiceError>;

    /// Get an object for fees in each of the configured token types
    fn get_network_fees(&self) -> Result<FeeMap, LedgerServiceError>;

    /// Get block version info from the latest block
    fn get_network_block_version(&self) -> Result<BlockVersion, LedgerServiceError>;

    /// Get a proof of memberships for TxOuts with indexes `indices`.
    ///
    /// # Arguments
    ///
    ///| Name                | Purpose                             | Notes |
    ///|---------------------|-------------------------------------|-------|
    ///| `indices`           | The index of the TXO in the ledger. |       |
    ///
    fn get_tx_out_proof_of_memberships(
        &self,
        indices: &[u64],
    ) -> Result<Vec<TxOutMembershipProof>, LedgerServiceError>;

    /// Get a list of indices of TXO in the ledger from TxOut publc keys.
    ///
    /// # Arguments
    ///
    ///| Name          | Purpose                                   | Notes |
    ///|---------------|-------------------------------------------|-------|
    ///| `public_keys` | The public keys of the TXO in the ledger. |       |
    ///
    fn get_indices_from_txo_public_keys(
        &self,
        public_keys: &[CompressedRistrettoPublic],
    ) -> Result<Vec<u64>, LedgerServiceError>;

    /// Get a list of indices of TXO in the ledger from TxOut publc keys.
    ///
    /// # Arguments
    ///
    ///| Name          | Purpose                                   | Notes |
    ///|---------------|-------------------------------------------|-------|
    ///| `public_keys` | The public keys of the TXO in the ledger. |       |
    ///
    fn get_block_index_from_txo_public_key(
        &self,
        public_key: &CompressedRistrettoPublic,
    ) -> Result<u64, LedgerServiceError>;

    /// Sample a desired number of mixins from the ledger, excluding a list of tx outs
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                  | Notes                                                                               |
    ///|--------------------|------------------------------------------|-------------------------------------------------------------------------------------|
    ///| `num_mixins`       | The number of mixins to sample           | Must be less than the number of txos in the ledger minus number of excluded outputs |
    ///| `excluded_indices` | Indices of Txos to exclude from sampling | Txo must exist in the ledger                                                        |
    ///
    fn sample_mixins(
        &self,
        num_mixins: usize,
        excluded_indices: &[u64],
    ) -> Result<(Vec<TxOut>, Vec<TxOutMembershipProof>), LedgerServiceError>;

    /// Search the ledger for blocks based on a query string (that can be either a block index, a tx out public key, or a key image)
    ///
    /// # Arguments
    ///
    ///| Name    | Purpose                     | Notes                                                                                                            |
    ///|---------|-----------------------------|------------------------------------------------------------------------------------------------------------------|
    ///| `query` | Query string to search for. | Currently the supported queries are a block index, or hex representations of a tx out public key or a key image. |
    ///
    fn search_ledger(
        &self, 
        query: &str
    ) -> Result<Vec<LedgerSearchResult>, LedgerServiceError>;
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
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let transaction_log =
            TransactionLog::get(&TransactionId(transaction_id_hex.to_string()), conn)?;
        let tx: Tx = mc_util_serial::decode(&transaction_log.tx)?;
        Ok(tx)
    }

    fn get_txo_object(&self, txo_id_hex: &str) -> Result<TxOut, LedgerServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let txo_details = Txo::get(txo_id_hex, conn)?;
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

    fn get_network_fees(&self) -> Result<FeeMap, LedgerServiceError> {
        Ok(FeeMap::try_from(
            self.get_latest_block_info()?.minimum_fees,
        )?)
    }

    fn get_network_block_version(&self) -> Result<BlockVersion, LedgerServiceError> {
        // If we are in offline mode, get the last block information from the last
        // synced block
        if self.offline {
            let num_blocks = self.ledger_db.num_blocks()?;
            if num_blocks < 1 {
                return Err(LedgerServiceError::NoLastBlockInfo);
            }

            let last_block = self.ledger_db.get_block(num_blocks - 1)?;
            let version = Ok(BlockVersion::try_from(last_block.version)?);
            return version;
        }

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
        if let Ok(mut query_bytes) = hex::decode(query) {
            // Hack - strip away the protobuf header if we have one. This hack
            // is needed because sometimes full-service returns protobuf-encoded
            // bytes instead of just returning the raw bytes.
            // See https://github.com/mobilecoinofficial/full-service/issues/201
            // The protobuf encoded bytes start with 0x0a20
            if query_bytes.len() == 34 && query_bytes[0] == 0x0a && query_bytes[1] == 0x20 {
                query_bytes.remove(0);
                query_bytes.remove(0);
            }

            // Search for a tx out by public key.
            if let Some(result) = self.search_ledger_by_tx_out_pub_key(&query_bytes)? {
                results.push(result);
            }

            // Search for a key image.
            if let Some(result) = self.search_ledger_by_txo_hash(&query_bytes)? {
                results.push(result);
            }

            // Search for a key image.
            if let Some(result) = self.search_ledger_by_key_image(&query_bytes)? {
                results.push(result);
            }
        }

        Ok(results)
    }
}

impl<T, FPR> WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn search_ledger_by_tx_out_pub_key(
        &self,
        query_bytes: &[u8],
    ) -> Result<Option<LedgerSearchResult>, LedgerServiceError> {
        let public_key = if let Ok(pk) = CompressedRistrettoPublic::try_from(query_bytes) {
            pk
        } else {
            return Ok(None);
        };

        let tx_out_global_index = match self.ledger_db.get_tx_out_index_by_public_key(&public_key) {
            Ok(index) => index,
            Err(LedgerError::NotFound) => return Ok(None),
            Err(e) => return Err(LedgerServiceError::from(e)),
        };

        let block_index = self
            .ledger_db
            .get_block_index_by_tx_out_index(tx_out_global_index)?;

        let block = self.ledger_db.get_block(block_index)?;

        let block_contents = self.ledger_db.get_block_contents(block_index)?;

        let block_contents_tx_out_index = block_contents
            .outputs
            .iter()
            .position(|tx_out| tx_out.public_key == public_key)
            .ok_or(LedgerServiceError::LedgerInconsistent)?
            as u64;

        let watcher_info = self.get_watcher_block_info(block_index)?;

        Ok(Some(LedgerSearchResult::TxOut {
            block,
            block_contents,
            block_contents_tx_out_index,
            tx_out_global_index,
            watcher_info,
        }))
    }

    fn search_ledger_by_txo_hash(
        &self,
        query_bytes: &[u8],
    ) -> Result<Option<LedgerSearchResult>, LedgerServiceError> {
        // // get any txo_id
        // let block_contents = self.ledger_db.get_block_contents(1)?;
        // let any_hash = block_contents.outputs[0].hash();
        // dbg!(hex::encode(&any_hash));
        // return Ok(None);

        let txo_id: [u8; 32] = match (*query_bytes).try_into() {
            Ok(array_ref) => array_ref,
            Err(_) => return Ok(None)
        };
        dbg!(&txo_id);

        let tx_out_global_index = match self.ledger_db.get_tx_out_index_by_hash(&txo_id) {
            Ok(index) => index,
            Err(_) => return Ok(None)
        };
        dbg!(tx_out_global_index);

        let block_index = self
            .ledger_db
            .get_block_index_by_tx_out_index(tx_out_global_index)?;

        let block = self.ledger_db.get_block(block_index)?;

        let block_contents = self.ledger_db.get_block_contents(block_index)?;

        let block_contents_tx_out_index = block_contents
            .outputs
            .iter()
            .position(|tx_out| tx_out.hash() == txo_id)
            .ok_or(LedgerServiceError::LedgerInconsistent)?
            as u64;

        let watcher_info = self.get_watcher_block_info(block_index)?;

        Ok(Some(LedgerSearchResult::TxOut {
            block,
            block_contents,
            block_contents_tx_out_index,
            tx_out_global_index,
            watcher_info,
        }))
    }

    fn search_ledger_by_key_image(
        &self,
        query_bytes: &[u8],
    ) -> Result<Option<LedgerSearchResult>, LedgerServiceError> {
        let key_image = if let Ok(ki) = KeyImage::try_from(query_bytes) {
            ki
        } else {
            return Ok(None);
        };

        let block_index = if let Some(idx) = self.ledger_db.check_key_image(&key_image)? {
            idx
        } else {
            return Ok(None);
        };

        let block = self.ledger_db.get_block(block_index)?;

        let block_contents = self.ledger_db.get_block_contents(block_index)?;

        let block_contents_key_image_index = block_contents
            .key_images
            .iter()
            .position(|key_image2| key_image2 == &key_image)
            .ok_or(LedgerServiceError::LedgerInconsistent)?
            as u64;

        let watcher_info = self.get_watcher_block_info(block_index)?;

        Ok(Some(LedgerSearchResult::KeyImage {
            block,
            block_contents,
            block_contents_key_image_index,
            watcher_info,
        }))
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
