// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Transaction model.

use diesel::prelude::*;
use hex_fmt::HexFmt;
use mc_common::HashMap;
use mc_transaction_core::{Amount, TokenId};
use std::{convert::TryFrom, fmt};

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{
            Account, NewTransactionInputTxo, NewTransactionLog, TransactionInputTxo,
            TransactionLog, TransactionOutputTxo, Txo,
        },
        schema::{
            transaction_input_txos, transaction_logs,
            transaction_logs::dsl::{id as dsl_id, transaction_logs as dsl_transaction_logs},
            transaction_output_txos, txos,
        },
        txo::{TxoID, TxoModel},
        Conn, WalletDbError,
    },
    service::models::tx_proposal::{OutputTxo, TxProposal, UnsignedTxProposal},
};

#[derive(Debug, PartialEq)]
pub struct TransactionId(pub String);

impl From<&TransactionLog> for TransactionId {
    fn from(tx_log: &TransactionLog) -> Self {
        Self(tx_log.id.clone())
    }
}

impl TryFrom<&TxProposal> for TransactionId {
    type Error = &'static str;
    fn try_from(_tx_proposal: &TxProposal) -> Result<Self, Self::Error> {
        Self::try_from(_tx_proposal.payload_txos.clone())
    }
}

impl TryFrom<&UnsignedTxProposal> for TransactionId {
    type Error = &'static str;
    fn try_from(_tx_proposal: &UnsignedTxProposal) -> Result<Self, Self::Error> {
        Self::try_from(_tx_proposal.payload_txos.clone())
    }
}

impl TryFrom<Vec<OutputTxo>> for TransactionId {
    type Error = &'static str;
    fn try_from(_payload_txos: Vec<OutputTxo>) -> Result<Self, Self::Error> {
        Ok(Self(
            HexFmt(
                _payload_txos
                    .iter()
                    .map(|txo| txo.tx_out.public_key)
                    .min()
                    .ok_or("no valid payload_txo")?,
            )
            .to_string(),
        ))
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum TxStatus {
    // The transaction log has been built but not yet signed
    Built,
    // The transaction log has been signed but not yet submitted to consensus
    Signed,
    // The transaction log has been submitted to consensus
    Pending,
    // The txos associated with this transaction log have appeared on the ledger, indicating that
    // the transaction was successful
    Succeeded,
    // Either consensus has rejected the tx proposal, or the tombstone block index has passed
    // without the txos in this transaction showing on the ledger
    Failed,
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxStatus::Built => write!(f, "built"),
            TxStatus::Signed => write!(f, "signed"),
            TxStatus::Pending => write!(f, "pending"),
            TxStatus::Succeeded => write!(f, "succeeded"),
            TxStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TxoType {
    // used as an input in a transaction
    Input,
    // used as an output in a transaction that is not change
    Payload,
    // used as an output in a transaction that is change
    Change,
}

impl fmt::Display for TxoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxoType::Input => write!(f, "input"),
            TxoType::Payload => write!(f, "payload"),
            TxoType::Change => write!(f, "change"),
        }
    }
}

#[derive(Debug)]
pub struct ValueMap(pub HashMap<TokenId, u64>);

#[derive(Debug)]
pub struct AssociatedTxos {
    pub inputs: Vec<Txo>,
    pub outputs: Vec<(Txo, String)>,
    pub change: Vec<(Txo, String)>,
}

impl TransactionLog {
    pub fn fee_amount(&self) -> Amount {
        Amount::new(
            self.fee_value as u64,
            TokenId::from(self.fee_token_id as u64),
        )
    }
}

#[rustfmt::skip]
pub trait TransactionLogModel {
    /// Get a transaction log from the transaction id.
    ///
    /// # Arguments
    ///
    ///| Name   | Purpose                                                | Notes                                     |
    ///|--------|--------------------------------------------------------|-------------------------------------------|
    ///| `id`   | The transaction ID to get transaction log.             | Transaction log must exist in the wallet. |
    ///| `conn` | An reference to the pool connection of wallet database |                                           |
    ///
    /// # Returns
    /// * TransactionLog
    fn get(
        id: &TransactionId, 
        conn: Conn
    ) -> Result<TransactionLog, WalletDbError>;

    /// Get the Txos associated with a given transaction id, grouped according to their type.
    ///
    /// # Arguments
    ///
    ///| Name   | Purpose                                                | Notes                                     |
    ///|--------|--------------------------------------------------------|-------------------------------------------|
    ///| `conn` | An reference to the pool connection of wallet database |                                           |
    ///
    /// # Returns:
    /// * AssoiatedTxos(inputs, outputs, change)
    fn get_associated_txos(&self, conn: Conn) -> Result<AssociatedTxos, WalletDbError>;


