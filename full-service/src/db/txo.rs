// Copyright (c) 2020-2023 MobileCoin Inc.

//! DB impl for the Txo model.

use diesel::{
    dsl::{count, exists, not},
    prelude::*,
};
use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::{logger::global_log, HashMap};
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_core::{
    constants::MAX_INPUTS,
    ring_signature::KeyImage,
    tx::{TxOut, TxOutMembershipProof},
    Amount, TokenId,
};
use mc_transaction_extra::{
    AuthenticatedSenderMemo, AuthenticatedSenderWithPaymentIntentIdMemo,
    AuthenticatedSenderWithPaymentRequestIdMemo, MemoType, RegisteredMemoType,
    TxOutConfirmationNumber, UnusedMemo,
};
use mc_util_serial::Message;
use std::{convert::TryFrom, fmt, str::FromStr};

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{
            Account, AssignedSubaddress, AuthenticatedSenderMemo as AuthenticatedSenderMemoModel,
            NewAuthenticatedSenderMemo, NewTransactionOutputTxo, NewTxo, TransactionOutputTxo, Txo,
        },
        transaction_log::TransactionId,
        Conn, WalletDbError,
    },
    service::models::tx_proposal::OutputTxo,
    util::b58::b58_encode_public_address,
};

#[derive(Debug, PartialEq)]
pub enum TxoStatus {
    // The txo has been created as part of build-transaction, but its associated transaction is
    // neither pending, or submitted successfully
    Created,

    // The txo has been received but the subaddress index and key image cannot be determined. This
    // happens typically when an account is imported but all subaddresses it was using were not
    // recreated
    Orphaned,

    // The txo is part of a pending transaction
    Pending,

    // The txo is created and released as part of a successful transaction, but we do not know
    // what block it was received at, the subaddress, keyimage, nor spent block. For example,
    // the txo going to a recipient's account is often secreted.
    Secreted,

    // The txo has a known spent block index
    Spent,

    // The txo has been received at a known subaddress index with a known key image, has not been
    // spent, and is not part of a pending transaction
    Unspent,

    // The txo has been received at a known subaddress index, but the key image cannot
    // be derived (usually because this is a view only account)
    Unverified,
}

#[derive(Debug, PartialEq)]
pub enum TxoMemo {
    Unused,
    AuthenticatedSender(AuthenticatedSenderMemoModel),
}

impl fmt::Display for TxoStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxoStatus::Created => write!(f, "created"),
            TxoStatus::Orphaned => write!(f, "orphaned"),
            TxoStatus::Pending => write!(f, "pending"),
            TxoStatus::Secreted => write!(f, "secreted"),
            TxoStatus::Spent => write!(f, "spent"),
            TxoStatus::Unspent => write!(f, "unspent"),
            TxoStatus::Unverified => write!(f, "unverified"),
        }
    }
}

impl FromStr for TxoStatus {
    type Err = WalletDbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(TxoStatus::Created),
            "orphaned" => Ok(TxoStatus::Orphaned),
            "pending" => Ok(TxoStatus::Pending),
            "secreted" => Ok(TxoStatus::Secreted),
            "spent" => Ok(TxoStatus::Spent),
            "unspent" => Ok(TxoStatus::Unspent),
            "unverified" => Ok(TxoStatus::Unverified),
            _ => Err(WalletDbError::InvalidTxoStatus(s.to_string())),
        }
    }
}

/// A unique ID derived from a TxOut in the ledger.
#[derive(Debug)]
pub struct TxoID(pub String);

impl From<&TxOut> for TxoID {
    fn from(src: &TxOut) -> TxoID {
        let digest: [u8; 32] = src.digest32::<MerlinTranscript>(b"txo_data");
        Self(hex::encode(digest))
    }
}

impl From<&Txo> for TxoID {
    fn from(src: &Txo) -> TxoID {
        Self(src.id.clone())
    }
}

impl fmt::Display for TxoID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct SpendableTxosResult {
    pub spendable_txos: Vec<Txo>,
    pub max_spendable_in_wallet: u128,
}

impl Txo {
    pub fn amount(&self) -> Amount {
        Amount::new(self.value as u64, TokenId::from(self.token_id as u64))
    }
}

pub struct TxoInfo {
    pub txo: Txo,
    pub memo: TxoMemo,
    pub status: TxoStatus,
}

#[rustfmt::skip]
pub trait TxoModel {
    /// Saves a received TxOut to local database.
    ///
    /// The subaddress_index may be None, and the Txo is said to be "orphaned",
    /// if the subaddress is not yet being tracked by the wallet.
    /// 
    /// # Arguments
    /// 
    ///| Name                   | Purpose                                                                         | Notes           |
    ///|------------------------|---------------------------------------------------------------------------------|-----------------|
    ///| `tx_out`               | This is a TxOut object contained in the ledger                                  |                 |
    ///| `subaddress_index`     | The assigned subaddress index for this TXO with respect to its received account | Assign if known |
    ///| `key_image`            | The fingerprint of the TxOut                                                    |                 |
    ///| `amount`               | The value in this TxOut                                                         | Unit in picoMob |
    ///| `received_block_index` | The index of the block at which this TxOut was received.                        |                 |
    ///| `account_id_hex`       | The account ID for the account which received this TxOut                        |                 |
    ///| `conn`                 | An reference to the pool connection of wallet database                          |                 |
    ///
    /// # Returns
    /// * txo_id_hex
    fn create_received(
        tx_out: TxOut,
        subaddress_index: Option<u64>,
        key_image: Option<KeyImage>,
        amount: Amount,
        received_block_index: u64,
        account_id_hex: &str,
        conn: Conn,
    ) -> Result<String, WalletDbError>;