    /// Update the block index of where the associate transaction was submitted to a transaction log.
    ///
    /// # Arguments
    /// 
    ///| Name                    | Purpose                                                 | Notes |
    ///|-------------------------|---------------------------------------------------------|-------|
    ///| `submitted_block_index` | The block index of where the transaction was submitted. |       |
    ///| `conn`                  | An reference to the pool connection of wallet database  |       |
    ///
    /// # Returns:
    /// * unit
    fn update_submitted_block_index(
        &self,
        submitted_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Update arbitrary comments to a transaction log of an associate transaction .
    ///
    /// # Arguments
    /// 
    ///| Name      | Purpose                                                | Notes |
    ///|-----------|--------------------------------------------------------|-------|
    ///| `comment` | The arbitrary comments of the existing transaction.    |       |
    ///| `conn`    | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns:
    /// * unit
    fn update_comment(&self, comment: String, conn: Conn) -> Result<(), WalletDbError>;

    /// Update encoded value of the associate transaction and the tombstone_block_index to a transaction log.
    ///
    /// # Arguments
    /// 
    ///| Name                    | Purpose                                                                  | Notes                                                   |
    ///|-------------------------|--------------------------------------------------------------------------|---------------------------------------------------------|
    ///| `tx`                    | The encoded value of the associate transaction object.                   | Encoded value of a CryptoNote-style transaction object. |
    ///| `tombstone_block_index` | The block index at which this transaction is no longer considered valid. |                                                         |
    ///| `conn`                  | An reference to the pool connection of wallet database                   |                                                         |
    ///
    /// # Returns:
    /// * unit
    fn update_tx_and_tombstone_block_index(
        &self,
        tx: &[u8],
        tombstone_block_index: Option<i64>,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// List all transaction logs and their associated Txos for a given account.
    /// 
    /// # Arguments
    ///
    ///| Name              | Purpose                                                    | Notes                               |
    ///|-------------------|------------------------------------------------------------|-------------------------------------|
    ///| `account_id`      | The account id to scan for transaction logs.               | Account must exist in the database. |
    ///| `offset`          | The pagination offset. Results start at the offset index.  | Optional. Defaults to 0.            |
    ///| `limit`           | Limit for the number of results.                           | Optional.                           |
    ///| `min_block_index` | The minimum block index to find transaction logs from.     |                                     |
    ///| `max_block_index` | The maximum block index to find transaction logs from.     |                                     |
    ///| `conn`            | An reference to the pool connection of wallet database     |                                     |
    ///
    /// # Returns:
    /// * Vec(TransactionLog, AssociatedTxos(inputs, outputs, change))
    fn list_all(
        account_id: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletDbError>;

    /// Log a transaction that has been built but not yet signed.
    /// 
    /// # Arguments
    ///
    ///| Name                   | Purpose                                                 | Notes                               |
    ///|------------------------|---------------------------------------------------------|-------------------------------------|
    ///| `unsigned_tx_proposal` | The unsigned transaction proposal that will be logged.  |                                     |
    ///| `account_id`           | The account id to scan for transaction logs.            | Account must exist in the database. |
    ///| `conn`                 | An reference to the pool connection of wallet database  |                                     |
    ///
    /// # Returns:
    /// * TransactionLog
    fn log_built(
        unsigned_tx_proposal: &UnsignedTxProposal,
        account_id: &AccountID,
        conn: Conn,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Log a transaction that has been signed
    /// 
    /// # Arguments
    /// 
    ///| Name             | Purpose                                                   | Notes                               |
    ///|------------------|-----------------------------------------------------------|-------------------------------------|
    ///| `tx_proposal`    | The signed transaction proposal that will be logged.      |                                     |
    ///| `comment`        | The arbitrary comments of the current signed transaction. |                                     |
    ///| `account_id_hex` | The account id to scan for transaction logs.              | Account must exist in the database. |
    ///| `conn`           | An reference to the pool connection of wallet database    |                                     |
    ///
    /// # Returns:
    /// * TransactionLog
    fn log_signed(
        tx_proposal: TxProposal,
        comment: String,
        account_id_hex: &str,
        conn: Conn,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Log a submitted transaction.
    ///
    /// When submitting a transaction, we store relevant information to the
    /// transaction logs, and we also track information about each of the
    /// txos involved in the transaction.
    ///
    /// Note: We expect transactions created with this wallet to have one
    /// recipient, with the rest of the minted txos designated as
    /// change. Other wallets may choose to behave differently, but
    /// our TransactionLogs Table assumes this behavior.
    /// 
    /// # Arguments
    /// 
    ///| Name             | Purpose                                                      | Notes                               |
    ///|------------------|--------------------------------------------------------------|-------------------------------------|
    ///| `tx_proposal`    | The submitted transaction proposal that will be logged.      |                                     |
    ///| `block_index`    | The block index of where the transaction was submitted.      |                                     |
    ///| `comment`        | The arbitrary comments of the current submitted transaction. |                                     |
    ///| `account_id_hex` | The account id to scan for transaction logs.                 | Account must exist in the database. |
    ///| `conn`           | An reference to the pool connection of wallet database       |                                     |
    ///
    /// # Returns:
    /// * TransactionLog
    fn log_submitted(
        tx_proposal: &TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: Conn,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Remove all transaction logs for an account.
    /// 
    /// # Arguments
    ///
    ///| Name             | Purpose                                                | Notes                               |
    ///|------------------|--------------------------------------------------------|-------------------------------------|
    ///| `account_id_hex` | The account id to scan for transaction logs.           | Account must exist in the database. |
    ///| `conn`           | An reference to the pool connection of wallet database |                                     |
    ///
    /// # Returns
    /// * unit
    fn delete_all_for_account(
        account_id_hex: &str, 
        conn: Conn
    ) -> Result<(), WalletDbError>;

    /// Update the finalized block index to all pending transaction logs that have an output
    /// transaction corresponding to `transaction_output_txo_id_hex`.
    /// 
    /// # Arguments
    /// * `transaction_output_txo_id_hex` - The txo ID for which to get all transaction logs
    ///   associated with this txo ID.
    /// * `finalized_block_index` - The block index at which the transaction will be completed and
    ///   finalized.
    /// * `conn` - A reference to the pool connection of wallet database
    fn update_pending_associated_with_txo_to_succeeded(
        transaction_output_txo_id_hex: &str,
        finalized_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError>;
    
     /// Update all transaction logs that have an input transaction corresponding to
    /// `transaction_input_txo_id_hex` to failed.
    ///
    /// Note: When processing inputs and outputs from the same block, be sure to mark the
    /// appropriate transaction logs as succeeded prior to calling this method. See
    /// `update_pending_associated_with_txo_to_succeeded()`.
    ///
    /// # Arguments
    /// * `transaction_input_txo_id_hex` - The txo ID for which to get all transaction logs
    ///   associated with this txo ID.
    /// * `conn` - A reference to the pool connection of wallet database
    fn update_consumed_txo_to_failed(
        transaction_input_txo_id_hex: &str,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Set the status of a transaction log to failed if its tombstone_block_index is less than the given block index.
    /// 
    /// # Arguments
    ///
    ///| Name          | Purpose                                                                              | Notes |
    ///|---------------|--------------------------------------------------------------------------------------|-------|
    ///| `block_index` | The block index used for comparing the tombstone_block_index of the transaction log. |       |
    ///| `conn`        | An reference to the pool connection of wallet database                               |       |
    ///
    /// # Returns
    /// * unit
    fn update_pending_exceeding_tombstone_block_index_to_failed(
        account_id: &AccountID,
        block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Retrieve the status of an associated transaction from a transaction log.
    /// 
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * TxStatus
    fn status(&self) -> TxStatus;
    
    /// Get the total value of transaction outputs for given token id from current transaction log instances.
    /// 
    /// # Arguments
    /// 
    ///| Name       | Purpose                                                | Notes |
    ///|------------|--------------------------------------------------------|-------|
    ///| `token_id` | The id of a supported type of token.                   |       |
    ///| `conn`     | An reference to the pool connection of wallet database |       |
    ///
    /// # Returns
    /// * aggreagated value (u64)
    fn value_for_token_id(
        &self, 
        token_id: TokenId, 
        conn: Conn
    ) -> Result<u64, WalletDbError>;
    
    /// Get the total value of transaction outputs for each token id from current transaction log instances.
    /// 
    /// # Arguments
    /// * None
    /// 
    /// # Returns
    /// * ValueMap<TokenId, aggreagated value (u64)>
    fn value_map(&self, conn: Conn) -> Result<ValueMap, WalletDbError>;
}

impl TransactionLogModel for TransactionLog {
    fn status(&self) -> TxStatus {
        if self.failed {
            TxStatus::Failed
        } else if self.finalized_block_index.is_some() {
            TxStatus::Succeeded
        } else if self.submitted_block_index.is_some() {
            TxStatus::Pending
        } else {
            TxStatus::Built
        }
    }

    fn get(id: &TransactionId, conn: Conn) -> Result<TransactionLog, WalletDbError> {
        match dsl_transaction_logs
            .filter(dsl_id.eq(id.to_string()))
            .get_result::<TransactionLog>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::TransactionLogNotFound(id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_associated_txos(&self, conn: Conn) -> Result<AssociatedTxos, WalletDbError> {
        let inputs: Vec<Txo> = txos::table
            .inner_join(transaction_input_txos::table)
            .filter(transaction_input_txos::transaction_log_id.eq(&self.id))
            .select(txos::all_columns)
            .load(conn)?;

        let payload: Vec<(Txo, String)> = txos::table
            .inner_join(transaction_output_txos::table)
            .filter(transaction_output_txos::transaction_log_id.eq(&self.id))
            .filter(transaction_output_txos::is_change.eq(false))
            .select((
                txos::all_columns,
                transaction_output_txos::recipient_public_address_b58,
            ))
            .load(conn)?;

        let change: Vec<(Txo, String)> = txos::table
            .inner_join(transaction_output_txos::table)
            .filter(transaction_output_txos::transaction_log_id.eq(&self.id))
            .filter(transaction_output_txos::is_change.eq(true))
            .select((
                txos::all_columns,
                transaction_output_txos::recipient_public_address_b58,
            ))
            .load(conn)?;

        Ok(AssociatedTxos {
            inputs,
            outputs: payload,
            change,
        })
    }

    fn update_submitted_block_index(
        &self,
        submitted_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        diesel::update(self)
            .set(transaction_logs::submitted_block_index.eq(Some(submitted_block_index as i64)))
            .execute(conn)?;

        Ok(())
    }

    fn update_comment(&self, comment: String, conn: Conn) -> Result<(), WalletDbError> {
        diesel::update(self)
            .set(transaction_logs::comment.eq(comment))
            .execute(conn)?;

        Ok(())
    }

    fn update_tx_and_tombstone_block_index(
        &self,
        tx: &[u8],
        tombstone_block_index: Option<i64>,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        diesel::update(self)
            .set((
                transaction_logs::tx.eq(tx),
                transaction_logs::tombstone_block_index.eq(tombstone_block_index),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn list_all(
        account_id: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
        conn: Conn,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletDbError> {
        let mut query = transaction_logs::table.into_boxed();

        if let Some(account_id) = account_id {
            query = query.filter(transaction_logs::account_id.eq(account_id));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(min_block_index) = min_block_index {
            query =
                query.filter(transaction_logs::submitted_block_index.ge(min_block_index as i64));
        }

        if let Some(max_block_index) = max_block_index {
            query =
                query.filter(transaction_logs::submitted_block_index.le(max_block_index as i64));
        }

        let transaction_logs: Vec<TransactionLog> = query
            .order(transaction_logs::submitted_block_index.desc())
            .load(conn)?;

        let results = transaction_logs
            .into_iter()
            .map(|log| {
                let associated_txos = log.get_associated_txos(conn)?;
                let value_map = log.value_map(conn)?;
                Ok((log, associated_txos, value_map))
            })
            .collect::<Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletDbError>>()?;

        Ok(results)
    }

    fn log_built(
        unsigned_tx_proposal: &UnsignedTxProposal,
        account_id: &AccountID,
        conn: Conn,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        Account::get(account_id, conn)?;

        let unsigned_tx = &unsigned_tx_proposal.unsigned_tx;
        let transaction_log_id = TransactionId::try_from(unsigned_tx_proposal)
            .map_err(|e| WalletDbError::InvalidArgument(e.to_string()))?;

        let new_transaction_log = NewTransactionLog {
            id: &transaction_log_id.to_string(),
            account_id: &account_id.to_string(),
            fee_value: unsigned_tx.tx_prefix.fee as i64,
            fee_token_id: unsigned_tx.tx_prefix.fee_token_id as i64,
            submitted_block_index: None,
            tombstone_block_index: None,
            finalized_block_index: None,
            comment: "",
            tx: &[],
            failed: false,
        };

        diesel::insert_into(transaction_logs::table)
            .values(&new_transaction_log)
            .execute(conn)?;

        // Get each input txo and add it to the transaction_input_txos
        // table for this transaction.
        for input_txo in unsigned_tx_proposal.unsigned_input_txos.iter() {
            let txo_id = TxoID::from(&input_txo.tx_out);
            let new_transaction_input_txo = NewTransactionInputTxo {
                transaction_log_id: &transaction_log_id.to_string(),
                txo_id: &txo_id.to_string(),
            };

            diesel::insert_into(transaction_input_txos::table)
                .values(&new_transaction_input_txo)
                .execute(conn)?;
        }

        for payload_txo in unsigned_tx_proposal.payload_txos.iter() {
            Txo::create_new_output(payload_txo, false, &transaction_log_id, conn)?;
        }

        for change_txo in unsigned_tx_proposal.change_txos.iter() {
            Txo::create_new_output(change_txo, true, &transaction_log_id, conn)?;
        }

        TransactionLog::get(&transaction_log_id, conn)
    }

    fn log_signed(
        tx_proposal: TxProposal,
        comment: String,
        account_id_hex: &str,
        conn: Conn,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        let transaction_log_id = TransactionId::try_from(&tx_proposal)
            .map_err(|e| WalletDbError::InvalidArgument(e.to_string()))?;
        let tx = mc_util_serial::encode(&tx_proposal.tx);

        match TransactionLog::get(&transaction_log_id, conn) {
            Ok(transaction_log) => {
                // If the transaction log already exists, we just need to update
                // the input txos with their key images that
                // were generated during signing and the tx bytes with the
                // signed tx and the tombstone block index.
                for input_txo in tx_proposal.input_txos.iter() {
                    let txo_id = TxoID::from(&input_txo.tx_out);
                    Txo::update_key_image(&txo_id.to_string(), &input_txo.key_image, None, conn)?;
                }
                transaction_log.update_comment(comment, conn)?;
                transaction_log.update_tx_and_tombstone_block_index(
                    &tx,
                    Some(tx_proposal.tx.prefix.tombstone_block as i64),
                    conn,
                )?;
            }
            Err(WalletDbError::TransactionLogNotFound(_)) => {
                let new_transaction_log = NewTransactionLog {
                    id: &transaction_log_id.to_string(),
                    account_id: account_id_hex,
                    fee_value: tx_proposal.tx.prefix.fee as i64,
                    fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
                    submitted_block_index: None,
                    tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
                    finalized_block_index: None,
                    comment: &comment,
                    tx: &tx,
                    failed: false,
                };

                diesel::insert_into(crate::db::schema::transaction_logs::table)
                    .values(&new_transaction_log)
                    .execute(conn)?;

                for txo in tx_proposal.input_txos.iter() {
                    let txo_id = TxoID::from(&txo.tx_out);
                    Txo::update_key_image(&txo_id.to_string(), &txo.key_image, None, conn)?;
                    let transaction_input_txo = NewTransactionInputTxo {
                        transaction_log_id: &transaction_log_id.to_string(),
                        txo_id: &txo_id.to_string(),
                    };

                    diesel::insert_into(crate::db::schema::transaction_input_txos::table)
                        .values(&transaction_input_txo)
                        .execute(conn)?;
                }

                for output_txo in tx_proposal.payload_txos.iter() {
                    Txo::create_new_output(output_txo, false, &transaction_log_id, conn)?;
                }

                for change_txo in tx_proposal.change_txos.iter() {
                    Txo::create_new_output(change_txo, true, &transaction_log_id, conn)?;
                }
            }
            Err(e) => return Err(e),
        }

        TransactionLog::get(&transaction_log_id, conn)
    }

    fn log_submitted(
        tx_proposal: &TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: Conn,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        let transaction_log_id = TransactionId::try_from(tx_proposal)
            .map_err(|e| WalletDbError::InvalidArgument(e.to_string()))?;
        let tx = mc_util_serial::encode(&tx_proposal.tx);

        match TransactionLog::get(&transaction_log_id, conn) {
            Ok(transaction_log) => {
                transaction_log.update_submitted_block_index(block_index, conn)?;
                transaction_log.update_comment(comment, conn)?;
            }

            Err(WalletDbError::TransactionLogNotFound(_)) => {
                let new_transaction_log = NewTransactionLog {
                    id: &transaction_log_id.to_string(),
                    account_id: account_id_hex,
                    fee_value: tx_proposal.tx.prefix.fee as i64,
                    fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
                    submitted_block_index: Some(block_index as i64),
                    tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
                    finalized_block_index: None,
                    comment: &comment,
                    tx: &tx,
                    failed: false,
                };

                diesel::insert_into(crate::db::schema::transaction_logs::table)
                    .values(&new_transaction_log)
                    .execute(conn)?;

                for input_txo in tx_proposal.input_txos.iter() {
                    let txo_id = TxoID::from(&input_txo.tx_out);
                    Txo::update_key_image(&txo_id.to_string(), &input_txo.key_image, None, conn)?;
                    let transaction_input_txo = NewTransactionInputTxo {
                        transaction_log_id: &transaction_log_id.to_string(),
                        txo_id: &txo_id.to_string(),
                    };

                    diesel::insert_into(crate::db::schema::transaction_input_txos::table)
                        .values(&transaction_input_txo)
                        .execute(conn)?;
                }

                for output_txo in tx_proposal.payload_txos.iter() {
                    Txo::create_new_output(output_txo, false, &transaction_log_id, conn)?;
                }

                for change_txo in tx_proposal.change_txos.iter() {
                    Txo::create_new_output(change_txo, true, &transaction_log_id, conn)?;
                }
            }

            Err(e) => {
                return Err(e);
            }
        }

        TransactionLog::get(&transaction_log_id, conn)
    }

    fn delete_all_for_account(account_id_hex: &str, conn: Conn) -> Result<(), WalletDbError> {
        let transaction_input_txos: Vec<TransactionInputTxo> = transaction_input_txos::table
            .inner_join(transaction_logs::table)
            .filter(transaction_logs::account_id.eq(account_id_hex))
            .select(transaction_input_txos::all_columns)
            .load(conn)?;

        for transaction_input_txo in transaction_input_txos {
            diesel::delete(&transaction_input_txo).execute(conn)?;
        }

        let transaction_output_txos: Vec<TransactionOutputTxo> = transaction_output_txos::table
            .inner_join(transaction_logs::table)
            .filter(transaction_logs::account_id.eq(account_id_hex))
            .select(transaction_output_txos::all_columns)
            .load(conn)?;

        for transaction_output_txo in transaction_output_txos {
            diesel::delete(&transaction_output_txo).execute(conn)?;
        }

        diesel::delete(
            transaction_logs::table.filter(transaction_logs::account_id.eq(account_id_hex)),
        )
        .execute(conn)?;

        Ok(())
    }

    fn update_pending_associated_with_txo_to_succeeded(
        transaction_output_txo_id_hex: &str,
        finalized_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        // Find all submitted transaction logs associated with this txo that have not
        // yet been finalized (there should only ever be one).
        let transaction_log_ids: Vec<String> = transaction_logs::table
            .inner_join(transaction_output_txos::table)
            .filter(transaction_output_txos::txo_id.eq(transaction_output_txo_id_hex))
            .filter(transaction_logs::submitted_block_index.is_not_null()) // we actually sent this transaction
            .filter(transaction_logs::failed.eq(false)) // non-failed transactions
            .filter(transaction_logs::finalized_block_index.is_null()) // non-completed transactions
            .select(transaction_logs::id)
            .load(conn)?;

        diesel::update(
            transaction_logs::table.filter(transaction_logs::id.eq_any(transaction_log_ids)),
        )
        .set((transaction_logs::finalized_block_index.eq(finalized_block_index as i64),))
        .execute(conn)?;

        Ok(())
    }

    fn update_consumed_txo_to_failed(
        transaction_input_txo_id_hex: &str,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        let transaction_log_ids: Vec<String> = transaction_logs::table
            .inner_join(transaction_input_txos::table)
            .filter(transaction_input_txos::txo_id.eq(transaction_input_txo_id_hex))
            .filter(transaction_logs::failed.eq(false))
            .filter(transaction_logs::finalized_block_index.is_null())
            .select(transaction_logs::id)
            .load(conn)?;

        diesel::update(
            transaction_logs::table.filter(transaction_logs::id.eq_any(transaction_log_ids)),
        )
        .set((transaction_logs::failed.eq(true),))
        .execute(conn)?;

        Ok(())
    }

    fn update_pending_exceeding_tombstone_block_index_to_failed(
        account_id: &AccountID,
        block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        diesel::update(
            transaction_logs::table
                .filter(transaction_logs::account_id.eq(&account_id.0))
                .filter(transaction_logs::tombstone_block_index.lt(block_index as i64))
                .filter(transaction_logs::failed.eq(false))
                .filter(transaction_logs::finalized_block_index.is_null()),
        )
        .set((transaction_logs::failed.eq(true),))
        .execute(conn)?;

        Ok(())
    }

    fn value_for_token_id(&self, token_id: TokenId, conn: Conn) -> Result<u64, WalletDbError> {
        let associated_txos = self.get_associated_txos(conn)?;

        let output_total = associated_txos
            .outputs
            .iter()
            .filter(|(txo, _)| txo.token_id as u64 == *token_id)
            .map(|(txo, _)| txo.value as u64)
            .sum::<u64>();

        Ok(output_total)
    }

    fn value_map(&self, conn: Conn) -> Result<ValueMap, WalletDbError> {
        let associated_txos = self.get_associated_txos(conn)?;

        let mut value_map: HashMap<TokenId, u64> = HashMap::default();
        for (txo, _) in associated_txos.outputs.iter() {
            let token_id = TokenId::from(txo.token_id as u64);
            let value = value_map.entry(token_id).or_insert(0);
            *value += txo.value as u64;
        }
        Ok(ValueMap(value_map))
    }
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, CHANGE_SUBADDRESS_INDEX};
    use mc_common::logger::{async_test_with_logger, test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, tx::Tx, Token};
    use mc_transaction_extra::TxOutConfirmationNumber;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{
        assert_matches::assert_matches,
        collections::HashMap,
        ops::DerefMut,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::{
        db::{account::AccountID, transaction_log::TransactionId, txo::TxoStatus},
        service::{
            models::transaction_memo::TransactionMemo, sync::SyncThread,
            transaction_builder::WalletTransactionBuilder,
        },
        test_utils::{
            add_block_with_tx_outs, builder_for_random_recipient, create_test_txo_for_recipient,
            create_test_unsigned_txproposal_and_log, get_resolver_factory, get_test_ledger,
            manually_sync_account, random_account_with_seed_values, WalletDbTestContext, MOB,
        },
        util::b58::b58_encode_public_address,
    };

    #[async_test_with_logger]
    // Test the happy path for log_submitted. When a transaction is submitted to the
    // MobileCoin network, several things must happen for Full-Service to
    // maintain accurate transaction history.
    //
    // 1. The minted TXO(s) were created in the txos table
    // 2. The spent TXO(s) are marked as pending
    // 3. The change TXO(s) are marked as minted, secreted
    // 4. The transaction_log is created and added to the transaction_log table
    // 5. Once the change is received, it is marked as minted, unspent
    async fn test_log_submitted(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);
        builder
            .add_recipient(recipient.clone(), 50 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let tx_proposal = unsigned_tx_proposal.clone().sign(&account).await.unwrap();

        assert_eq!(
            TransactionId::try_from(&tx_proposal),
            TransactionId::try_from(&unsigned_tx_proposal)
        );

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        // The log's account ID matches the account_id which submitted the tx
        assert_eq!(tx_log.account_id, AccountID::from(&account_key).to_string());
        assert_eq!(tx_log.value_for_token_id(Mob::ID, conn).unwrap(), 50 * MOB);
        assert_eq!(tx_log.fee_value as u64, Mob::MINIMUM_FEE);
        assert_eq!(tx_log.fee_token_id as u64, *Mob::ID);
        assert_eq!(tx_log.status(), TxStatus::Pending);
        assert_eq!(
            tx_log.submitted_block_index,
            Some(ledger_db.num_blocks().unwrap() as i64)
        );
        // There is no comment for this submission
        assert_eq!(tx_log.comment, "");

        // The tx in the log matches the tx in the proposal
        let tx: Tx = mc_util_serial::decode(&tx_log.tx).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Check the associated_txos for this transaction_log are as expected
        let associated_txos = tx_log
            .get_associated_txos(wallet_db.get_pooled_conn().unwrap().deref_mut())
            .unwrap();

        // There is one associated input TXO to this transaction, and it is now pending.
        assert_eq!(associated_txos.inputs.len(), 1);
        let input_details = Txo::get(
            &associated_txos.inputs[0].id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(input_details.value as u64, 70 * MOB);
        assert_eq!(
            input_details
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Pending
        );
        assert_eq!(input_details.subaddress_index.unwrap(), 0);

        // There is one associated output TXO to this transaction, and its recipient
        // is the destination addr
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].1,
            b58_encode_public_address(&recipient).unwrap()
        );
        let output_details = Txo::get(
            &associated_txos.outputs[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(output_details.value as u64, 50 * MOB);

        // We cannot know any details about the received_to_account for this TXO, as it
        // was sent out of the wallet
        assert!(output_details.subaddress_index.is_none());

        // Assert change is as expected
        assert_eq!(associated_txos.change.len(), 1);
        let change_details = Txo::get(
            &associated_txos.change[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(change_details.value as u64, 20 * MOB - Mob::MINIMUM_FEE);

        // Note, this will still be marked as not change until the txo
        // appears on the ledger and the account syncs.
        // change becomes unspent once scanned.
        // The subaddress will also be set once received.
        assert_eq!(change_details.subaddress_index, None,);

        let key_images: Vec<KeyImage> = tx_proposal
            .input_txos
            .iter()
            .map(|txo| txo.key_image)
            .collect();

        // Note: This block doesn't contain the fee output.
        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                tx_proposal.change_txos[0].tx_out.clone(),
                tx_proposal.payload_txos[0].tx_out.clone(),
            ],
            &key_images,
            &mut rng,
        );

        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
        let _sync = manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(tx_log.account_id.to_string()),
            &logger,
        );

        let updated_tx_log = TransactionLog::get(
            &TransactionId::from(&tx_log),
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(updated_tx_log.status(), TxStatus::Succeeded);

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(
            updated_change_details.status(conn).unwrap(),
            TxoStatus::Unspent
        );
        assert_eq!(
            updated_change_details.account_id.unwrap(),
            tx_log.account_id
        );
        assert_eq!(
            updated_change_details.subaddress_index,
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    #[async_test_with_logger]
    async fn test_log_submitted_zero_change(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[100 * MOB, 200 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);
        // Add outlays all to the same recipient, so that we exceed u64::MAX in this tx
        let value = 100 * MOB - Mob::MINIMUM_FEE;
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();

        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let tx_proposal = unsigned_tx_proposal.sign(&account).await.unwrap();

        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        assert_eq!(tx_log.account_id, AccountID::from(&account_key).to_string());
        let associated_txos = tx_log
            .get_associated_txos(wallet_db.get_pooled_conn().unwrap().deref_mut())
            .unwrap();
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].1,
            b58_encode_public_address(&recipient).unwrap()
        );

        assert_eq!(tx_log.value_for_token_id(Mob::ID, conn).unwrap(), value);
        assert_eq!(tx_log.fee_value as u64, Mob::MINIMUM_FEE);
        assert_eq!(tx_log.fee_token_id as u64, *Mob::ID);
        assert_eq!(tx_log.status(), TxStatus::Pending);
        assert_eq!(
            tx_log.submitted_block_index.unwrap() as u64,
            ledger_db.num_blocks().unwrap()
        );
        assert_eq!(tx_log.comment, "");
        let tx: Tx = mc_util_serial::decode(&tx_log.tx).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Get associated Txos
        let associated = tx_log
            .get_associated_txos(wallet_db.get_pooled_conn().unwrap().deref_mut())
            .unwrap();
        assert_eq!(associated.inputs.len(), 1);
        assert_eq!(associated.outputs.len(), 1);
        assert_eq!(associated.change.len(), 1);
    }

    #[async_test_with_logger]
    async fn test_delete_transaction_logs_for_account(logger: Logger) {
        use diesel::dsl::count_star;

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );

        let account_id = AccountID::from(&account_key);

        // Build a transaction
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);
        builder.add_recipient(recipient, 50 * MOB, Mob::ID).unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let tx_proposal = unsigned_tx_proposal.sign(&account).await.unwrap();

        // Log submitted transaction from tx_proposal
        TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        // Check that we created transaction_logs and transaction_txo_types entries.
        assert_eq!(
            Ok(1),
            transaction_logs::table
                .select(count_star())
                .first(&mut wallet_db.get_pooled_conn().unwrap())
        );
        assert_eq!(
            Ok(1),
            transaction_input_txos::table
                .select(count_star())
                .first(&mut wallet_db.get_pooled_conn().unwrap())
        );
        assert_eq!(
            Ok(2),
            transaction_output_txos::table
                .select(count_star())
                .first(&mut wallet_db.get_pooled_conn().unwrap())
        );

        // Delete the transaction logs for one account.
        let result = TransactionLog::delete_all_for_account(
            &account_id.to_string(),
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        );
        assert!(result.is_ok());

        // For the given account, the transaction logs and the txo types are
        // deleted.
        assert_eq!(
            Ok(0),
            transaction_logs::table
                .select(count_star())
                .first(&mut wallet_db.get_pooled_conn().unwrap())
        );
        assert_eq!(
            Ok(0),
            transaction_input_txos::table
                .select(count_star())
                .first(&mut wallet_db.get_pooled_conn().unwrap())
        );
        assert_eq!(
            Ok(0),
            transaction_output_txos::table
                .select(count_star())
                .first(&mut wallet_db.get_pooled_conn().unwrap())
        );
    }

    // Test that transaction logging can handle submitting a value greater than
    // i64::Max Note: i64::Max is 9_223_372_036_854_775_807, or about 9.2M MOB.
    // The biggest MOB amount that can be represented on chain is u64::MAX,
    // 18_446_744_073_709_551_615, or about 18M MOB.
    //
    // This test confirms that submitting a transaction_log for < u64::Max, but >
    // i64::Max succeeds
    #[async_test_with_logger]
    async fn test_log_submitted_big_int(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[7_000_000 * MOB, 14_000_000 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction for > i64::Max
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);
        builder
            .add_recipient(recipient, 10_000_000 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let tx_proposal = unsigned_tx_proposal.sign(&account).await.unwrap();

        assert_eq!(
            tx_proposal.payload_txos[0].amount.value,
            10_000_000_000_000_000_000
        );

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        let pmob_value = tx_log.value_for_token_id(Mob::ID, conn).unwrap();
        assert_eq!(pmob_value, 10_000_000 * MOB);
    }

    // Test that logging a submitted transaction to self results in the inputs,
    // outputs, and change being handled correctly.
    //
    // By "handled correctly," we mean:
    //
    // 1. The inputs are marked as pending, until they are seen as spent in the
    // processed block 2. The outputs are marked as Minted & Secreted, until
    // they are processed, then they are    marked Minted & Unspent
    // 3. The change is marked Minted & Secreted, until it is processed; then it
    // is marked    Minted & Unspent
    //
    // Note: This is also testing 2 inputs, as opposed to the happy path test
    // above, which tests only 1 input.
    #[async_test_with_logger]
    async fn test_log_submitted_to_self(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[7 * MOB, 8 * MOB],
            &mut rng,
            &logger,
        );

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let mut builder = WalletTransactionBuilder::new(
            AccountID::from(&account_key).to_string(),
            ledger_db.clone(),
            get_resolver_factory(&mut rng).unwrap(),
        );
        // Add self at main subaddress as the recipient
        builder
            .add_recipient(account_key.subaddress(0), 12 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let tx_proposal = unsigned_tx_proposal.sign(&account).await.unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        // Get the associated txos for this transaction
        let associated_txos = tx_log
            .get_associated_txos(wallet_db.get_pooled_conn().unwrap().deref_mut())
            .unwrap();

        // There are two input TXOs to this transaction, and they are both now pending.
        assert_eq!(associated_txos.inputs.len(), 2);
        let input_details0 = Txo::get(
            &associated_txos.inputs[0].id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(input_details0.value, associated_txos.inputs[0].value);

        assert_eq!(
            input_details0
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Pending
        );
        assert_eq!(input_details0.subaddress_index, Some(0));

        let input_details1 = Txo::get(
            &associated_txos.inputs[1].id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(input_details1.value, associated_txos.inputs[1].value);

        assert_eq!(
            input_details1
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Pending
        );
        assert_eq!(input_details1.subaddress_index, Some(0));

        assert_eq!(
            input_details0.value as u64 + input_details1.value as u64,
            15 * MOB
        );

        // There is one associated output TXO to this transaction, and its recipient
        // is our own address
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].1,
            b58_encode_public_address(&account_key.subaddress(0)).unwrap()
        );
        let output_details = Txo::get(
            &associated_txos.outputs[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(output_details.value as u64, 12 * MOB);

        // We cannot know any details about the received_to_account for this TXO (until
        // it is scanned)
        assert!(output_details.subaddress_index.is_none());

        // Assert change is as expected
        assert_eq!(associated_txos.change.len(), 1);
        let change_details = Txo::get(
            &associated_txos.change[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        // Change = (8 + 7) - 12 - fee
        assert_eq!(change_details.value as u64, 3 * MOB - Mob::MINIMUM_FEE);
        assert_eq!(change_details.subaddress_index, None);

        // Now - we will add the spent Txos, outputs, and change to the ledger, so we
        // can scan and verify
        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                tx_proposal.change_txos[0].tx_out.clone(),
                tx_proposal.payload_txos[0].tx_out.clone(),
            ],
            &[
                mc_util_serial::decode(&input_details0.key_image.unwrap()).unwrap(),
                mc_util_serial::decode(&input_details1.key_image.unwrap()).unwrap(),
            ],
            &mut rng,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 15);
        let _sync = manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(tx_log.account_id.to_string()),
            &logger,
        );

        // Get the Input Txos again
        let updated_input_details0 = Txo::get(
            &associated_txos.inputs[0].id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        let updated_input_details1 = Txo::get(
            &associated_txos.inputs[1].id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        // The inputs are now spent
        assert_eq!(
            updated_input_details0
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Spent
        );
        assert_eq!(
            updated_input_details1
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Spent
        );

        // The received_to account is ourself, which is the same as the account
        // account_id in the transaction log. The type is "Received"
        assert_eq!(
            updated_input_details0.account_id,
            Some(tx_log.account_id.clone())
        );
        assert_eq!(updated_input_details0.subaddress_index, Some(0_i64));

        assert_eq!(
            updated_input_details1.account_id,
            Some(tx_log.account_id.clone())
        );
        assert_eq!(updated_input_details1.subaddress_index, Some(0_i64));

        // Get the output txo again
        let updated_output_details = Txo::get(
            &associated_txos.outputs[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        // The minted from account is ourself, and it is unspent, minted
        assert_eq!(
            updated_output_details
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Unspent
        );

        // The received to account is ourself, and it is unspent, minted
        assert_eq!(
            updated_output_details.account_id,
            Some(tx_log.account_id.clone())
        );

        // Received to main subaddress
        assert_eq!(updated_output_details.subaddress_index, Some(0_i64));

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(
            updated_change_details
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Unspent
        );
        assert_eq!(updated_change_details.account_id, Some(tx_log.account_id));
        assert_eq!(
            updated_change_details.subaddress_index,
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    #[async_test_with_logger]
    async fn test_log_built_signed_and_submitted(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);
        builder
            .add_recipient(recipient.clone(), 50 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();

        let tx_log =
            TransactionLog::log_built(&unsigned_tx_proposal, &AccountID::from(&account_key), conn)
                .unwrap();

        let expected_tx_log = TransactionLog {
            id: TransactionId::try_from(&unsigned_tx_proposal)
                .expect("Failed to convert UnsignedTxProposal to String")
                .to_string(),
            account_id: AccountID::from(&account_key).to_string(),
            fee_value: unsigned_tx_proposal.unsigned_tx.tx_prefix.fee as i64,
            fee_token_id: unsigned_tx_proposal.unsigned_tx.tx_prefix.fee_token_id as i64,
            submitted_block_index: None,
            tombstone_block_index: None,
            finalized_block_index: None,
            comment: "".to_string(),
            tx: vec![],
            failed: false,
        };

        assert_eq!(tx_log, expected_tx_log);

        let tx_proposal = unsigned_tx_proposal.clone().sign(&account).await.unwrap();
        let tx_bytes = mc_util_serial::encode(&tx_proposal.tx);

        assert_eq!(
            TransactionId::try_from(&tx_proposal),
            TransactionId::try_from(&unsigned_tx_proposal)
        );

        let tx_log = TransactionLog::log_signed(
            tx_proposal.clone(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        let expected_tx_log = TransactionLog {
            id: TransactionId::try_from(&unsigned_tx_proposal)
                .expect("Failed to convert UnsignedTxProposal to String")
                .to_string(),
            account_id: AccountID::from(&account_key).to_string(),
            fee_value: tx_proposal.tx.prefix.fee as i64,
            fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
            submitted_block_index: None,
            tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
            finalized_block_index: None,
            comment: "".to_string(),
            tx: tx_bytes.clone(),
            failed: false,
        };

        assert_eq!(tx_log, expected_tx_log);

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        let expected_tx_log = TransactionLog {
            id: TransactionId::try_from(&unsigned_tx_proposal)
                .expect("Failed to convert UnsignedTxProposal to String")
                .to_string(),
            account_id: AccountID::from(&account_key).to_string(),
            fee_value: tx_proposal.tx.prefix.fee as i64,
            fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
            submitted_block_index: Some(ledger_db.num_blocks().unwrap() as i64),
            tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
            finalized_block_index: None,
            comment: "".to_string(),
            tx: tx_bytes,
            failed: false,
        };
        assert_eq!(tx_log, expected_tx_log);
        assert_eq!(tx_log.value_for_token_id(Mob::ID, conn).unwrap(), 50 * MOB);
        assert_eq!(tx_log.status(), TxStatus::Pending);

        // Check the associated_txos for this transaction_log are as expected
        let associated_txos = tx_log
            .get_associated_txos(wallet_db.get_pooled_conn().unwrap().deref_mut())
            .unwrap();

        // There is one associated input TXO to this transaction, and it is now pending.
        assert_eq!(associated_txos.inputs.len(), 1);
        let input_details = Txo::get(
            &associated_txos.inputs[0].id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(input_details.value as u64, 70 * MOB);
        assert_eq!(
            input_details
                .status(wallet_db.get_pooled_conn().unwrap().deref_mut())
                .unwrap(),
            TxoStatus::Pending
        );
        assert_eq!(input_details.subaddress_index.unwrap(), 0);

        // There is one associated output TXO to this transaction, and its recipient
        // is the destination addr
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].1,
            b58_encode_public_address(&recipient).unwrap()
        );
        let output_details = Txo::get(
            &associated_txos.outputs[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(output_details.value as u64, 50 * MOB);

        // We cannot know any details about the received_to_account for this TXO, as it
        // was sent out of the wallet
        assert!(output_details.subaddress_index.is_none());

        // Assert change is as expected
        assert_eq!(associated_txos.change.len(), 1);
        let change_details = Txo::get(
            &associated_txos.change[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(change_details.value as u64, 20 * MOB - Mob::MINIMUM_FEE);

        // Note, this will still be marked as not change until the txo
        // appears on the ledger and the account syncs.
        // change becomes unspent once scanned.
        // The subaddress will also be set once received.
        assert_eq!(change_details.subaddress_index, None,);

        let key_images: Vec<KeyImage> = tx_proposal
            .input_txos
            .iter()
            .map(|txo| txo.key_image)
            .collect();

        // Note: This block doesn't contain the fee output.
        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                tx_proposal.change_txos[0].tx_out.clone(),
                tx_proposal.payload_txos[0].tx_out.clone(),
            ],
            &key_images,
            &mut rng,
        );

        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
        let _sync = manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(tx_log.account_id.to_string()),
            &logger,
        );

        let updated_tx_log = TransactionLog::get(
            &TransactionId::from(&tx_log),
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(updated_tx_log.status(), TxStatus::Succeeded);

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].0.id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        assert_eq!(
            updated_change_details.status(conn).unwrap(),
            TxoStatus::Unspent
        );
        assert_eq!(
            updated_change_details.account_id.unwrap(),
            tx_log.account_id
        );
        assert_eq!(
            updated_change_details.subaddress_index,
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    #[async_test_with_logger]
    async fn test_log_submitted_with_comment_change(logger: Logger) {
        // Test setup

        // log_submitted

        // Returned transaction log should be
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(
            ledger_db.clone(),
            wallet_db.clone(),
            Arc::new(Mutex::new(HashMap::<AccountID, bool>::new())),
            logger.clone(),
        );

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);
        builder.add_recipient(recipient, 50 * MOB, Mob::ID).unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(conn, None).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();

        let tx_log =
            TransactionLog::log_built(&unsigned_tx_proposal, &AccountID::from(&account_key), conn)
                .unwrap();

        let expected_tx_log = TransactionLog {
            id: TransactionId::try_from(&unsigned_tx_proposal)
                .expect("Failed to convert UnsignedTxProposal to String")
                .to_string(),
            account_id: AccountID::from(&account_key).to_string(),
            fee_value: unsigned_tx_proposal.unsigned_tx.tx_prefix.fee as i64,
            fee_token_id: unsigned_tx_proposal.unsigned_tx.tx_prefix.fee_token_id as i64,
            submitted_block_index: None,
            tombstone_block_index: None,
            finalized_block_index: None,
            comment: "".to_string(),
            tx: vec![],
            failed: false,
        };

        assert_eq!(tx_log, expected_tx_log);

        let tx_proposal = unsigned_tx_proposal.clone().sign(&account).await.unwrap();
        let tx_bytes = mc_util_serial::encode(&tx_proposal.tx);

        assert_eq!(
            TransactionId::try_from(&tx_proposal),
            TransactionId::try_from(&unsigned_tx_proposal)
        );

        let tx_log = TransactionLog::log_signed(
            tx_proposal.clone(),
            "first change".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        let expected_tx_log = TransactionLog {
            id: TransactionId::try_from(&unsigned_tx_proposal)
                .expect("Failed to convert UnsignedTxProposal to String")
                .to_string(),
            account_id: AccountID::from(&account_key).to_string(),
            fee_value: tx_proposal.tx.prefix.fee as i64,
            fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
            submitted_block_index: None,
            tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
            finalized_block_index: None,
            comment: "first change".to_string(),
            tx: tx_bytes.clone(),
            failed: false,
        };

        assert_eq!(tx_log, expected_tx_log);

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "second change".to_string(),
            &AccountID::from(&account_key).to_string(),
            conn,
        )
        .unwrap();

        let expected_tx_log = TransactionLog {
            id: TransactionId::try_from(&unsigned_tx_proposal)
                .expect("Failed to convert UnsignedTxProposal to String")
                .to_string(),
            account_id: AccountID::from(&account_key).to_string(),
            fee_value: tx_proposal.tx.prefix.fee as i64,
            fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
            submitted_block_index: Some(ledger_db.num_blocks().unwrap() as i64),
            tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
            finalized_block_index: None,
            comment: "second change".to_string(),
            tx: tx_bytes,
            failed: false,
        };

        assert_eq!(tx_log.tx, expected_tx_log.tx);
    }

    #[test]
    fn test_try_from_vec_output_txo_for_transaction_id() {
        let mut rng: StdRng = SeedableRng::from_entropy();
        let root_id = RootIdentity::from_random(&mut rng);
        let recipient_account_key = AccountKey::from(&root_id);
        let amount = 77;

        let num_loops = 10;

        for loop_number in 1..=num_loops {
            let mut output_vec: Vec<OutputTxo> = Vec::new();

            for _ in 1..=loop_number {
                let subaddress_index = 0;
                let (tx_out, _) = create_test_txo_for_recipient(
                    &recipient_account_key,
                    subaddress_index,
                    Amount::new(amount * MOB, Mob::ID),
                    &mut rng,
                );

                let output_txo = OutputTxo {
                    tx_out: tx_out.clone(),
                    recipient_public_address: recipient_account_key.subaddress(0),
                    confirmation_number: TxOutConfirmationNumber::default(),
                    amount: Amount::new(amount * MOB, Mob::ID),
                    shared_secret: None,
                };

                output_vec.push(output_txo);
            }

            let min_public_key = output_vec
                .iter()
                .map(|txo| txo.tx_out.public_key)
                .min()
                .unwrap();
            let transaction_id = TransactionId::try_from(output_vec).unwrap();

            assert_eq!(min_public_key.to_string(), transaction_id.0);
        }
    }

    #[test]
    fn test_try_from_empty_vec_output_txo_for_transaction_id() {
        let transaction_log_id = TransactionId::try_from(vec![]);

        assert_matches!(transaction_log_id, Err("no valid payload_txo"));
    }

    #[test_with_logger]
    fn fail_by_tombstone_block_only_fails_for_given_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let mut ledger_db = get_test_ledger(5, &[], 12, &mut rng);
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let account_keys = (0..3)
            .map(|_| {
                random_account_with_seed_values(
                    &wallet_db,
                    &mut ledger_db,
                    &[70 * MOB],
                    &mut rng,
                    &logger,
                )
            })
            .collect::<Vec<_>>();
        let accounts_and_logs: Vec<_> = account_keys
            .iter()
            .map(|account_key| {
                let account_id = AccountID::from(account_key);
                let (_, unsigned_tx_proposal) = create_test_unsigned_txproposal_and_log(
                    account_key.clone(),
                    account_key.default_subaddress(),
                    MOB,
                    wallet_db.clone(),
                    ledger_db.clone(),
                );
                let tx_proposal = unsigned_tx_proposal
                    .sign_with_local_signer(account_key)
                    .unwrap();

                let _ = TransactionLog::log_signed(
                    tx_proposal.clone(),
                    "".to_string(),
                    &AccountID::from(account_key).to_string(),
                    conn,
                )
                .unwrap();

                let transaction_log = TransactionLog::log_submitted(
                    &tx_proposal,
                    ledger_db.num_blocks().unwrap(),
                    "".to_string(),
                    &AccountID::from(account_key).to_string(),
                    conn,
                )
                .unwrap();
                (account_id, transaction_log)
            })
            .collect();

        let (account_ids, transaction_logs) = accounts_and_logs
            .iter()
            .cloned()
            .unzip::<_, _, Vec<_>, Vec<_>>();

        // Sanity check that all have the same tombstone block
        let tombstone_block_index = transaction_logs[0].tombstone_block_index.unwrap();
        for tx_log in &transaction_logs {
            assert_eq!(tx_log.tombstone_block_index, Some(tombstone_block_index));
        }

        TransactionLog::update_pending_exceeding_tombstone_block_index_to_failed(
            &account_ids[0],
            tombstone_block_index as u64 + 1,
            conn,
        )
        .unwrap();

        let updated_transaction_logs = transaction_logs
            .iter()
            .map(|t| TransactionLog::get(&TransactionId::from(t), conn).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(updated_transaction_logs[0].status(), TxStatus::Failed);
        for tx_log in &updated_transaction_logs[1..] {
            assert_eq!(tx_log.status(), TxStatus::Pending);
        }
    }
}