    /// Create a TxOut payload and insert to local database.
    /// 
    /// # Arguments
    /// 
    ///| Name             | Purpose                                                                               | Notes                                               |
    ///|------------------|---------------------------------------------------------------------------------------|-----------------------------------------------------|
    ///| `output_txo`     | This is the transaction output TxOut that will be insert to database                  | Either a change or payload transaction output TxOut |
    ///| `is_change`      | A boolean value to indicate if this transaction output TxOut is a change or a payload | Assign if known                                     |
    ///| `transaction_id` | The transaction id at which the transaction output TxOut associates with              |                                                     |
    ///| `conn`           | An reference to the pool connection of wallet database                                |                                                     |
    ///
    /// # Returns
    /// * unit
    fn create_new_output(
        output_txo: &OutputTxo,
        is_change: bool,
        transaction_id: &TransactionId,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Update an existing Txo to spendable by including its subaddress_index
    /// and optionally the key_image in the case of view only accounts.
    /// 
    /// # Arguments
    /// 
    ///| Name                 | Purpose                                                                                                         | Notes           |
    ///|----------------------|-----------------------------------------------------------------------------------------------------------------|-----------------|
    ///| `subaddress_index`   | The index of the subaddress that will be added to current TxOut                                                 |                 |
    ///| `received_key_image` | The fingerprint of the current TxOut                                                                            |                 |
    ///| `block_index`        | The index of block at which the current TxOut is recevied                                                       |                 |
    ///| `account_id_hex`     | The account ID for the account which received this TxOut                                                        |                 |
    ///| `amount`             | The value in this TxOut                                                                                         | Unit in picoMob |
    ///| `target_key`         | The one-time public address of this txo.                                                                        |                 |
    ///| `public_key`         | The per output tx public key                                                                                    |                 |
    ///| `e_fog_hint`         | The encrypted fog hint for the fog ingest server.                                                               |                 |
    ///| `shared_secret`      | A cryptographic key shared between the sender and recipient that is used to decrypt the TxOut's amount and memo |                 |
    ///| `conn`               | An reference to the pool connection of wallet database                                                          |                 |
    ///
    /// # Returns
    /// * unit
    #[allow(clippy::too_many_arguments)]
    fn update_as_received(
        &self,
        subaddress_index: Option<u64>,
        received_key_image: Option<KeyImage>,
        block_index: u64,
        account_id_hex: &str,
        amount: Amount,
        target_key: &[u8],
        public_key: &[u8],
        e_fog_hint: &[u8],
        shared_secret: Option<&[u8]>,
        memo_type: Option<i32>,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Update a Txo's status to spent
    /// 
    /// # Arguments
    /// 
    ///| Name                | Purpose                                                | Notes |
    ///|---------------------|--------------------------------------------------------|-------|
    ///| `txo_id_hex`        | The id of the TxOut object that will be updated        |       |
    ///| `spent_block_index` | The index of block where the TxOut was spent           |       |
    ///| `conn`              | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns
    /// * unit
    fn update_spent_block_index(
        txo_id_hex: &str,
        spent_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Update a Txo's key image and optionally update its status to spent
    /// 
    /// # Arguments
    /// 
    ///| Name                | Purpose                                                | Notes |
    ///|---------------------|--------------------------------------------------------|-------|
    ///| `txo_id_hex`        | The id of the TxOut object that will be updated        |       |
    ///| `key_image`         | The fingerprint of the TxOut                           |       |
    ///| `spent_block_index` | The index of block where the TxOut was spent           |       |
    ///| `conn`              | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns
    /// * unit
    fn update_key_image(
        txo_id_hex: &str,
        key_image: &KeyImage,
        spent_block_index: Option<u64>,
        conn: Conn,
    ) -> Result<(), WalletDbError>;


    fn update_is_synced_to_t3(&self, is_synced: bool, conn: Conn) -> Result<(), WalletDbError>;

    fn get_txos_that_need_to_be_synced_to_t3(
        limit: Option<usize>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get a list of TxOut within the given conditions
    /// 
    /// # Arguments
    /// 
    ///| Name                       | Purpose                                                       | Notes                                                                                    |
    ///|----------------------------|---------------------------------------------------------------|------------------------------------------------------------------------------------------|
    ///| `status`                   | The status of Txos to filter on                               | Option in `Created`, `Orphaned`, `Pending`, `Secreted`, `Spent`, `Unspent`, `Unverified` |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                                                                          |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                                                                          |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.                                                                 |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                                                                                |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                                                                          |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                                                                          |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    fn list(
        status: Option<TxoStatus>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all Txos associated with a given account.
    /// 
    /// # Arguments
    ///
    ///| Name                       | Purpose                                                       | Notes                                                                                    |
    ///|----------------------------|---------------------------------------------------------------|------------------------------------------------------------------------------------------|
    ///| `account_id_hex`           | The account id where the list of Txos from                    | Account must exist in the database.                                                      |
    ///| `status`                   | The status of Txos to filter on                               | Option in `Created`, `Orphaned`, `Pending`, `Secreted`, `Spent`, `Unspent`, `Unverified` |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                                                                          |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                                                                          |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.                                                                 |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                                                                                |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                                                                          |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                                                                          |
    ///
    /// # Returns
    /// * Vector of TxoOut
    #[allow(clippy::too_many_arguments)]
    fn list_for_account(
        account_id_hex: &str,
        status: Option<TxoStatus>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all Txos associated with an assigned subaddress
    /// 
    /// # Arguments
    /// 
    ///| Name                       | Purpose                                                       | Notes                                                                                    |
    ///|----------------------------|---------------------------------------------------------------|------------------------------------------------------------------------------------------|
    ///| `assigned_subaddress_b58`  | The subaddress where the list of Txos from                    |                                                                                          |
    ///| `status`                   | The status of Txos to filter on                               | Option in `Created`, `Orphaned`, `Pending`, `Secreted`, `Spent`, `Unspent`, `Unverified` |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                                                                          |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                                                                          |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.                                                                 |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                                                                                |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                                                                          |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                                                                          |
    ///
    /// # Returns
    /// * Vector of TxoOut
    #[allow(clippy::too_many_arguments)]
    fn list_for_address(
        assigned_subaddress_b58: &str,
        status: Option<TxoStatus>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get a map from key images to unspent txos for this account.
    /// 
    /// # Arguments
    /// 
    ///| Name             | Purpose                                                | Notes                               |
    ///|------------------|--------------------------------------------------------|-------------------------------------|
    ///| `account_id_hex` | The account id where the key images and Txos from      | Account must exist in the database. |
    ///| `token_id`       | The id of a supported type of token to filter on       |                                     |
    ///| `conn`           | An reference to the pool connection of wallet database |                                     |
    ///
    /// # Returns
    /// * A hashmap of a KeyImage key and a TxOut id string
    fn list_unspent_or_pending_key_images(
        account_id_hex: &str,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<HashMap<KeyImage, String>, WalletDbError>;

    /// Get all unspent Txos associated  with an account or an assigned subaddress
    /// 
    /// # Arguments
    ///
    ///| Name                       | Purpose                                                       | Notes                                                                                    |
    ///|----------------------------|---------------------------------------------------------------|------------------------------------------------------------------------------------------|
    ///| `account_id_hex`           | The account id where the list of Txos from                    | Account must exist in the database.                                                      |
    ///| `assigned_subaddress_b58`  | The subaddress where the list of Txos from                    |                                                                                          |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                                                                          |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                                                                          |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                                                                          |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.                                                                 |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                                                                                |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                                                                          |
    ///
    /// # Returns
    /// * Vector of TxoOut
    #[allow(clippy::too_many_arguments)]
    fn list_unspent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all spent Txos associated  with an account or an assigned subaddress
    /// 
    /// # Arguments
    /// 
    ///| Name                       | Purpose                                                       | Notes                                |
    ///|----------------------------|---------------------------------------------------------------|--------------------------------------|
    ///| `account_id_hex`           | The account id where the list of Txos from                    | Account must exist in the database.  |
    ///| `assigned_subaddress_b58`  | The subaddress where the list of Txos from                    |                                      |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                      |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                      |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                      |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.             |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                            |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                      |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    #[allow(clippy::too_many_arguments)]
    fn list_spent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all orphaned Txos associated with an account or an assigned subaddress
    /// 
    /// # Arguments
    /// 
    ///| Name                       | Purpose                                                       | Notes                                |
    ///|----------------------------|---------------------------------------------------------------|--------------------------------------|
    ///| `account_id_hex`           | The account id where the list of Txos from                    | Account must exist in the database.  |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                      |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                      |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                      |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.             |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                            |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                      |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    fn list_orphaned(
        account_id_hex: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all pending Txos associated with an account or an assigned subaddress
    /// 
    /// # Arguments
    /// 
    ///| Name                       | Purpose                                                       | Notes                                |
    ///|----------------------------|---------------------------------------------------------------|--------------------------------------|
    ///| `account_id_hex`           | The account id where the list of Txos from                    | Account must exist in the database.  |
    ///| `assigned_subaddress_b58`  | The subaddress where the list of Txos from                    |                                      |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                      |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                      |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                      |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.             |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                            |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                      |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    #[allow(clippy::too_many_arguments)]
    fn list_pending(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all unverified Txos associated with an account or an assigned subaddress
    /// 
    /// # Arguments
    /// 
    ///| Name                       | Purpose                                                       | Notes                                |
    ///|----------------------------|---------------------------------------------------------------|--------------------------------------|
    ///| `account_id_hex`           | The account id where the list of Txos from                    | Account must exist in the database.  |
    ///| `assigned_subaddress_b58`  | The subaddress where the list of Txos from                    |                                      |
    ///| `token_id`                 | The id of a supported type of token to filter on              |                                      |
    ///| `min_received_block_index` | The minimum block index to query for received txos, inclusive |                                      |
    ///| `max_received_block_index` | The maximum block index to query for received txos, inclusive |                                      |
    ///| `offset`                   | The pagination offset. Results start at the offset index.     | Optional. Defaults to 0.             |
    ///| `limit`                    | Limit for the number of results.                              | Optional.                            |
    ///| `conn`                     | An reference to the pool connection of wallet database        |                                      |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    #[allow(clippy::too_many_arguments)]
    fn list_unverified(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;


    /// Get all spendable Txos and max spendable value in wallet associated with an account or an assigned subaddress
    /// 
    /// # Arguments
    /// 
    ///| Name                      | Purpose                                                    | Notes                               |
    ///|---------------------------|------------------------------------------------------------|-------------------------------------|
    ///| `account_id_hex`          | The account id at which the list of Txos from              | Account must exist in the database. |
    ///| `max_spendable_value`     | The upper limit for the spendable TxOut value to filter on |                                     |
    ///| `assigned_subaddress_b58` | The subaddress at which the list of Txos from              |                                     |
    ///| `token_id`                | The id of a supported type of token to filter on           |                                     |
    ///| `conn`                    | An reference to the pool connection of wallet database     |                                     |
    ///
    /// 
    /// # Returns
    /// * spendable_txos: Vector of TxoOut
    /// * max_spendable_in_wallet: u128
    fn list_spendable(
        account_id_hex: Option<&str>,
        max_spendable_value: Option<u64>,
        assigned_subaddress_b58: Option<&str>,
        token_id: u64,
        default_token_fee: u64,
        conn: Conn,
    ) -> Result<SpendableTxosResult, WalletDbError>;

    /// Get all created Txos in wallet associated with an account
    /// 
    /// # Arguments
    /// 
    ///| Name             | Purpose                                                | Notes                               |
    ///|------------------|--------------------------------------------------------|-------------------------------------|
    ///| `account_id_hex` | The account id where the Txos from                     | Account must exist in the database. |
    ///| `conn`           | An reference to the pool connection of wallet database |                                     |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    fn list_created(account_id_hex: Option<&str>, conn: Conn) -> Result<Vec<Txo>, WalletDbError>;

    /// Get all secreted Txos in wallet associated with an account
    /// 
    /// # Arguments
    /// 
    ///| Name             | Purpose                                                | Notes                               |
    ///|------------------|--------------------------------------------------------|-------------------------------------|
    ///| `account_id_hex` | The account id where the Txos from                     | Account must exist in the database. |
    ///| `conn`           | An reference to the pool connection of wallet database |                                     |
    /// 
    /// # Returns
    /// * Vector of TxoOut
    fn list_secreted(account_id_hex: Option<&str>, conn: Conn) -> Result<Vec<Txo>, WalletDbError>;

    /// Get the details for a specific Txo.
    ///
    /// # Arguments
    /// 
    ///| Name         | Purpose                                                | Notes |
    ///|--------------|--------------------------------------------------------|-------|
    ///| `txo_id_hex` | The TxOut id from which to retrieve a TxOut            |       |
    ///| `conn`       | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns:
    /// * TxOut
    fn get(txo_id_hex: &str, conn: Conn) -> Result<Txo, WalletDbError>;


    /// Get several Txos by Txo public_keys
    ///
    /// # Arguments
    ///
    ///| Name          | Purpose                                                | Notes |
    ///|---------------|--------------------------------------------------------|-------|
    ///| `public_keys` | The public key where to retrieve Txos from             |       |
    ///| `conn`        | An reference to the pool connection of wallet database |       |
    /// 
    /// # Returns:
    /// * Vector of TxoOut
    fn select_by_public_key(
        public_keys: &[&CompressedRistrettoPublic],
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Select several Txos by their TxoIds
    ///
    /// # Arguments
    /// 
    ///| Name      | Purpose                                                | Notes |
    ///|-----------|--------------------------------------------------------|-------|
    ///| `txo_ids` | The list of TxOut IDs from which to retrieve Txos      |       |
    ///| `conn`    | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns:
    /// * Vector of TxoOut
    fn select_by_id(txo_ids: &[String], conn: Conn) -> Result<Vec<Txo>, WalletDbError>;

    /// Select a set of unspent Txos to reach a given value.
    ///
    /// # Arguments
    ///
    ///| Name                  | Purpose                                                    | Notes                               |
    ///|-----------------------|------------------------------------------------------------|-------------------------------------|
    ///| `account_id_hex`      | The account id where the Txos from                         | Account must exist in the database. |
    ///| `target_value`        | The value used to filter spendable Txos on its value       |                                     |
    ///| `max_spendable_value` | The upper limit for the spendable TxOut value to filter on |                                     |
    ///| `token_id`            | The id of a supported type of token to filter on           |                                     |
    ///| `default_token_fee`   | The default transaction fee in Mob network                 |                                     |
    ///| `conn`                | An reference to the pool connection of wallet database     |                                     |
    ///
    /// # Returns:
    /// * Vector of TxoOut
    fn select_spendable_txos_for_value(
        account_id_hex: &str,
        target_value: u128,
        max_spendable_value: Option<u64>,
        token_id: u64,
        default_token_fee: u64,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Validate a confirmation number for a TxOut
    ///
    /// # Arguments
    /// 
    ///| Name           | Purpose                                                        | Notes                               |
    ///|----------------|----------------------------------------------------------------|-------------------------------------|
    ///| `account_id`   | The account id used to retrieve the account key                | Account must exist in the database. |
    ///| `txo_id_hex`   | The TxOut id used to retrieve the TxOut public_key             |                                     |
    ///| `confirmation` | The confirmation to valid the TxOut public_key and account key |                                     |
    ///| `conn`         | An reference to the pool connection of wallet database         |                                     |
    ///
    /// # Returns:
    /// * Bool - true if verified
    fn validate_confirmation(
        account_id: &AccountID,
        txo_id_hex: &str,
        confirmation: &TxOutConfirmationNumber,
        conn: Conn,
    ) -> Result<bool, WalletDbError>;

    /// Remove account id from all Txos at which the account associates to
    /// 
    /// # Arguments
    /// 
    ///| Name         | Purpose                                                                               | Notes |
    ///|--------------|---------------------------------------------------------------------------------------|-------|
    ///| `account_id` | The account id needs to be removed from all Txos at which the account associates to   |       |
    ///| `conn`       | An reference to the pool connection of wallet database                                |       |
    ///
    /// # Returns
    /// * unit
    fn scrub_account(account_id_hex: &str, conn: Conn) -> Result<(), WalletDbError>;

    /// Delete txos which are not referenced by any account or transaction.
    /// 
    /// # Arguments
    /// 
    ///| Name   | Purpose                                                | Notes |
    ///|--------|--------------------------------------------------------|-------|
    ///| `conn` | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns
    /// * unit
    fn delete_unreferenced(conn: Conn) -> Result<(), WalletDbError>;

    /// Get status for current TxOut
    /// 
    /// # Arguments
    /// 
    ///| Name   | Purpose                                                | Notes |
    ///|--------|--------------------------------------------------------|-------|
    ///| `conn` | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns
    /// * TxoStatus 
    fn status(&self, conn: Conn) -> Result<TxoStatus, WalletDbError>;

    /// Get memo for current TxOut
    fn memo(&self, conn: Conn) -> Result<TxoMemo, WalletDbError>;

    /// Get the membership proof from ledger DB for current TxOut
    /// 
    /// # Arguments
    /// 
    ///| Name        | Purpose                                                   | Notes                                     |
    ///|-------------|-----------------------------------------------------------|-------------------------------------------|
    ///| `ledger_db` | A reference to the instance of the whole ledger database. | This object has a connection to ledger DB |
    ///
    /// # Returns
    /// * TxOutMembershipProof 
    fn membership_proof(&self, ledger_db: &LedgerDB) -> Result<TxOutMembershipProof, WalletDbError>;

    /// Update the key image and spent block index for a txo by its public key.
    /// 
    /// # Arguments
    /// 
    ///| Name        | Purpose                                                   | Notes                                     |
    ///|-------------|-----------------------------------------------------------|-------------------------------------------|
    ///| `public_key` | The compressed ristretto public to use to search for the txo. | |
    ///| `key_image` | The Key Image to add to the txo. | |
    ///| `spent_block_index` | The spent block index to update for the txo. |  |
    ///| `conn` | A reference to the connection of wallet database |  |
    ///
    /// # Returns
    /// * unit 
    fn update_key_image_by_pubkey(
        public_key: &CompressedRistrettoPublic,
        key_image: &KeyImage,
        spent_block_index: Option<u64>,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Get the public address of the recipient of this txo, if available. 
    /// If we created the txo, it would be the address at which we received it. Otherwise,
    /// it will require a lookup of who we sent it to in the transaction_txo_outputs table
    fn recipient_public_address(&self, conn: Conn) -> Result<Option<PublicAddress>, WalletDbError>;
}

impl TxoModel for Txo {
    fn create_received(
        txo: TxOut,
        subaddress_index: Option<u64>,
        key_image: Option<KeyImage>,
        amount: Amount,
        received_block_index: u64,
        account_id_hex: &str,
        conn: Conn,
    ) -> Result<String, WalletDbError> {
        // Verify that the account exists.
        let account = Account::get(&AccountID(account_id_hex.to_string()), conn)?;
        let shared_secret =
            account.get_shared_secret(&RistrettoPublic::try_from(&txo.public_key)?)?;

        let txo_id = TxoID::from(&txo);
        let shared_secret_vec = shared_secret.encode_to_vec();

        // Decrypt the memo and determine its type bytes.
        let memo_payload = match txo.e_memo {
            Some(e_memo) => e_memo.decrypt(&shared_secret),
            None => UnusedMemo.into(),
        };
        let memo_type = Some(two_bytes_to_i32(*memo_payload.get_memo_type()));

        // Ensure that the TXO is added to the database.
        match Txo::get(&txo_id.to_string(), conn) {
            // If we already have this TXO for this account (e.g. from minting in a previous
            // transaction), we need to update it
            Ok(found_txo) => {
                found_txo.update_as_received(
                    subaddress_index,
                    key_image,
                    received_block_index,
                    account_id_hex,
                    amount,
                    &mc_util_serial::encode(&txo.target_key),
                    &mc_util_serial::encode(&txo.public_key),
                    &mc_util_serial::encode(&txo.e_fog_hint),
                    Some(&shared_secret_vec),
                    memo_type,
                    conn,
                )?;
            }

            // If we don't already have this TXO, create a new entry
            Err(WalletDbError::TxoNotFound(_)) => {
                let key_image_bytes = key_image.map(|k| mc_util_serial::encode(&k));
                let new_txo = NewTxo {
                    id: &txo_id.to_string(),
                    value: amount.value as i64,
                    token_id: *amount.token_id as i64,
                    target_key: &mc_util_serial::encode(&txo.target_key),
                    public_key: &mc_util_serial::encode(&txo.public_key),
                    e_fog_hint: &mc_util_serial::encode(&txo.e_fog_hint),
                    subaddress_index: subaddress_index.map(|i| i as i64),
                    key_image: key_image_bytes.as_deref(),
                    received_block_index: Some(received_block_index as i64),
                    spent_block_index: None,
                    confirmation: None,
                    account_id: Some(account_id_hex.to_string()),
                    shared_secret: Some(&shared_secret_vec),
                    memo_type,
                };

                diesel::insert_into(crate::db::schema::txos::table)
                    .values(&new_txo)
                    .execute(conn)?;
            }
            Err(e) => {
                return Err(e);
            }
        };

        // Interpret the memo payload and save it to the correct database table.
        // Check that there is no existing memo before creating a new one.
        use crate::db::schema::authenticated_sender_memos;
        if let Ok(MemoType::AuthenticatedSender(memo)) = MemoType::try_from(&memo_payload) {
            match authenticated_sender_memos::table
                .filter(authenticated_sender_memos::txo_id.eq(txo_id.to_string()))
                .get_result::<AuthenticatedSenderMemoModel>(conn)
            {
                Ok(_) => {}
                Err(diesel::result::Error::NotFound) => {
                    let new_memo = NewAuthenticatedSenderMemo {
                        txo_id: &txo_id.to_string(),
                        sender_address_hash: &memo.sender_address_hash().to_string(),
                        payment_request_id: None,
                        payment_intent_id: None,
                    };

                    diesel::insert_into(authenticated_sender_memos::table)
                        .values(&new_memo)
                        .execute(conn)?;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(txo_id.to_string())
    }

    fn create_new_output(
        output_txo: &OutputTxo,
        is_change: bool,
        transaction_id: &TransactionId,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let txo_id = TxoID::from(&output_txo.tx_out);
        let encoded_confirmation = mc_util_serial::encode(&output_txo.confirmation_number);
        let new_txo = NewTxo {
            id: &txo_id.to_string(),
            account_id: None,
            value: output_txo.amount.value as i64,
            token_id: *output_txo.amount.token_id as i64,
            target_key: &mc_util_serial::encode(&output_txo.tx_out.target_key),
            public_key: &mc_util_serial::encode(&output_txo.tx_out.public_key),
            e_fog_hint: &mc_util_serial::encode(&output_txo.tx_out.e_fog_hint),
            subaddress_index: None,
            key_image: None,
            received_block_index: None,
            spent_block_index: None,
            confirmation: Some(&encoded_confirmation),
            shared_secret: None, // no account id so we don't
            memo_type: None,
        };

        diesel::insert_into(txos::table)
            .values(&new_txo)
            .execute(conn)?;

        let recipient_public_address_b58 =
            &b58_encode_public_address(&output_txo.recipient_public_address)?;

        let new_transaction_output_txo = NewTransactionOutputTxo {
            transaction_log_id: &transaction_id.to_string(),
            txo_id: &txo_id.to_string(),
            recipient_public_address_b58,
            is_change,
        };

        diesel::insert_into(crate::db::schema::transaction_output_txos::table)
            .values(&new_transaction_output_txo)
            .execute(conn)?;

        Ok(())
    }

    fn update_as_received(
        &self,
        subaddress_index: Option<u64>,
        received_key_image: Option<KeyImage>,
        block_index: u64,
        account_id_hex: &str,
        amount: Amount,
        target_key: &[u8],
        public_key: &[u8],
        e_fog_hint: &[u8],
        shared_secret: Option<&[u8]>,
        memo_type: Option<i32>,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let encoded_key_image = received_key_image.map(|k| mc_util_serial::encode(&k));

        diesel::update(self)
            .set((
                txos::subaddress_index.eq(subaddress_index.map(|i| i as i64)),
                txos::key_image.eq(encoded_key_image),
                txos::received_block_index.eq(Some(block_index as i64)),
                txos::account_id.eq(Some(account_id_hex)),
                txos::value.eq(amount.value as i64),
                txos::token_id.eq(*amount.token_id as i64),
                txos::target_key.eq(target_key),
                txos::public_key.eq(public_key),
                txos::e_fog_hint.eq(e_fog_hint),
                txos::shared_secret.eq(shared_secret),
                txos::memo_type.eq(memo_type),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn update_spent_block_index(
        txo_id_hex: &str,
        spent_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        diesel::update(txos::table.filter(txos::id.eq(txo_id_hex)))
            .set((txos::spent_block_index.eq(Some(spent_block_index as i64)),))
            .execute(conn)?;
        Ok(())
    }

    fn update_key_image(
        txo_id_hex: &str,
        key_image: &KeyImage,
        spent_block_index: Option<u64>,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let encoded_key_image = mc_util_serial::encode(key_image);

        diesel::update(txos::table.filter(txos::id.eq(txo_id_hex)))
            .set((
                txos::key_image.eq(Some(encoded_key_image)),
                txos::spent_block_index.eq(spent_block_index.map(|i| i as i64)),
            ))
            .execute(conn)?;

        Ok(())
    }

    fn update_is_synced_to_t3(&self, is_synced: bool, conn: Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        diesel::update(self)
            .set(txos::is_synced_to_t3.eq(is_synced))
            .execute(conn)?;

        Ok(())
    }

    fn get_txos_that_need_to_be_synced_to_t3(
        limit: Option<usize>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let memo_types_predicate = txos::memo_type
            .eq(two_bytes_to_i32(AuthenticatedSenderMemo::MEMO_TYPE_BYTES))
            .or(txos::memo_type.eq(two_bytes_to_i32(
                AuthenticatedSenderWithPaymentIntentIdMemo::MEMO_TYPE_BYTES,
            )))
            .or(txos::memo_type.eq(two_bytes_to_i32(
                AuthenticatedSenderWithPaymentRequestIdMemo::MEMO_TYPE_BYTES,
            )));

        let mut query = txos::table.into_boxed();

        query = query
            .filter(txos::is_synced_to_t3.eq(false))
            .filter(memo_types_predicate);

        if let Some(l) = limit {
            query = query.limit(l as i64);
        }

        Ok(query.load(conn)?)
    }

    fn list(
        status: Option<TxoStatus>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        if let Some(status) = status {
            match status {
                TxoStatus::Unverified => {
                    return Txo::list_unverified(
                        None,
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Unspent => {
                    return Txo::list_unspent(
                        None,
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Pending => {
                    return Txo::list_pending(
                        None,
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Spent => {
                    return Txo::list_spent(
                        None,
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Orphaned => {
                    return Txo::list_orphaned(
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Created => {
                    return Txo::list_created(None, conn);
                }
                TxoStatus::Secreted => {
                    return Txo::list_secreted(None, conn);
                }
            }
        }

        let mut query = txos::table.into_boxed();

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query.order(txos::received_block_index.desc()).load(conn)?)
    }

    fn list_for_account(
        account_id_hex: &str,
        status: Option<TxoStatus>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        if let Some(status) = status {
            match status {
                TxoStatus::Unverified => {
                    return Txo::list_unverified(
                        Some(account_id_hex),
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Unspent => {
                    return Txo::list_unspent(
                        Some(account_id_hex),
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Pending => {
                    return Txo::list_pending(
                        Some(account_id_hex),
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Spent => {
                    return Txo::list_spent(
                        Some(account_id_hex),
                        None,
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Orphaned => {
                    return Txo::list_orphaned(
                        Some(account_id_hex),
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Created => {
                    return Txo::list_created(Some(account_id_hex), conn);
                }
                TxoStatus::Secreted => {
                    return Txo::list_secreted(Some(account_id_hex), conn);
                }
            }
        }

        let mut query = txos::table.into_boxed();

        query = query.filter(txos::account_id.eq(account_id_hex));

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query.order(txos::received_block_index.desc()).load(conn)?)
    }

    fn list_for_address(
        assigned_subaddress_b58: &str,
        status: Option<TxoStatus>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        if let Some(status) = status {
            match status {
                TxoStatus::Unverified => {
                    return Txo::list_unverified(
                        None,
                        Some(assigned_subaddress_b58),
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Unspent => {
                    return Txo::list_unspent(
                        None,
                        Some(assigned_subaddress_b58),
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Pending => {
                    return Txo::list_pending(
                        None,
                        Some(assigned_subaddress_b58),
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Spent => {
                    return Txo::list_spent(
                        None,
                        Some(assigned_subaddress_b58),
                        token_id,
                        min_received_block_index,
                        max_received_block_index,
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Orphaned => {
                    return Ok(vec![]);
                }
                TxoStatus::Created => {
                    return Ok(vec![]);
                }
                TxoStatus::Secreted => {
                    return Ok(vec![]);
                }
            }
        }

        let subaddress = AssignedSubaddress::get(assigned_subaddress_b58, conn)?;

        let mut query = txos::table.into_boxed();

        query = query
            .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
            .filter(txos::account_id.eq(subaddress.account_id));

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query.order(txos::received_block_index.desc()).load(conn)?)
    }

    fn list_unspent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs, txos};

        /*
            SELECT * FROM txos
            LEFT JOIN transaction_txos
            ON txos.id = transaction_txos.txo_id
            LEFT JOIN transaction_logs
            ON transaction_txos.transaction_log_id = transaction_logs.id
            WHERE (transaction_logs.id IS NULL
            OR ((transaction_txos.used_as = "input" AND (transaction_logs.failed = 1 OR transaction_logs.submitted_block_index = null))
            OR (transaction_txos.used_as != "input" AND transaction_logs.failed = 0)))
            AND txos.key_image IS NOT NULL
            AND txos.spent_block_index IS NULL
        */

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_input_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_input_txos::transaction_log_id)),
            );

        query = query.filter(
            transaction_logs::id
                .is_null()
                .or(transaction_logs::failed.eq(true))
                .or(transaction_logs::id
                    .is_not_null()
                    .and(transaction_logs::submitted_block_index.is_null())),
        );

        query = query.filter(txos::received_block_index.is_not_null());
        query = query.filter(txos::key_image.is_not_null());
        query = query.filter(txos::spent_block_index.is_null());

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query
                .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
                .filter(txos::account_id.eq(subaddress.account_id));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query
            .select(txos::all_columns)
            .distinct()
            .order(txos::received_block_index.desc())
            .load(conn)?)
    }

    fn list_unverified(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs, txos};

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_input_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_input_txos::transaction_log_id)),
            );

        query = query.filter(
            transaction_logs::id
                .is_null()
                .or(transaction_logs::failed.eq(true))
                .or(transaction_logs::id
                    .is_not_null()
                    .and(transaction_logs::submitted_block_index.is_null())),
        );

        query = query
            .filter(txos::received_block_index.is_not_null())
            .filter(txos::subaddress_index.is_not_null())
            .filter(txos::key_image.is_null());

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query
                .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
                .filter(txos::account_id.eq(subaddress.account_id));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query
            .distinct()
            .order(txos::received_block_index.desc())
            .load(conn)?)
    }

    fn list_created(account_id_hex: Option<&str>, conn: Conn) -> Result<Vec<Txo>, WalletDbError> {
        /*
            SELECT
                *
            FROM txos
            LEFT JOIN transaction_output_txos
            ON txos.id = transaction_output_txos.txo_id
            LEFT JOIN transaction_logs
            ON transaction_output_txos.transaction_log_id = transaction_logs.id
            WHERE
             txos.key_image IS NULL
            AND txos.spent_block_index IS NULL
            AND txos.subaddress_index IS NULL
            AND txos.received_block_index IS NULL
            AND (
              (transaction_logs.failed = 1) OR
              (transaction_logs.failed = 0 AND transaction_logs.finalized_block_index isNULL AND  submitted_block_index ISNULL)
              )
        */
        use crate::db::schema::{transaction_logs, transaction_output_txos, txos};

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_output_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_output_txos::transaction_log_id)),
            );

        query = query
            .filter(txos::key_image.is_null())
            .filter(txos::spent_block_index.is_null())
            .filter(txos::subaddress_index.is_null())
            .filter(txos::received_block_index.is_null())
            .filter(
                transaction_logs::failed
                    .eq(true)
                    .or(transaction_logs::failed
                        .eq(false)
                        .and(transaction_logs::finalized_block_index.is_null())
                        .and(transaction_logs::submitted_block_index.is_null())),
            );

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(transaction_logs::account_id.eq(account_id_hex))
        }

        Ok(query.select(txos::all_columns).distinct().load(conn)?)
    }

    fn list_secreted(account_id_hex: Option<&str>, conn: Conn) -> Result<Vec<Txo>, WalletDbError> {
        /*
            SELECT *
            FROM
                txos
                LEFT JOIN transaction_output_txos on txos.id = transaction_output_txos.txo_id
                 LEFT JOIN transaction_logs ON transaction_output_txos.transaction_log_id = transaction_logs.id
            WHERE
                transaction_output_txos.is_change = 0   AND
                transaction_logs.failed = 0             AND
                transaction_logs.submitted_block_index is not NULL AND
                transaction_logs.finalized_block_index is not NULL AND
                transaction_logs.account_id = <INPUT PARAMETER ACCOUNT_ID>
                transaction_log.account_id != txos.account_id
        */

        use crate::db::schema::{transaction_logs, transaction_output_txos, txos};

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_output_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_output_txos::transaction_log_id)),
            );

        query = query
            .filter(transaction_output_txos::is_change.eq(false))
            .filter(transaction_logs::failed.eq(false))
            .filter(transaction_logs::finalized_block_index.is_not_null())
            .filter(transaction_logs::submitted_block_index.is_not_null())
            .filter(not(transaction_logs::account_id
                .nullable()
                .eq(txos::account_id)));

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(transaction_logs::account_id.eq(account_id_hex))
        }

        Ok(query.select(txos::all_columns).distinct().load(conn)?)
    }

    fn list_unspent_or_pending_key_images(
        account_id_hex: &str,
        token_id: Option<u64>,
        conn: Conn,
    ) -> Result<HashMap<KeyImage, String>, WalletDbError> {
        use crate::db::schema::txos;

        let mut query = txos::table.into_boxed();

        query = query
            .filter(txos::key_image.is_not_null())
            .filter(txos::account_id.eq(account_id_hex))
            .filter(txos::subaddress_index.is_not_null())
            .filter(txos::spent_block_index.is_null());

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        let results: Vec<(Option<Vec<u8>>, String)> = query
            .select((txos::key_image, txos::id))
            .order(txos::received_block_index.desc())
            .load(conn)?;

        Ok(results
            .into_iter()
            .filter_map(|(key_image, txo_id_hex)| match key_image {
                Some(key_image_encoded) => {
                    let key_image = mc_util_serial::decode(key_image_encoded.as_slice()).ok()?;
                    Some((key_image, txo_id_hex))
                }
                None => None,
            })
            .collect())
    }

    fn list_spent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let mut query = txos::table.into_boxed();

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        query = query.filter(txos::spent_block_index.is_not_null());

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query
                .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
                .filter(txos::account_id.eq(subaddress.account_id));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query.order(txos::received_block_index.desc()).load(conn)?)
    }

    fn list_orphaned(
        account_id_hex: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let mut query = txos::table.into_boxed();

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        query = query
            .filter(txos::subaddress_index.is_null())
            .filter(txos::key_image.is_null());

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query.order(txos::received_block_index.desc()).load(conn)?)
    }

    fn list_pending(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        min_received_block_index: Option<u64>,
        max_received_block_index: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs, txos};

        let mut query = txos::table
            .into_boxed()
            .inner_join(transaction_input_txos::table)
            .inner_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_input_txos::transaction_log_id)),
            );

        query = query
            .filter(transaction_logs::failed.eq(false))
            .filter(transaction_logs::finalized_block_index.is_null())
            .filter(transaction_logs::submitted_block_index.is_not_null());

        query = query
            .filter(txos::subaddress_index.is_not_null())
            .filter(txos::spent_block_index.is_null());

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query
                .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
                .filter(txos::account_id.eq(subaddress.account_id));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(min_received_block_index) = min_received_block_index {
            query = query.filter(txos::received_block_index.ge(min_received_block_index as i64));
        }

        if let Some(max_received_block_index) = max_received_block_index {
            query = query.filter(txos::received_block_index.le(max_received_block_index as i64));
        }

        Ok(query
            .select(txos::all_columns)
            .distinct()
            .order(txos::received_block_index.desc())
            .load(conn)?)
    }

    fn get(txo_id_hex: &str, conn: Conn) -> Result<Txo, WalletDbError> {
        use crate::db::schema::txos;

        let txo = match txos::table
            .filter(txos::id.eq(txo_id_hex))
            .get_result::<Txo>(conn)
        {
            Ok(t) => t,
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::TxoNotFound(txo_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(txo)
    }

    fn select_by_public_key(
        public_keys: &[&CompressedRistrettoPublic],
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let public_key_blobs: Vec<Vec<u8>> = public_keys
            .iter()
            .map(|p| mc_util_serial::encode(*p))
            .collect();
        let selected = txos::table
            .filter(txos::public_key.eq_any(public_key_blobs))
            .load(conn)?;
        Ok(selected)
    }

    fn select_by_id(txo_ids: &[String], conn: Conn) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let txos: Vec<Txo> = txos::table.filter(txos::id.eq_any(txo_ids)).load(conn)?;

        Ok(txos)
    }

    fn list_spendable(
        account_id_hex: Option<&str>,
        max_spendable_value: Option<u64>,
        assigned_subaddress_b58: Option<&str>,
        token_id: u64,
        default_token_fee: u64,
        conn: Conn,
    ) -> Result<SpendableTxosResult, WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs, txos};

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_input_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_input_txos::transaction_log_id)),
            );

        query = query
            .filter(transaction_logs::id.is_null())
            .or_filter(transaction_logs::failed.eq(true))
            .or_filter(
                transaction_logs::id
                    .is_not_null()
                    .and(transaction_logs::submitted_block_index.is_null()),
            );

        query = query
            .filter(txos::received_block_index.is_not_null())
            .filter(txos::spent_block_index.is_null())
            .filter(txos::subaddress_index.is_not_null())
            .filter(txos::token_id.eq(token_id as i64));

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query
                .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
                .filter(txos::account_id.eq(subaddress.account_id));
        }

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        let mut spendable_txos = query
            .select(txos::all_columns)
            .distinct()
            .load(conn)?
            .drain(..)
            .filter(|txo: &Txo| txo.value as u64 <= max_spendable_value.unwrap_or(u64::MAX))
            .collect::<Vec<Txo>>();

        spendable_txos.sort_by(|a: &Txo, b: &Txo| (b.value as u64).cmp(&(a.value as u64)));

        // The maximum spendable is limited by the maximal number of inputs we can use.
        // Since the txos are sorted by decreasing value, this is the maximum
        // value we can possibly spend in one transaction.
        // Note, u128::Max = 340_282_366_920_938_463_463_374_607_431_768_211_455, which
        // is far beyond the total number of pMOB in the MobileCoin system
        // (250_000_000_000_000_000_000)
        let mut max_spendable_in_wallet: u128 = spendable_txos
            .iter()
            .take(MAX_INPUTS as usize)
            .map(|utxo: &Txo| (utxo.value as u64) as u128)
            .sum();

        if max_spendable_in_wallet > default_token_fee as u128 {
            max_spendable_in_wallet -= default_token_fee as u128;
        } else {
            max_spendable_in_wallet = 0;
        }

        Ok(SpendableTxosResult {
            spendable_txos,
            max_spendable_in_wallet,
        })
    }

    fn select_spendable_txos_for_value(
        account_id_hex: &str,
        target_value: u128,
        max_spendable_value: Option<u64>,
        token_id: u64,
        default_token_fee: u64,
        conn: Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        let SpendableTxosResult {
            mut spendable_txos,
            max_spendable_in_wallet,
        } = Txo::list_spendable(
            Some(account_id_hex),
            max_spendable_value,
            None,
            token_id,
            default_token_fee,
            conn,
        )?;

        if spendable_txos.is_empty() {
            return Err(WalletDbError::NoSpendableTxos(token_id.to_string()));
        }

        // If we're trying to spend more than we have in the wallet, we may need to
        // defrag
        if target_value > max_spendable_in_wallet + default_token_fee as u128 {
            // See if we merged the UTXOs we would be able to spend this amount.
            let total_unspent_value_in_wallet: u128 = spendable_txos
                .iter()
                .map(|utxo| (utxo.value as u64) as u128)
                .sum();

            if total_unspent_value_in_wallet >= target_value + default_token_fee as u128 {
                return Err(WalletDbError::InsufficientFundsFragmentedTxos);
            } else {
                return Err(WalletDbError::InsufficientFundsUnderMaxSpendable(format!(
                    "Max spendable value in wallet: {max_spendable_in_wallet:?}, but target value: {target_value:?}"
                )));
            }
        }

        // Select the actual Txos to spend. We want to opportunistically fill up the
        // input slots with dust, from any subaddress, so we take from the back
        // of the Txo vec. This is a knapsack problem, and the selection could
        // be improved. For now, we simply move the window of MAX_INPUTS up from
        // the back of the sorted vector until we have a window with
        // a large enough sum.
        let mut selected_utxos: Vec<Txo> = Vec::new();
        let mut total: u128 = 0;
        loop {
            if total >= target_value {
                global_log::debug!("total is greater than target value");
                break;
            }

            // Grab the next (smallest) utxo, in order to opportunistically sweep up dust
            let next_utxo = spendable_txos.pop().ok_or_else(|| {
                WalletDbError::InsufficientFunds(format!(
                    "Not enough Txos to sum to target value: {target_value:?}"
                ))
            })?;
            selected_utxos.push(next_utxo.clone());
            total += (next_utxo.value as u64) as u128;
            global_log::debug!(
                "select_spendable_txos_for_value: selected utxo: {:?}, total: {:?}, target: {:?}",
                next_utxo.value as u64,
                total,
                target_value,
            );

            // Cap at maximum allowed inputs.
            if selected_utxos.len() > MAX_INPUTS as usize {
                // Remove the lowest utxo.
                let removed = selected_utxos.remove(0);
                total -= (removed.value as u64) as u128;
            }
        }

        if selected_utxos.is_empty() || selected_utxos.len() > MAX_INPUTS as usize {
            return Err(WalletDbError::InsufficientFunds(
                "Logic error. Could not select Txos despite having sufficient funds".to_string(),
            ));
        }

        Ok(selected_utxos)
    }

    fn validate_confirmation(
        account_id: &AccountID,
        txo_id_hex: &str,
        confirmation: &TxOutConfirmationNumber,
        conn: Conn,
    ) -> Result<bool, WalletDbError> {
        let txo = Txo::get(txo_id_hex, conn)?;
        let public_key: RistrettoPublic = mc_util_serial::decode(&txo.public_key)?;
        let account = Account::get(account_id, conn)?;
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        Ok(confirmation.validate(&public_key, account_key.view_private_key()))
    }

    fn scrub_account(account_id_hex: &str, conn: Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let txos_received_by_account = txos::table.filter(txos::account_id.eq(account_id_hex));

        diesel::update(txos_received_by_account)
            .set(txos::account_id.eq::<Option<String>>(None))
            .execute(conn)?;

        Ok(())
    }

    fn delete_unreferenced(conn: Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_output_txos, txos};

        /*
           SELECT * FROM txos
           WHERE NOT EXISTS (SELECT * FROM transaction_input_txos WHERE transaction_input_txos.txo_id = txos.id)
           AND NOT EXISTS (SELECT * FROM transaction_output_txos WHERE transaction_output_txos.txo_id = txos.id)
           AND txos.account_id_hex IS NULL
        */

        let unreferenced_txos = txos::table
            .filter(not(exists(
                transaction_input_txos::table.filter(transaction_input_txos::txo_id.eq(txos::id)),
            )))
            .filter(not(exists(
                transaction_output_txos::table.filter(transaction_output_txos::txo_id.eq(txos::id)),
            )))
            .filter(txos::account_id.is_null());

        diesel::delete(unreferenced_txos).execute(conn)?;

        Ok(())
    }

    fn status(&self, conn: Conn) -> Result<TxoStatus, WalletDbError> {
        use crate::db::schema::{
            transaction_input_txos, transaction_logs, transaction_output_txos, txos,
        };

        if self.spent_block_index.is_some() {
            return Ok(TxoStatus::Spent);
        }

        let num_pending_logs: i64 = transaction_logs::table
            .inner_join(transaction_input_txos::table)
            .inner_join(transaction_output_txos::table)
            .filter(
                transaction_input_txos::txo_id
                    .eq(&self.id)
                    .or(transaction_output_txos::txo_id.eq(&self.id)),
            )
            .filter(transaction_logs::finalized_block_index.is_null())
            .filter(transaction_logs::submitted_block_index.is_not_null())
            .filter(transaction_logs::failed.eq(false))
            .select(count(transaction_logs::id))
            .first(conn)?;

        if num_pending_logs > 0 {
            return Ok(TxoStatus::Pending);
        }

        let num_secreted_logs: i64 = transaction_logs::table
            .left_join(transaction_output_txos::table)
            .left_join(txos::table.on(transaction_output_txos::txo_id.eq(txos::id)))
            .filter(transaction_output_txos::txo_id.eq(&self.id))
            .filter(transaction_output_txos::is_change.eq(false))
            .filter(transaction_logs::failed.eq(false))
            .filter(transaction_logs::finalized_block_index.is_not_null())
            .filter(transaction_logs::submitted_block_index.is_not_null())
            .filter(not(transaction_logs::account_id
                .nullable()
                .eq(txos::account_id)))
            .select(count(transaction_logs::id))
            .first(conn)?;

        if num_secreted_logs > 0 {
            return Ok(TxoStatus::Secreted);
        }

        let num_created_logs: i64 = transaction_logs::table
            .inner_join(transaction_output_txos::table)
            .filter(transaction_output_txos::txo_id.eq(&self.id))
            .filter(
                transaction_logs::failed
                    .eq(true)
                    .or(transaction_logs::failed
                        .eq(false)
                        .and(transaction_logs::finalized_block_index.is_null())
                        .and(transaction_logs::submitted_block_index.is_null())),
            )
            .select(count(transaction_logs::id))
            .first(conn)?;

        if num_created_logs > 0 {
            return Ok(TxoStatus::Created);
        }

        if self.subaddress_index.is_some() && self.key_image.is_some() {
            Ok(TxoStatus::Unspent)
        } else if self.subaddress_index.is_some() {
            Ok(TxoStatus::Unverified)
        } else {
            Ok(TxoStatus::Orphaned)
        }
    }

    fn memo(&self, conn: Conn) -> Result<TxoMemo, WalletDbError> {
        use crate::db::schema::authenticated_sender_memos;
        Ok(
            match self.memo_type {
                None => TxoMemo::Unused,
                Some(mtype) => {
                    match i32_to_two_bytes(mtype) {
                        <AuthenticatedSenderMemo as RegisteredMemoType>::MEMO_TYPE_BYTES |
                        <AuthenticatedSenderWithPaymentIntentIdMemo as RegisteredMemoType>::MEMO_TYPE_BYTES |
                        <AuthenticatedSenderWithPaymentRequestIdMemo as RegisteredMemoType>::MEMO_TYPE_BYTES
                            => {
                                let db_memo = authenticated_sender_memos::table.filter(
                                    authenticated_sender_memos::txo_id.eq(&self.id),
                                    ).first::<AuthenticatedSenderMemoModel>(conn)?;
                                TxoMemo::AuthenticatedSender(db_memo)
                            },
                        _ => TxoMemo::Unused,
                    }
                }
            }
        )
    }

    fn membership_proof(
        &self,
        ledger_db: &LedgerDB,
    ) -> Result<TxOutMembershipProof, WalletDbError> {
        let index = ledger_db.get_tx_out_index_by_public_key(&self.public_key()?)?;
        let membership_proof = ledger_db
            .get_tx_out_proof_of_memberships(&[index])?
            .first()
            .ok_or_else(|| WalletDbError::MissingTxoMembershipProof(self.id.clone()))?
            .clone();

        Ok(membership_proof)
    }

    fn update_key_image_by_pubkey(
        public_key: &CompressedRistrettoPublic,
        key_image: &KeyImage,
        spent_block_index: Option<u64>,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let pubkey = &mc_util_serial::encode(public_key);

        let txo = txos::table
            .filter(txos::public_key.eq(pubkey))
            .first::<Txo>(conn)?;

        let txo = Txo::get(&txo.id, conn)?;
        Self::update_key_image(&txo.id, key_image, spent_block_index, conn)?;

        Ok(())
    }

    fn recipient_public_address(&self, conn: Conn) -> Result<Option<PublicAddress>, WalletDbError> {
        use crate::db::schema::transaction_output_txos;

        match (&self.account_id, self.subaddress_index) {
            // if an account in the database owns the TXO and we have an available
            // subaddress index (not orphaned) we can lookup the public address that
            // it was sent to
            (Some(account_id), Some(subaddress_index)) => {
                return Ok(Some(
                    AssignedSubaddress::get_for_account_by_index(
                        account_id,
                        subaddress_index,
                        conn,
                    )?
                    .public_address()?,
                ));
            }
            // If we do not own it, we can look up its transaction_output_txo which will give us the
            // recipient public b58
            (None, None) => {
                let transaction_output_txo: TransactionOutputTxo = transaction_output_txos::table
                    .filter(transaction_output_txos::txo_id.eq(&self.id))
                    .first(conn)?;

                return Ok(Some(transaction_output_txo.recipient_public_address()?));
            }
            // The rest are either orphaned txos or invalid states we should never hit, which we
            // both want to ignore
            _ => return Ok(None),
        }
    }
}

fn i32_to_two_bytes(value: i32) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

fn two_bytes_to_i32(bytes: [u8; 2]) -> i32 {
    ((bytes[0] as i32) << 8) | (bytes[1] as i32)
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{
        AccountKey, PublicAddress, RootIdentity, ShortAddressHash, CHANGE_SUBADDRESS_INDEX,
    };
    use mc_common::{
        logger::{async_test_with_logger, log, test_with_logger, Logger},
        HashSet,
    };
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::Ledger;
    use mc_rand::RngCore;
    use mc_transaction_core::{tokens::Mob, Amount, Token, TokenId};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{iter::FromIterator, ops::DerefMut, time::Duration};

    use crate::{
        db::{
            account::{AccountID, AccountModel},
            models::{Account, TransactionLog},
            transaction_log::TransactionLogModel,
        },
        service::{transaction::TransactionMemo, transaction_builder::WalletTransactionBuilder},
        test_utils::{
            add_block_with_tx, add_block_with_tx_outs, create_test_minted_and_change_txos,
            create_test_received_txo, create_test_txo_for_recipient,
            create_test_txo_for_recipient_with_memo, create_test_unsigned_txproposal_and_log,
            get_resolver_factory, get_test_ledger, manually_sync_account,
            random_account_with_seed_values, WalletDbTestContext, MOB,
        },
        WalletDb,
    };

    use super::*;

    // The narrative for this test is that Alice receives a Txo, then sends a
    // transaction to Bob. We verify expected qualities of the Txos involved at
    // each step of the lifecycle.
    // Note: This is not a replacement for a service-level test, but instead tests
    // basic assumptions after common DB operations with the Txo.
    #[async_test_with_logger]
    async fn test_received_txo_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let mut ledger_db = get_test_ledger(5, &[], 12, &mut rng);

        let root_id = RootIdentity::from_random(&mut rng);
        let alice_account_key = AccountKey::from(&root_id);
        let (alice_account_id, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(1),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Create TXO for Alice
        let (for_alice_txo, for_alice_key_image) = create_test_txo_for_recipient(
            &alice_account_key,
            0,
            Amount::new(1000 * MOB, Mob::ID),
            &mut rng,
        );

        // Let's add this txo to the ledger
        add_block_with_tx_outs(
            &mut ledger_db,
            &[for_alice_txo.clone()],
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        let alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, &logger);

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            conn,
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        // Verify that the Txo is what we expect
        let shared_secret = alice_account
            .get_shared_secret(&RistrettoPublic::try_from(&for_alice_txo.public_key).unwrap())
            .unwrap();
        let expected_txo = Txo {
            id: TxoID::from(&for_alice_txo).to_string(),
            value: 1000 * MOB as i64,
            token_id: 0,
            target_key: mc_util_serial::encode(&for_alice_txo.target_key),
            public_key: mc_util_serial::encode(&for_alice_txo.public_key),
            e_fog_hint: mc_util_serial::encode(&for_alice_txo.e_fog_hint),
            subaddress_index: Some(0),
            key_image: Some(mc_util_serial::encode(&for_alice_key_image)),
            received_block_index: Some(12),
            spent_block_index: None,
            confirmation: None,
            account_id: Some(alice_account_id.to_string()),
            shared_secret: Some(shared_secret.encode_to_vec()),
            memo_type: Some(0),
            is_synced_to_t3: false,
        };

        assert_eq!(expected_txo, txos[0]);

        // Verify that the status filter works as well
        let unspent = Txo::list_unspent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            None,
            None,
            conn,
        )
        .unwrap();
        assert_eq!(unspent.len(), 1);

        // Now we'll "spend" the TXO by sending it to ourselves, but at a subaddress we
        // have not yet assigned. At the DB layer, we accomplish this by
        // constructing the output txos, then logging sent and received for this
        // account.
        let (mut transaction_log, unsigned_tx_proposal) = create_test_unsigned_txproposal_and_log(
            alice_account_key.clone(),
            alice_account_key.subaddress(4),
            33 * MOB,
            wallet_db.clone(),
            ledger_db.clone(),
        );

        check_associated_txos_status(
            conn,
            &transaction_log,
            TxoStatus::Unspent,
            TxoStatus::Created,
            TxoStatus::Created,
        );

        let tx_proposal = unsigned_tx_proposal.sign(&alice_account).await.unwrap();
        // There should be 2 outputs, one to dest and one change
        assert_eq!(tx_proposal.tx.prefix.outputs.len(), 2);
        transaction_log = TransactionLog::log_signed(
            tx_proposal.clone(),
            "".to_string(),
            &AccountID::from(&alice_account_key).to_string(),
            conn,
        )
        .unwrap();

        check_associated_txos_status(
            conn,
            &transaction_log,
            TxoStatus::Unspent,
            TxoStatus::Created,
            TxoStatus::Created,
        );

        transaction_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&alice_account_key).to_string(),
            conn,
        )
        .unwrap();

        check_associated_txos_status(
            conn,
            &transaction_log,
            TxoStatus::Pending,
            TxoStatus::Pending,
            TxoStatus::Pending,
        );

        let associated_txos = transaction_log.get_associated_txos(conn).unwrap();
        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();

        assert_eq!(minted_txo.value as u64, 33 * MOB);
        assert_eq!(change_txo.value as u64, 967 * MOB - Mob::MINIMUM_FEE);

        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                tx_proposal.change_txos[0].tx_out.clone(),
                tx_proposal.payload_txos[0].tx_out.clone(),
            ],
            &[for_alice_key_image],
            &mut rng,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Now we'll process these Txos and verify that the TXO was "spent."
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, &logger);

        check_associated_txos_status(
            conn,
            &transaction_log,
            TxoStatus::Spent,
            TxoStatus::Orphaned,
            TxoStatus::Unspent,
        );

        // We should now have 3 txos for this account - one spent, one change (minted),
        // and one minted (destined for alice).
        let txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 3);

        // test spent
        let spent_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            Some(TxoStatus::Spent),
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(spent_txos.len(), 1);

        // test unspent
        let unspent_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            Some(TxoStatus::Unspent),
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent_txos.len(), 1);

        // Check that we have 2 spendable (1 is orphaned)
        let spendable: Vec<&Txo> = txos.iter().filter(|f| f.key_image.is_some()).collect();
        assert_eq!(spendable.len(), 2);

        // Check that we have one spent - went from [Received, Unspent] -> [Received,
        // Spent]
        let spent = Txo::list_spent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            None,
            None,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(spent.len(), 1);
        assert_eq!(
            spent[0].key_image,
            Some(mc_util_serial::encode(&for_alice_key_image))
        );
        assert_eq!(spent[0].spent_block_index.unwrap(), 13);

        // Check that we have one orphaned - went from [Minted, Secreted] -> [Minted,
        // Orphaned]
        let orphaned = Txo::list_orphaned(
            Some(&alice_account_id.to_string()),
            Some(0),
            None,
            None,
            None,
            None,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(orphaned.len(), 1);
        assert!(orphaned[0].key_image.is_none());
        assert_eq!(orphaned[0].received_block_index.unwrap(), 13);
        assert!(orphaned[0].account_id.is_some());

        // Check that we have one unspent (change) - went from [Minted, Secreted] ->
        // [Minted, Unspent]
        let unspent = Txo::list_unspent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            None,
            None,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent.len(), 1);
        assert_eq!(unspent[0].received_block_index.unwrap(), 13);
        // Store the key image for when we spend this Txo below
        let for_bob_key_image: KeyImage =
            mc_util_serial::decode(&unspent[0].key_image.clone().unwrap()).unwrap();

        // Note: To receive at Subaddress 4, we need to add an assigned subaddress
        // (currently this Txo is be orphaned). We add thrice, because currently
        // assigned subaddress is at 1.
        for _ in 0..3 {
            AssignedSubaddress::create_next_for_account(
                &alice_account_id.to_string(),
                "",
                &ledger_db,
                &mut wallet_db.get_pooled_conn().unwrap(),
            )
            .unwrap();
        }

        let alice_account =
            Account::get(&alice_account_id, &mut wallet_db.get_pooled_conn().unwrap()).unwrap();
        assert_eq!(alice_account.next_block_index, 14);
        assert_eq!(
            alice_account
                .next_subaddress_index(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            5
        );

        // Verify that there are two unspent txos - the one that was previously
        // orphaned, and change.
        let unspent = Txo::list_unspent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            None,
            None,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent.len(), 2);

        let updated_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // There are now 3 total Txos for our account
        assert_eq!(updated_txos.len(), 3);

        // Verify that there is one change Txo in our current Txos
        let change: Vec<&Txo> = updated_txos
            .iter()
            .filter(|f| {
                if let Some(subaddress_index) = f.subaddress_index {
                    subaddress_index as u64 == CHANGE_SUBADDRESS_INDEX
                } else {
                    false
                }
            })
            .collect();
        assert_eq!(change.len(), 1);

        // Create a new account and send some MOB to it
        let bob_root_id = RootIdentity::from_random(&mut rng);
        let bob_account_key = AccountKey::from(&bob_root_id);
        let (bob_account_id, _public_address_b58) = Account::create_from_root_entropy(
            &bob_root_id.root_entropy,
            Some(1),
            None,
            None,
            "Bob's Main Account",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let (transaction_log, tx_proposal) = create_test_minted_and_change_txos(
            alice_account_key.clone(),
            bob_account_key.subaddress(0),
            72 * MOB,
            wallet_db.clone(),
            ledger_db.clone(),
        )
        .await;

        let associated_txos = transaction_log
            .get_associated_txos(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();

        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();

        assert_eq!(minted_txo.value as u64, 72 * MOB);
        assert_eq!(change_txo.value as u64, 928 * MOB - (2 * Mob::MINIMUM_FEE));

        // Add the minted Txos to the ledger
        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                tx_proposal.change_txos[0].tx_out.clone(),
                tx_proposal.payload_txos[0].tx_out.clone(),
            ],
            &[for_bob_key_image],
            &mut rng,
        );

        // Process the latest block for Bob (note, Bob is starting to sync from block 0)
        let _bob_account = manually_sync_account(&ledger_db, &wallet_db, &bob_account_id, &logger);
        // Process the latest block for Alice
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, &logger);

        check_associated_txos_status(
            conn,
            &transaction_log,
            TxoStatus::Spent,
            TxoStatus::Secreted,
            TxoStatus::Unspent,
        );

        // We should now have 1 txo in Bob's account.
        let txos = Txo::list_for_account(
            &AccountID::from(&bob_account_key).to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        let bob_txo = txos[0].clone();
        assert_eq!(bob_txo.subaddress_index.unwrap(), 0);
        assert!(bob_txo.key_image.is_some());
    }

    fn check_associated_txos_status(
        conn: Conn,
        transaction_log: &TransactionLog,
        expected_input_status: TxoStatus,
        expected_output_status: TxoStatus,
        expected_change_status: TxoStatus,
    ) {
        let mut associated_txos = transaction_log.get_associated_txos(conn).unwrap();
        associated_txos
            .inputs
            .sort_by(|a, b| (b.spent_block_index).cmp(&a.spent_block_index));
        let input_txo = associated_txos.inputs.first().unwrap();
        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();
        assert_eq!(input_txo.status(conn).unwrap(), expected_input_status);
        assert_eq!(minted_txo.status(conn).unwrap(), expected_output_status);
        assert_eq!(change_txo.status(conn).unwrap(), expected_change_status);
    }

    #[test_with_logger]
    fn test_list_spendable_with_large_values(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(1),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            conn,
        )
        .unwrap();

        // Create some very large TXOs for the account, above i64::MAX
        let (_txo_hex, _txo, _key_image) = create_test_received_txo(
            &account_key,
            0,
            Amount::new(u64::MAX, Mob::ID),
            145,
            &mut rng,
            &wallet_db,
        );

        let (_txo_hex, _txo, _key_image) = create_test_received_txo(
            &account_key,
            0,
            Amount::new(u64::MAX, Mob::ID),
            146,
            &mut rng,
            &wallet_db,
        );

        // Create some small TXOs for the account, more than the max number of TXOs we
        // can use as inputs per transaction (16). 20 should do.
        for i in 1..=20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new(100, Mob::ID), // 100.0 MOB
                (146 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        let spendable_txos = Txo::list_spendable(
            Some(&account_id_hex.to_string()),
            None,
            None,
            0,
            Mob::MINIMUM_FEE,
            conn,
        )
        .unwrap();

        // Max spendable SHOULD be the largest 16 txos in the wallet (which is the
        // limit enforced by consensus), which would be u64::MAX * 2 + (100 * 14) -
        // Mob::MINIMUM_FEE = u64::MAX * 2 + 1400 - Mob::MINIMUM_FEE
        assert_eq!(
            spendable_txos.max_spendable_in_wallet,
            (u64::MAX as u128) * 2 + (100 * 14) - Mob::MINIMUM_FEE as u128
        );
    }

    #[test_with_logger]
    fn test_select_txos_for_value(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(1),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account
        // [100, 200, 300, ... 2000]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new(100 * MOB * i, Mob::ID), // 100.0 MOB * i
                144 + i,
                &mut rng,
                &wallet_db,
            );
        }

        // Greedily take smallest to exact value
        let txos_for_value = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            300 * MOB as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value as u64));
        assert_eq!(result_set, HashSet::from_iter([100 * MOB, 200 * MOB]));

        // Once we include the fee, we need another txo
        let txos_for_value = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            (300 * MOB + Mob::MINIMUM_FEE) as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value as u64));
        assert_eq!(
            result_set,
            HashSet::from_iter([100 * MOB, 200 * MOB, 300 * MOB])
        );

        // Setting max spendable value gives us insufficient funds - only allows 100
        let res = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            (300 * MOB + Mob::MINIMUM_FEE) as u128,
            Some(200 * MOB),
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        );

        match res {
            Err(WalletDbError::InsufficientFundsUnderMaxSpendable(_)) => {}
            Ok(_) => panic!("Should error with InsufficientFundsUnderMaxSpendable"),
            Err(_) => panic!("Should error with InsufficientFundsUnderMaxSpendable"),
        }

        // sum(300..1800) to get a window where we had to increase past the smallest
        // txos, and also fill up all 16 input slots.
        let txos_for_value = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            16800 * MOB as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value as u64));
        assert_eq!(
            result_set,
            HashSet::from_iter([
                300 * MOB,
                400 * MOB,
                500 * MOB,
                600 * MOB,
                700 * MOB,
                800 * MOB,
                900 * MOB,
                1000 * MOB,
                1100 * MOB,
                1200 * MOB,
                1300 * MOB,
                1400 * MOB,
                1500 * MOB,
                1600 * MOB,
                1700 * MOB,
                1800 * MOB,
            ])
        );
    }

    #[test_with_logger]
    fn test_select_txos_locked_when_flagged(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(1),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account
        // [100, 200, 300, ... 2000]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new(100 * MOB * i, Mob::ID), // 100.0 MOB * i
                144 + i,
                &mut rng,
                &wallet_db,
            );
        }

        // sum(300..1800) to get a window where we had to increase past the smallest
        // txos, and also fill up all 16 input slots.
        Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            16800 * MOB as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let res = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            16800 * MOB as u128,
            Some(100 * MOB),
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        );

        match res {
            Err(WalletDbError::InsufficientFundsUnderMaxSpendable(_)) => {}
            Ok(_) => panic!("Should error with InsufficientFundsUnderMaxSpendable, but got Ok"),
            Err(e) => panic!(
                "Should error with InsufficientFundsUnderMaxSpendable, but got {:?}",
                e
            ),
        }
    }

    #[test_with_logger]
    fn test_select_txos_fragmented(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account. Total value is 2000, but max can spend is
        // 1600 [100, 100, ... 100]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new(100 * MOB, Mob::ID),
                144 + i as u64,
                &mut rng,
                &wallet_db,
            );
        }

        let res = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(), // FIXME: WS-11 - take AccountID
            1800 * MOB as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        );
        match res {
            Err(WalletDbError::InsufficientFundsFragmentedTxos) => {}
            Ok(_) => panic!("Should error with InsufficientFundsFragmentedTxos"),
            Err(e) => panic!(
                "Should error with InsufficientFundsFragmentedTxos but got {:?}",
                e
            ),
        }
    }

    #[async_test_with_logger]
    async fn test_create_minted(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let root_id = RootIdentity::from_random(&mut rng);
        let src_account = AccountKey::from(&root_id);

        // Seed our ledger with some utxos for the src_account
        let known_recipients = vec![src_account.subaddress(0)];
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());

        Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Process the txos in the ledger into the DB
        manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID::from(&src_account),
            &logger,
        );

        let recipient =
            AccountKey::from(&RootIdentity::from_random(&mut rng)).subaddress(rng.next_u64());

        let txos = Txo::list_for_account(
            &AccountID::from(&src_account).to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(txos.len(), 12);

        let (transaction_log, _) = create_test_minted_and_change_txos(
            src_account,
            recipient,
            MOB,
            wallet_db.clone(),
            ledger_db,
        )
        .await;

        let associated_txos = transaction_log
            .get_associated_txos(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();

        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();

        assert_eq!(minted_txo.value as u64, MOB);
        assert!(minted_txo.account_id.is_none());

        assert_eq!(change_txo.value as u64, 4999 * MOB - Mob::MINIMUM_FEE);
        assert!(change_txo.account_id.is_none());
    }

    // Test that the confirmation number validates correctly.
    #[async_test_with_logger]
    async fn test_validate_confirmation(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // The account which will receive the Txo
        log::info!(logger, "Creating account");
        let root_id = RootIdentity::from_random(&mut rng);
        let recipient_account_key = AccountKey::from(&root_id);
        let recipient_account_id = AccountID::from(&recipient_account_key);
        Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "Alice",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Start sync thread
        log::info!(logger, "Starting sync thread");

        log::info!(logger, "Creating a random sender account");
        let sender_account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );
        let sender_account_id = AccountID::from(&sender_account_key);

        // Create TxProposal from the sender account, which contains the Confirmation
        // Number
        log::info!(logger, "Creating transaction builder");
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let sender_account = Account::get(&AccountID::from(&sender_account_key), conn).unwrap();

        let mut builder: WalletTransactionBuilder<MockFogPubkeyResolver> =
            WalletTransactionBuilder::new(
                AccountID::from(&sender_account_key).to_string(),
                ledger_db.clone(),
                get_resolver_factory(&mut rng).unwrap(),
            );
        builder
            .add_recipient(
                recipient_account_key.default_subaddress(),
                50 * MOB,
                Mob::ID,
            )
            .unwrap();
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        let unsigned_tx_proposal = builder
            .build(TransactionMemo::RTH(None, None), conn)
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&sender_account).await.unwrap();

        // Sleep to make sure that the foreign keys exist
        std::thread::sleep(Duration::from_secs(3));

        // Let's log this submitted Tx for the sender, which will create_minted for the
        // sent Txo
        log::info!(logger, "Logging submitted transaction");
        let tx_log = TransactionLog::log_submitted(
            &proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &sender_account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Now we need to let this txo hit the ledger, which will update sender and
        // receiver
        log::info!(logger, "Adding block from submitted");
        add_block_with_tx(&mut ledger_db, proposal.tx, &mut rng);

        // Now let our sync thread catch up for both sender and receiver
        log::info!(logger, "Manually syncing account");
        manually_sync_account(&ledger_db, &wallet_db, &recipient_account_id, &logger);
        manually_sync_account(&ledger_db, &wallet_db, &sender_account_id, &logger);

        // Then let's make sure we received the Txo on the recipient account
        log::info!(logger, "Listing all Txos for recipient account");
        let txos = Txo::list_for_account(
            &recipient_account_id.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        let received_txo = txos[0].clone();

        // Note: Because this txo is both received and sent, between two different
        // accounts, its confirmation number does get updated. Typically, received txos
        // have None for the confirmation number.
        assert!(received_txo.confirmation.is_some());

        // Get the txo from the sent perspective
        log::info!(logger, "Listing all Txos for sender account");
        let sender_txos = Txo::list_for_account(
            &sender_account_id.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // We seeded with 3 received (70, 80, 90), and a change txo
        assert_eq!(sender_txos.len(), 4);

        // Get the associated Txos with the transaction log
        log::info!(logger, "Getting associated Txos with the transaction");
        let associated = tx_log
            .get_associated_txos(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();
        let sent_outputs = associated.outputs;
        assert_eq!(sent_outputs.len(), 1);
        let sent_txo_details = Txo::get(
            &sent_outputs[0].0.id,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // These two txos should actually be the same txo, and the account_txo_status is
        // what differentiates them.
        assert_eq!(sent_txo_details, received_txo);

        assert!(sent_txo_details.confirmation.is_some());
        let confirmation: TxOutConfirmationNumber =
            mc_util_serial::decode(&sent_txo_details.confirmation.unwrap()).unwrap();
        log::info!(logger, "Validating the confirmation number");
        let verified = Txo::validate_confirmation(
            &AccountID::from(&recipient_account_key),
            &received_txo.id,
            &confirmation,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert!(verified);
    }

    #[test_with_logger]
    fn test_select_txos_by_public_key(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (_account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Seed Txos
        let mut src_txos = Vec::new();
        for i in 0..10 {
            let (_txo_id, txo, _key_image) = create_test_received_txo(
                &account_key,
                i,
                Amount::new(i * MOB, Mob::ID),
                i,
                &mut rng,
                &wallet_db,
            );
            src_txos.push(txo);
        }
        let pubkeys: Vec<&CompressedRistrettoPublic> =
            src_txos.iter().map(|t| &t.public_key).collect();

        let txos_and_status =
            Txo::select_by_public_key(&pubkeys, &mut wallet_db.get_pooled_conn().unwrap())
                .expect("Could not get txos by public keys");
        assert_eq!(txos_and_status.len(), 10);

        let txos_and_status =
            Txo::select_by_public_key(&pubkeys[0..5], &mut wallet_db.get_pooled_conn().unwrap())
                .expect("Could not get txos by public keys");
        assert_eq!(txos_and_status.len(), 5);
    }

    #[test_with_logger]
    fn test_delete_unreferenced_txos(logger: Logger) {
        use crate::db::schema::txos;
        use diesel::dsl::count;

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(1),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        let account =
            Account::get(&account_id_hex, &mut wallet_db.get_pooled_conn().unwrap()).unwrap();

        // Create some txos.
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            0
        );
        for _ in 0..10 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new(100 * MOB, Mob::ID), // 100.0 MOB * i
                144_u64,
                &mut rng,
                &wallet_db,
            );
        }
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            10
        );

        let txos = Txo::list_for_account(
            &account_id_hex.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 10);

        // Delete the account. No Txos are left.
        account
            .delete(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();

        let txos = Txo::list_for_account(
            &account_id_hex.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 0);

        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            0
        );
    }

    #[test_with_logger]
    fn test_list_spendable_more_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            conn,
        )
        .unwrap();

        let txo_value = 100 * MOB;

        for i in 1..=20 {
            let (_txo_id, _txo, _key_image) = create_test_received_txo(
                &account_key,
                i,
                Amount::new(txo_value, Mob::ID),
                i,
                &mut rng,
                &wallet_db,
            );
        }

        let SpendableTxosResult {
            spendable_txos,
            max_spendable_in_wallet,
        } = Txo::list_spendable(
            Some(&account_id.to_string()),
            None,
            None,
            0,
            Mob::MINIMUM_FEE,
            conn,
        )
        .unwrap();

        assert_eq!(spendable_txos.len(), 20);
        assert_eq!(
            max_spendable_in_wallet as u64,
            txo_value * 16 - Mob::MINIMUM_FEE
        );
    }

    #[test_with_logger]
    fn test_list_spendable_less_than_min_fee(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            conn,
        )
        .unwrap();

        let txo_value = 100;

        for i in 1..=10 {
            let (_txo_id, _txo, _key_image) = create_test_received_txo(
                &account_key,
                i,
                Amount::new(txo_value, Mob::ID),
                i,
                &mut rng,
                &wallet_db,
            );
        }

        let SpendableTxosResult {
            spendable_txos,
            max_spendable_in_wallet,
        } = Txo::list_spendable(
            Some(&account_id.to_string()),
            None,
            None,
            0,
            Mob::MINIMUM_FEE,
            conn,
        )
        .unwrap();

        assert_eq!(spendable_txos.len(), 10);
        assert_eq!(max_spendable_in_wallet as u64, 0);
    }

    #[test_with_logger]
    fn test_list_spendable_max_spendable_value(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            conn,
        )
        .unwrap();

        let txo_value_low = 100 * MOB;
        let txo_value_high = 200 * MOB;

        for i in 1..=5 {
            let (_txo_id, _txo, _key_image) = create_test_received_txo(
                &account_key,
                i,
                Amount::new(txo_value_low, Mob::ID),
                i,
                &mut rng,
                &wallet_db,
            );
        }
        for i in 1..=5 {
            let (_txo_id, _txo, _key_image) = create_test_received_txo(
                &account_key,
                i,
                Amount::new(txo_value_high, Mob::ID),
                i,
                &mut rng,
                &wallet_db,
            );
        }
        // Create some txos with token id != 0 to make sure it doesn't select those
        for i in 1..=5 {
            create_test_received_txo(
                &account_key,
                i,
                Amount::new(txo_value_low, TokenId::from(1)),
                i,
                &mut rng,
                &wallet_db,
            );
        }
        for i in 1..=5 {
            create_test_received_txo(
                &account_key,
                i,
                Amount::new(txo_value_high, TokenId::from(1)),
                i,
                &mut rng,
                &wallet_db,
            );
        }

        let SpendableTxosResult {
            spendable_txos,
            max_spendable_in_wallet,
        } = Txo::list_spendable(
            Some(&account_id.to_string()),
            Some(100 * MOB),
            None,
            0,
            Mob::MINIMUM_FEE,
            conn,
        )
        .unwrap();

        assert_eq!(spendable_txos.len(), 5);
        assert_eq!(
            max_spendable_in_wallet as u64,
            txo_value_low * 5 - Mob::MINIMUM_FEE
        );
    }

    #[test_with_logger]
    fn test_unspent_txo_query(logger: Logger) {
        // make sure it only includes txos with key image and subaddress
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let amount = Amount::new(28922973268924, Mob::ID);

        let (txo, key_image) = create_test_txo_for_recipient(&account_key, 1, amount, &mut rng);

        // create 1 txo with no key image and no subaddress
        Txo::create_received(
            txo.clone(),
            None,
            None,
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let txos = Txo::list_unspent(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            conn,
        )
        .unwrap();
        assert_eq!(txos.len(), 0);

        // create 1 txo with subaddress, but not key image
        Txo::create_received(
            txo.clone(),
            Some(1),
            None,
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let txos = Txo::list_unspent(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            conn,
        )
        .unwrap();
        assert_eq!(txos.len(), 0);

        // create 1 txo with key image and subaddress
        Txo::create_received(
            txo,
            Some(1),
            Some(key_image),
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let txos = Txo::list_unspent(
            Some(&account_id.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            conn,
        )
        .unwrap();
        assert_eq!(txos.len(), 1);
    }

    fn setup_select_unspent_txos_tests(logger: Logger, fragmented: bool) -> (AccountID, WalletDb) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        if fragmented {
            let (_txo_id, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new(28922973268924, Mob::ID),
                15,
                &mut rng,
                &wallet_db,
            );

            for i in 1..=15 {
                let (_txo_id, _txo, _key_image) = create_test_received_txo(
                    &account_key,
                    i,
                    Amount::new(10000000000, Mob::ID),
                    i,
                    &mut rng,
                    &wallet_db,
                );
            }

            for i in 1..=20 {
                let (_txo_id, _txo, _key_image) = create_test_received_txo(
                    &account_key,
                    i,
                    Amount::new(1000000000, Mob::ID),
                    i,
                    &mut rng,
                    &wallet_db,
                );
            }

            for i in 1..=500 {
                let (_txo_id, _txo, _key_image) = create_test_received_txo(
                    &account_key,
                    i,
                    Amount::new(100000000, Mob::ID),
                    i,
                    &mut rng,
                    &wallet_db,
                );
            }
        } else {
            for i in 1..=20 {
                let (_txo_id, _txo, _key_image) = create_test_received_txo(
                    &account_key,
                    i,
                    Amount::new(i * MOB, Mob::ID),
                    i,
                    &mut rng,
                    &wallet_db,
                );
            }
            // Create some txos with token id != 0
            for i in 1..=20 {
                let (_txo_id, _txo, _key_image) = create_test_received_txo(
                    &account_key,
                    i,
                    Amount::new(i * MOB, TokenId::from(1)),
                    i,
                    &mut rng,
                    &wallet_db,
                );
            }
        }

        (account_id, wallet_db)
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_equal_max_spendable_in_account(logger: Logger) {
        let target_value = (200 * MOB - Mob::MINIMUM_FEE) as u128;
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            target_value,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 16);
        let sum: u64 = result.iter().map(|x| x.value as u64).sum();
        assert_eq!(target_value, (sum - Mob::MINIMUM_FEE) as u128);
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_over_max_spendable_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            201 * MOB as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        );

        assert!(result.is_err());
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_under_max_spendable_in_account_selects_dust(
        logger: Logger,
    ) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            3,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_over_total_mob_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            500 * MOB as u128,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        );
        assert!(result.is_err());
    }

    #[test_with_logger]
    fn test_select_unspent_txos_for_value_selects_correct_subset_of_txos_when_fragmented(
        logger: Logger,
    ) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, true);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            12400000000,
            None,
            0,
            Mob::MINIMUM_FEE,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 16);
        let sum: i64 = result.iter().map(|x| x.value).sum();
        assert_eq!(12400000000_i64, sum);
    }

    #[test_with_logger]
    fn test_create_received_with_memo(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let address_hash: ShortAddressHash = (&account_key.default_subaddress()).into();

        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let amount = Amount::new(1000 * MOB, Mob::ID);

        // Create a received txo without a memo, and show that no memo is created.
        let (txo, key_image) = create_test_txo_for_recipient(&account_key, 0, amount, &mut rng);

        let txo_id = Txo::create_received(
            txo,
            Some(0),
            Some(key_image),
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let txo = Txo::get(&txo_id, conn).unwrap();
        let memo = txo.memo(conn).unwrap();
        match memo {
            TxoMemo::Unused => {}
            _ => panic!("expected unused memo"),
        }

        // Create a txo with a memo, and get the memo from the wallet db.
        let (txo, key_image) = create_test_txo_for_recipient_with_memo(
            &account_key,
            0,
            amount,
            &mut rng,
            TransactionMemo::RTH(None, None),
        );

        let txo_id = Txo::create_received(
            txo,
            Some(0),
            Some(key_image),
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let txo = Txo::get(&txo_id, conn).unwrap();
        let memo = txo.memo(conn).expect("loading memo");
        match memo {
            TxoMemo::AuthenticatedSender(m) => {
                assert_eq!(m.sender_address_hash, address_hash.to_string());
                assert_eq!(m.payment_request_id, None);
                assert_eq!(m.payment_intent_id, None);
            }
            _ => panic!("expected sender memo"),
        }
    }

    #[test_with_logger]
    fn test_get_memos_for_t3_sync_get_correct_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);

        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let amount = Amount::new(1000 * MOB, Mob::ID);

        // Create a received txo without a memo (empty memo), which should NOT be
        // returned by get_memos_for_t3_sync.
        let (txo_with_empty_memo, key_image) =
            create_test_txo_for_recipient(&account_key, 0, amount, &mut rng);

        Txo::create_received(
            txo_with_empty_memo,
            Some(0),
            Some(key_image),
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Create a txo with an AuthorizedSenderMemo, which should be returned by
        // get_memos_for_t3_sync
        let (txo_with_memo_1, key_image) = create_test_txo_for_recipient_with_memo(
            &account_key,
            0,
            amount,
            &mut rng,
            TransactionMemo::RTH(None, None),
        );

        Txo::create_received(
            txo_with_memo_1.clone(),
            Some(0),
            Some(key_image),
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Create a txo with an AuthorizedSenderMemoWithPaymentRequestId, which should
        // be returned by get_memos_for_t3_sync
        let (txo_with_memo_2, key_image) = create_test_txo_for_recipient_with_memo(
            &account_key,
            0,
            amount,
            &mut rng,
            TransactionMemo::RTH(None, Some(500)),
        );

        Txo::create_received(
            txo_with_memo_2.clone(),
            Some(0),
            Some(key_image),
            amount,
            15,
            &account_id.to_string(),
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        let txos_that_need_to_be_synced_to_t3 =
            Txo::get_txos_that_need_to_be_synced_to_t3(None, conn).unwrap();

        assert!(txos_that_need_to_be_synced_to_t3.len() == 2);

        assert!(txos_that_need_to_be_synced_to_t3
            .iter()
            .any(|txo| txo.public_key().unwrap() == txo_with_memo_1.public_key));

        assert!(txos_that_need_to_be_synced_to_t3
            .iter()
            .any(|txo| txo.public_key().unwrap() == txo_with_memo_2.public_key));
    }

    // FIXME: once we have create_minted, then select_txos test with no
    // FIXME: test update txo after tombstone block is exceeded
    // FIXME: test update txo after it has landed via key_image update
    // FIXME: test for selecting utxos from multiple subaddresses in one account
    // FIXME: test for one TXO belonging to multiple accounts with get
    // FIXME: test create_received for various permutations of multiple accounts
}
