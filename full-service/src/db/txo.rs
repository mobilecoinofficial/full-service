// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Txo model.

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use mc_account_keys::{AccountKey, PublicAddress};
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    constants::MAX_INPUTS,
    ring_signature::KeyImage,
    tx::{TxOut, TxOutConfirmationNumber},
};
use std::fmt;

use crate::{
    db::{
        account::{AccountID, AccountModel},
        account_txo_status::AccountTxoStatusModel,
        assigned_subaddress::AssignedSubaddressModel,
        models::{
            Account, AccountTxoStatus, AssignedSubaddress, NewAccountTxoStatus, NewTxo, Txo,
            TXO_STATUS_ORPHANED, TXO_STATUS_PENDING, TXO_STATUS_SECRETED, TXO_STATUS_SPENT,
            TXO_STATUS_UNSPENT, TXO_TYPE_MINTED, TXO_TYPE_RECEIVED, TXO_USED_AS_CHANGE,
            TXO_USED_AS_OUTPUT,
        },
        WalletDbError,
    },
    util::b58::b58_encode_public_address,
};

/// A unique ID derived from a TxOut in the ledger.
#[derive(Debug)]
pub struct TxoID(pub String);

impl From<&TxOut> for TxoID {
    fn from(src: &TxOut) -> TxoID {
        let digest: [u8; 32] = src.digest32::<MerlinTranscript>(b"txo_data");
        Self(hex::encode(digest))
    }
}

impl fmt::Display for TxoID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct TxoDetails {
    // The TXO whose details we are storing
    pub txo: Txo,
    // The Account and Status at which this Txo was Received
    pub received_to_account: Option<AccountTxoStatus>,
    // The subaddress at which this TXO was received
    pub received_to_assigned_subaddress: Option<AssignedSubaddress>,
    // The Account and Status from which this TXO was minted
    pub minted_from_account: Option<AccountTxoStatus>,
}

#[derive(Debug, Clone)]
pub struct ProcessedTxProposalOutput {
    /// The recipient of this TxOut - None if change
    pub recipient: Option<PublicAddress>,
    pub txo_id_hex: String,
    pub value: i64,
    pub txo_type: String,
}

pub trait TxoModel {
    /// Upserts a received Txo.
    ///
    /// # Arguments
    /// * `txo` - a TxOut contained in the ledger.
    /// * `subaddress_index` - The receiving subaddress index, if known.
    /// * `key_image` -
    /// * `value` - The value of the output, in picoMob.
    /// * `received_block_index` - the block at which the Txo was received.
    /// * `account_id_hex` - the account ID for the account which received this
    ///   Txo.
    /// * `conn` - Sqlite database connection.
    ///
    /// The subaddress_index may be None, and the Txo is said to be "orphaned",
    /// if the subaddress is not yet being tracked by the wallet.
    ///
    /// # Returns
    /// * txo_id_hex
    fn create_received(
        tx_out: TxOut,
        subaddress_index: Option<i64>,
        key_image: Option<KeyImage>,
        value: u64,
        received_block_index: i64,
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;

    /// Processes a TxProposal to create a new minted Txo and a change Txo.
    ///
    /// Returns:
    /// * ProcessedTxProposalOutput
    fn create_minted(
        account_id_hex: &str,
        txo: &TxOut,
        tx_proposal: &TxProposal,
        outlay_index: usize,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ProcessedTxProposalOutput, WalletDbError>;

    /// Update an existing Txo to spendable by including its subaddress_index
    /// and key_image.
    fn update_to_spendable(
        &self,
        received_subaddress_index: Option<i64>,
        received_key_image: Option<KeyImage>,
        block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Update a Txo's received block count.
    fn update_received_block_index(
        &self,
        block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Update a Txo's status to pending
    fn update_to_pending(
        txo_id_hex: &TxoID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Get all Txos associated with a given account.
    fn list_for_account(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TxoDetails>, WalletDbError>;

    fn list_for_address(
        assigned_subaddress_b58: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TxoDetails>, WalletDbError>;

    /// Get a Vec<Txo> for all txos in a given account with a given txo_status.
    fn list_by_status(
        account_id_hex: &str,
        status: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get a Vec<Txo> for all txos in a given account with a given txo_type.
    fn list_by_type(
        account_id_hex: &str,
        txo_type: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get the details for a specific Txo.
    ///
    /// Returns:
    /// * TxoDetails
    fn get(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TxoDetails, WalletDbError>;

    /// Get several Txos by Txo public_keys, specific to an account.
    ///
    /// Returns:
    /// * Vec<Txo>
    fn select_by_public_key(
        account_id: &AccountID,
        public_keys: &[&CompressedRistrettoPublic],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError>;

    /// Select several Txos by their TxoIds
    ///
    /// Returns:
    /// * Vec<(Txo, TxoStatus)>
    fn select_by_id(
        txo_ids: &[String],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError>;

    /// Check whether all of the given Txos are spent.
    fn are_all_spent(
        txo_ids: &[String],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    /// Check whether any of the given Txos failed.
    fn any_failed(
        txo_ids: &[String],
        block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    /// Select a set of unspent Txos to reach a given value.
    ///
    /// Returns:
    /// * Vec<Txo>
    fn select_unspent_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        max_spendable_value: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Validate a confirmation number for a Txo
    ///
    /// Returns:
    /// * Bool - true if verified
    fn validate_confirmation(
        account_id: &AccountID,
        txo_id_hex: &str,
        confirmation: &TxOutConfirmationNumber,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    /// Delete txos which are not referenced by any account or transaction.
    fn delete_unreferenced(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl TxoModel for Txo {
    fn create_received(
        txo: TxOut,
        subaddress_index: Option<i64>,
        key_image: Option<KeyImage>,
        value: u64,
        received_block_index: i64,
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), &conn)?;

        let txo_id = TxoID::from(&txo);
        match Txo::get(&txo_id.to_string(), conn) {
            // If we already have this TXO for this account (e.g. from minting in a previous
            // transaction), we need to update it
            Ok(txo_details) => {
                // Check if this txo/pairing already exists for this account.
                match AccountTxoStatus::get(account_id_hex, &txo_id.to_string(), conn) {
                    // The txo/pairing exists for this account in this wallet.
                    Ok(account_txo_status) => {
                        match account_txo_status.txo_status.as_str() {
                            TXO_STATUS_SECRETED => {
                                match account_txo_status.txo_type.as_str() {
                                    // We minted this TXO and sent it to ourselves. It's
                                    // either change that we're now recovering as unspent,
                                    // or it's a new Txo that we sent to ourselves.
                                    TXO_TYPE_MINTED => {
                                        if subaddress_index.is_some() {
                                            // Transition from [Minted, Secreted] ->
                                            // [Minted,
                                            // Unspent]
                                            // This occurs when an account receives a
                                            // transaction from itself at a subaddress.
                                            txo_details.txo.update_to_spendable(
                                                subaddress_index,
                                                key_image,
                                                received_block_index,
                                                &conn,
                                            )?;
                                            account_txo_status.set_unspent(conn)?;
                                        } else {
                                            // Transition from [Minted, Secreted] ->
                                            // [Minted,
                                            // Orphaned]
                                            // This occurs when an account receives a
                                            // transaction from itself at an unknown
                                            // subaddress.
                                            txo_details.txo.update_received_block_index(
                                                received_block_index,
                                                conn,
                                            )?;
                                            account_txo_status.set_orphaned(conn)?;
                                        }
                                    }
                                    // Should not get [Received, Secreted]
                                    _ => {
                                        return Err(WalletDbError::UnexpectedAccountTxoStatus(
                                            account_txo_status.txo_status,
                                        ));
                                    }
                                }
                            }
                            TXO_STATUS_ORPHANED => {
                                // If we have a subaddress for this account and this Txo, we
                                // can update to spendable. True for [Minted, Orphaned] and
                                // [Received, Orphaned]
                                if subaddress_index.is_some() {
                                    txo_details.txo.update_to_spendable(
                                        subaddress_index,
                                        key_image,
                                        received_block_index,
                                        &conn,
                                    )?;
                                    account_txo_status.set_unspent(conn)?;
                                }
                            }
                            TXO_STATUS_UNSPENT => {}
                            TXO_STATUS_PENDING => {}
                            TXO_STATUS_SPENT => {}
                            _ => {
                                return Err(WalletDbError::UnexpectedAccountTxoStatus(
                                    account_txo_status.txo_status,
                                ));
                            }
                        }
                    }
                    // The txo/pairing exists for another account currently in the wallet.
                    // We also want to set it as unspent, but we need to create a new
                    // AccountTxoStatus entry.
                    Err(WalletDbError::AccountTxoStatusNotFound(_)) => {
                        let status = if subaddress_index.is_some() {
                            // If the Txo was already in the DB, but not for this account,
                            // we need to update to spendable with the subaddress and
                            // key_image
                            txo_details.txo.update_to_spendable(
                                subaddress_index,
                                key_image,
                                received_block_index,
                                &conn,
                            )?;
                            TXO_STATUS_UNSPENT
                        } else {
                            // Note: An orphaned Txo cannot be spent until the subaddress is
                            // recovered.
                            txo_details
                                .txo
                                .update_received_block_index(received_block_index, conn)?;
                            TXO_STATUS_ORPHANED
                        };
                        AccountTxoStatus::create(
                            account_id_hex,
                            &txo_id.to_string(),
                            status,
                            TXO_TYPE_RECEIVED,
                            conn,
                        )?;
                    }
                    Err(e) => return Err(e),
                }
            }

            // If we don't already have this TXO, create a new entry
            Err(WalletDbError::TxoNotFound(_)) => {
                let key_image_bytes = key_image.map(|k| mc_util_serial::encode(&k));
                let new_txo = NewTxo {
                    txo_id_hex: &txo_id.to_string(),
                    value: value as i64,
                    target_key: &mc_util_serial::encode(&txo.target_key),
                    public_key: &mc_util_serial::encode(&txo.public_key),
                    e_fog_hint: &mc_util_serial::encode(&txo.e_fog_hint),
                    txo: &mc_util_serial::encode(&txo),
                    subaddress_index,
                    key_image: key_image_bytes.as_deref(),
                    received_block_index: Some(received_block_index as i64),
                    pending_tombstone_block_index: None,
                    spent_block_index: None,
                    confirmation: None,
                    recipient_public_address_b58: "".to_string(), // NULL for received
                };

                diesel::insert_into(crate::db::schema::txos::table)
                    .values(&new_txo)
                    .execute(conn)?;

                let status = if subaddress_index.is_some() {
                    TXO_STATUS_UNSPENT
                } else {
                    // Note: An orphaned Txo cannot be spent until the subaddress is recovered.
                    TXO_STATUS_ORPHANED
                };

                // We should get a unique violation if this AccountTxoStatus already exists.
                AccountTxoStatus::create(
                    account_id_hex,
                    &txo_id.to_string(),
                    status,
                    TXO_TYPE_RECEIVED,
                    conn,
                )?;
            }
            Err(e) => {
                return Err(e);
            }
        };
        Ok(txo_id.to_string())
    }

    fn create_minted(
        account_id_hex: &str,
        output: &TxOut,
        tx_proposal: &TxProposal,
        output_index: usize,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ProcessedTxProposalOutput, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let txo_id = TxoID::from(output);

        let total_input_value: u64 = tx_proposal.utxos.iter().map(|u| u.value).sum();
        let total_output_value: u64 = tx_proposal.outlays.iter().map(|o| o.value).sum();
        let change_value: u64 = total_input_value - total_output_value - tx_proposal.fee();

        // Determine whether this output is an outlay destination, or change.
        let (value, confirmation, outlay_receiver) = if let Some(outlay_index) = tx_proposal
            .outlay_index_to_tx_out_index
            .iter()
            .find_map(|(k, &v)| if v == output_index { Some(k) } else { None })
        {
            let outlay = &tx_proposal.outlays[*outlay_index];
            (
                outlay.value,
                Some(*outlay_index),
                Some(outlay.receiver.clone()),
            )
        } else {
            // This is the change output. Note: there should only be one change output
            // per transaction, based on how we construct transactions. If we change
            // how we construct transactions, these assumptions will change, and should be
            // reflected in the TxProposal.
            (change_value, None, None)
        };

        // Update receiver, transaction_value, and transaction_txo_type, if outlay was
        // found.
        let (transaction_txo_type, log_value, recipient_public_address_b58) =
            if let Some(r) = outlay_receiver.clone() {
                (
                    TXO_USED_AS_OUTPUT,
                    total_output_value,
                    b58_encode_public_address(&r)?,
                )
            } else {
                // If not in an outlay, this output is change, according to how we build
                // transactions.
                (TXO_USED_AS_CHANGE, change_value, "".to_string())
            };

        let encoded_confirmation = confirmation
            .map(|p| mc_util_serial::encode(&tx_proposal.outlay_confirmation_numbers[p]));

        let new_txo = NewTxo {
            txo_id_hex: &txo_id.to_string(),
            value: value as i64,
            target_key: &mc_util_serial::encode(&output.target_key),
            public_key: &mc_util_serial::encode(&output.public_key),
            e_fog_hint: &mc_util_serial::encode(&output.e_fog_hint),
            txo: &mc_util_serial::encode(output),
            subaddress_index: None,
            /* Minted set subaddress_index to None. If later
             * received, updates. */
            key_image: None, // Only the recipient can calculate the KeyImage
            received_block_index: None,
            pending_tombstone_block_index: Some(tx_proposal.tx.prefix.tombstone_block as i64),
            spent_block_index: None,
            confirmation: encoded_confirmation.as_deref(),
            recipient_public_address_b58,
        };

        diesel::insert_into(txos::table)
            .values(&new_txo)
            .execute(conn)?;

        // Log a relationship between the account and the TXO.
        let new_account_txo_status = NewAccountTxoStatus {
            account_id_hex: &account_id_hex,
            txo_id_hex: &txo_id.to_string(),
            // The lifecycle of this txo starts at secreted. If it is change, then it will
            // become unspent when it is received.
            txo_status: TXO_STATUS_SECRETED,
            txo_type: TXO_TYPE_MINTED,
        };
        diesel::insert_into(account_txo_statuses::table)
            .values(&new_account_txo_status)
            .execute(conn)?;

        Ok(ProcessedTxProposalOutput {
            recipient: outlay_receiver,
            txo_id_hex: txo_id.to_string(),
            value: log_value as i64,
            txo_type: transaction_txo_type.to_string(),
        })
    }

    fn update_to_spendable(
        &self,
        received_subaddress_index: Option<i64>,
        received_key_image: Option<KeyImage>,
        block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos::{key_image, received_block_index, subaddress_index};

        // Verify that we have a subaddress, otherwise this transaction will be
        // unspendable.
        if received_subaddress_index.is_none() || received_key_image.is_none() {
            return Err(WalletDbError::NullSubaddressOnReceived);
        }

        let encoded_key_image = received_key_image.map(|k| mc_util_serial::encode(&k));

        diesel::update(self)
            .set((
                received_block_index.eq(Some(block_index)),
                subaddress_index.eq(received_subaddress_index),
                key_image.eq(encoded_key_image),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn update_received_block_index(
        &self,
        block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos::received_block_index;

        diesel::update(self)
            .set((received_block_index.eq(Some(block_index)),))
            .execute(conn)?;
        Ok(())
    }

    fn update_to_pending(
        txo_id: &TxoID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::account_txo_statuses::dsl::account_txo_statuses;

        let result = {
            // Find the account associated with this Txo.
            // Note: We should only be calling update_to_pending on inputs, which we had to
            // own to spend.
            // However, if we sent a tranaction from an account we own to a different
            // account in the same wallet, and then removed the first account,
            // then this could fail.
            let accounts = Account::get_by_txo_id(&txo_id.to_string(), conn)?;

            // Update the status to pending.
            if accounts.len() > 2 {
                return Err(WalletDbError::UnexpectedNumberOfAccountsAssociatedWithTxo(
                    accounts.len().to_string(),
                ));
            }

            // Update the status to pending. Only unspent can go to pending.
            for account in accounts {
                let status =
                    AccountTxoStatus::get(&account.account_id_hex, &txo_id.to_string(), conn)?;
                if status.txo_status == TXO_STATUS_UNSPENT {
                    diesel::update(
                        account_txo_statuses.find((&account.account_id_hex, &txo_id.to_string())),
                    )
                    .set(
                        crate::db::schema::account_txo_statuses::txo_status
                            .eq(TXO_STATUS_PENDING.to_string()),
                    )
                    .execute(conn)?;
                }
            }
            Ok(())
        };

        match result {
            Ok(()) => Ok(()),
            // If the account was not found, there is nothing to update, which is fine.
            Err(WalletDbError::AccountNotFound(_)) => Ok(()),
            // Let other errors propagate.
            Err(e) => Err(e),
        }
    }

    fn list_for_account(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TxoDetails>, WalletDbError> {
        use crate::db::schema::{
            account_txo_statuses as cols, account_txo_statuses::dsl::account_txo_statuses,
        };

        let statuses_query = account_txo_statuses
            .filter(cols::account_id_hex.eq(account_id_hex))
            .select(cols::txo_id_hex);

        // Optionally add limit and offset.
        let statuses: Vec<String> = if let (Some(o), Some(l)) = (offset, limit) {
            statuses_query.offset(o).limit(l).load(conn)?
        } else {
            statuses_query.load(conn)?
        };

        // TODO: This loop queries the txo and account_txo_statuses table N times. Use a
        // join instead.
        let details: Result<Vec<TxoDetails>, WalletDbError> =
            statuses.iter().map(|t| Txo::get(t, &conn)).collect();
        details
    }

    fn list_for_address(
        assigned_subaddress_b58: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TxoDetails>, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};
        let subaddress = AssignedSubaddress::get(&assigned_subaddress_b58, conn)?;

        // Note: The Same Txo may be referenced in the account_txo_statuses multiple
        // times due to its relationship with multiple accounts or serving
        // multiple roles within payments (for example, change - minted,
        // received, unspent, spent).
        let results: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table
                    .on(txos::txo_id_hex.eq(account_txo_statuses::txo_id_hex)),
            )
            .select(txos::all_columns)
            .filter(account_txo_statuses::account_id_hex.eq(subaddress.account_id_hex))
            .filter(txos::subaddress_index.eq(subaddress.subaddress_index))
            .distinct()
            .load(conn)?;

        let details: Result<Vec<TxoDetails>, WalletDbError> = results
            .iter()
            .map(|t| Txo::get(&t.txo_id_hex, &conn))
            .collect();
        details
    }

    fn list_by_status(
        account_id_hex: &str,
        status: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let results: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq(status))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        Ok(results)
    }

    fn list_by_type(
        account_id_hex: &str,
        txo_type: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let results: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_type.eq(txo_type))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        Ok(results)
    }

    fn get(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TxoDetails, WalletDbError> {
        use crate::db::schema::txos::dsl::{txo_id_hex as dsl_txo_id_hex, txos};

        let txo: Txo = match txos
            .filter(dsl_txo_id_hex.eq(txo_id_hex))
            .get_result::<Txo>(conn)
        {
            Ok(t) => t,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::TxoNotFound(txo_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        // Get all the accounts associated with this Txo
        let account_txo_statuses =
            AccountTxoStatus::get_all_associated_accounts(txo_id_hex, &conn)?;

        if account_txo_statuses.len() > 2 {
            return Err(WalletDbError::TxoAssociatedWithTooManyAccounts(format!(
                "({}: {:?})",
                txo_id_hex, account_txo_statuses
            )));
        }
        let mut txo_details = TxoDetails {
            txo: txo.clone(),
            received_to_account: None,
            received_to_assigned_subaddress: None,
            minted_from_account: None,
        };

        for account_txo_status in account_txo_statuses {
            match account_txo_status.txo_type.as_str() {
                TXO_TYPE_MINTED => {
                    txo_details.minted_from_account = Some(account_txo_status.clone());
                    // Note: Minted & Unspent/Pending/Spent means that this Txo was also
                    // received, and is either change, or a Txo that we sent to ourselves.
                    match account_txo_status.txo_status.as_str() {
                        TXO_STATUS_SECRETED => {}
                        // TXO_UNSPENT, TXO_PENDING, TXO_SPENT, TXO_ORPHANED
                        _ => txo_details.received_to_account = Some(account_txo_status.clone()),
                    }
                }
                TXO_TYPE_RECEIVED => {
                    txo_details.received_to_account = Some(account_txo_status.clone());
                }
                _ => {
                    return Err(WalletDbError::UnexpectedTransactionTxoType(
                        account_txo_status.txo_type,
                    ));
                }
            }

            // Get the subaddress details if assigned
            let assigned_subaddress: Option<AssignedSubaddress> =
                if let Some(subaddress_index) = txo.subaddress_index {
                    if let Ok(account) =
                        Account::get(&AccountID(account_txo_status.account_id_hex), conn)
                    {
                        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
                        let subaddress = account_key.subaddress(subaddress_index as u64);
                        let subaddress_b58 = b58_encode_public_address(&subaddress)?;
                        match AssignedSubaddress::get(&subaddress_b58, conn) {
                            Ok(a) => Some(a),
                            Err(WalletDbError::AssignedSubaddressNotFound(_s)) => None,
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
            txo_details.received_to_assigned_subaddress = assigned_subaddress;
        }

        Ok(txo_details)
    }

    fn select_by_public_key(
        account_id: &AccountID,
        public_keys: &[&CompressedRistrettoPublic],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let public_key_blobs: Vec<Vec<u8>> = public_keys
            .iter()
            .map(|p| mc_util_serial::encode(*p))
            .collect();
        let selected: Vec<(Txo, AccountTxoStatus)> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id.to_string()))
                    .and(txos::public_key.eq_any(public_key_blobs))),
            )
            .select((txos::all_columns, account_txo_statuses::all_columns))
            .load(conn)?;
        Ok(selected)
    }

    fn select_by_id(
        txo_ids: &[String],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let txos: Vec<(Txo, AccountTxoStatus)> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(txos::txo_id_hex.eq_any(txo_ids))),
            )
            .select((txos::all_columns, account_txo_statuses::all_columns))
            .load(conn)?;
        Ok(txos)
    }

    fn are_all_spent(
        txo_ids: &[String],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let txos: i64 = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(txos::txo_id_hex.eq_any(txo_ids))
                    .and(account_txo_statuses::txo_status.eq(TXO_STATUS_SPENT))),
            )
            .select(diesel::dsl::count(txos::txo_id_hex))
            .first(conn)?;

        Ok(txos == txo_ids.len() as i64)
    }

    fn any_failed(
        txo_ids: &[String],
        block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let txos: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(txos::txo_id_hex.eq_any(txo_ids))
                    .and(
                        account_txo_statuses::txo_status
                            .eq_any(vec![TXO_STATUS_UNSPENT, TXO_STATUS_PENDING]),
                    )
                    .and(txos::pending_tombstone_block_index.lt(Some(block_index)))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        // Report true if any txos have expired
        Ok(!txos.is_empty())
    }

    fn select_unspent_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        max_spendable_value: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{account_txo_statuses, txos};

        let mut spendable_txos: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq(TXO_STATUS_UNSPENT))
                    .and(txos::subaddress_index.is_not_null())
                    .and(txos::key_image.is_not_null()) // Could technically recreate with subaddress
                    .and(txos::value.le(max_spendable_value.unwrap_or(i64::MAX)))),
            )
            .select(txos::all_columns)
            .order_by(txos::value.desc())
            .load(conn)?;

        if spendable_txos.is_empty() {
            return Err(WalletDbError::NoSpendableTxos);
        }

        // The maximum spendable is limited by the maximal number of inputs we can use.
        // Since the txos are sorted by decreasing value, this is the maximum
        // value we can possibly spend in one transaction.
        // Note, u128::Max = 340_282_366_920_938_463_463_374_607_431_768_211_455, which
        // is far beyond the total number of pMOB in the MobileCoin system
        // (250_000_000_000_000_000_000)
        let max_spendable_in_wallet: u128 = spendable_txos
            .iter()
            .take(MAX_INPUTS as usize)
            .map(|utxo| (utxo.value as u64) as u128)
            .sum();
        // If we're trying to spend more than we have in the wallet, we may need to
        // defrag
        if target_value as u128 > max_spendable_in_wallet {
            // See if we merged the UTXOs we would be able to spend this amount.
            let total_unspent_value_in_wallet: u128 = spendable_txos
                .iter()
                .map(|utxo| (utxo.value as u64) as u128)
                .sum();
            if total_unspent_value_in_wallet >= target_value as u128 {
                return Err(WalletDbError::InsufficientFundsFragmentedTxos);
            } else {
                return Err(WalletDbError::InsufficientFundsUnderMaxSpendable(format!(
                    "Max spendable value in wallet: {:?}, but target value: {:?}",
                    max_spendable_in_wallet, target_value
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
        let mut total: u64 = 0;
        loop {
            if total >= target_value {
                break;
            }

            // Grab the next (smallest) utxo, in order to opportunistically sweep up dust
            let next_utxo = spendable_txos.pop().ok_or_else(|| {
                WalletDbError::InsufficientFunds(format!(
                    "Not enough Txos to sum to target value: {:?}",
                    target_value
                ))
            })?;
            selected_utxos.push(next_utxo.clone());
            total += next_utxo.value as u64;

            // Cap at maximum allowed inputs.
            if selected_utxos.len() > MAX_INPUTS as usize {
                // Remove the lowest utxo.
                let removed = selected_utxos.remove(0);
                total -= removed.value as u64;
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
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        let txo_details = Txo::get(txo_id_hex, conn)?;
        let public_key: RistrettoPublic = mc_util_serial::decode(&txo_details.txo.public_key)?;
        let account = Account::get(account_id, conn)?;
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        Ok(confirmation.validate(&public_key, account_key.view_private_key()))
    }

    fn delete_unreferenced(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{account_txo_statuses, transaction_txo_types, txos};
        // DELETE FROM txos
        //     WHERE txo_id_hex IN (
        //         SELECT
        //             txos.txo_id_hex
        //         FROM txos
        //             LEFT JOIN account_txo_statuses ats
        //                 ON txos.txo_id_hex = ats.txo_id_hex
        //             LEFT JOIN transaction_txo_types ttt
        //                 ON txos.txo_id_hex = ttt.txo_id_hex
        //         WHERE
        //             ats.txo_id_hex IS NULL
        //             AND ttt.txo_id_hex IS NULL
        //     );
        diesel::delete(
            txos::table.filter(
                txos::txo_id_hex.eq_any(
                    txos::table
                        .left_join(
                            account_txo_statuses::table
                                .on(txos::txo_id_hex.eq(account_txo_statuses::txo_id_hex)),
                        )
                        .left_join(
                            transaction_txo_types::table
                                .on(txos::txo_id_hex.eq(transaction_txo_types::txo_id_hex)),
                        )
                        .filter(account_txo_statuses::txo_id_hex.is_null())
                        .filter(transaction_txo_types::txo_id_hex.is_null())
                        .select(txos::txo_id_hex),
                ),
            ),
        )
        .execute(conn)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{AccountKey, RootIdentity};
    use mc_common::{
        logger::{log, test_with_logger, Logger},
        HashSet,
    };
    use mc_crypto_rand::RngCore;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::constants::MINIMUM_FEE;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{iter::FromIterator, time::Duration};

    use crate::{
        db::{
            account::{AccountID, AccountModel, DEFAULT_CHANGE_SUBADDRESS_INDEX},
            models::{Account, TransactionLog},
            transaction_log::TransactionLogModel,
        },
        service::{
            sync::{sync_account, SyncThread},
            transaction_builder::WalletTransactionBuilder,
        },
        test_utils::{
            add_block_with_db_txos, add_block_with_tx_outs, add_block_with_tx_proposal,
            create_test_minted_and_change_txos, create_test_received_txo,
            create_test_txo_for_recipient, get_resolver_factory, get_test_ledger,
            manually_sync_account, random_account_with_seed_values, wait_for_sync,
            WalletDbTestContext, MOB,
        },
        WalletDb,
    };

    use super::*;

    // The narrative for this test is that Alice receives a Txo, then sends a
    // transaction to Bob. We verify expected qualities of the Txos involved at
    // each step of the lifecycle.
    // Note: This is not a replacement for a service-level test, but instead tests
    // basic assumptions after common DB operations with the Txo.
    #[test_with_logger]
    fn test_received_txo_lifecycle(logger: Logger) {
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Create TXO for Alice
        let (for_alice_txo, for_alice_key_image) =
            create_test_txo_for_recipient(&alice_account_key, 0, 1000 * MOB as u64, &mut rng);

        // Let's add this txo to the ledger
        add_block_with_tx_outs(
            &mut ledger_db,
            &[for_alice_txo.clone()],
            &[KeyImage::from(rng.next_u64())],
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, 13, &logger);

        let txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        // Verify that the Txo is what we expect
        let expected_txo = Txo {
            id: 1,
            txo_id_hex: TxoID::from(&for_alice_txo).to_string(),
            value: 1000 * MOB,
            target_key: mc_util_serial::encode(&for_alice_txo.target_key),
            public_key: mc_util_serial::encode(&for_alice_txo.public_key),
            e_fog_hint: mc_util_serial::encode(&for_alice_txo.e_fog_hint),
            txo: mc_util_serial::encode(&for_alice_txo),
            subaddress_index: Some(0),
            key_image: Some(mc_util_serial::encode(&for_alice_key_image)),
            received_block_index: Some(12),
            pending_tombstone_block_index: None,
            spent_block_index: None,
            confirmation: None,
            recipient_public_address_b58: "".to_string(),
        };
        // Verify that the statuses table was updated correctly
        let expected_txo_status = AccountTxoStatus {
            account_id_hex: alice_account_id.to_string(),
            txo_id_hex: TxoID::from(&for_alice_txo).to_string(),
            txo_status: TXO_STATUS_UNSPENT.to_string(),
            txo_type: TXO_TYPE_RECEIVED.to_string(),
        };
        assert_eq!(txos[0].txo, expected_txo);
        assert_eq!(
            txos[0].received_to_account.clone().unwrap(),
            expected_txo_status
        );

        // Verify that the status filter works as well
        let unspent = Txo::list_by_status(
            &alice_account_id.to_string(),
            TXO_STATUS_UNSPENT,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent.len(), 1);

        // Now we'll "spend" the TXO by sending it to ourselves, but at a subaddress we
        // have not yet assigned. At the DB layer, we accomplish this by
        // constructing the output txos, then logging sent and received for this
        // account.
        let ((output_txo_id, output_value), (change_txo_id, change_value)) =
            create_test_minted_and_change_txos(
                alice_account_key.clone(),
                alice_account_key.subaddress(4),
                33 * MOB as u64,
                wallet_db.clone(),
                ledger_db.clone(),
                logger.clone(),
            );
        assert_eq!(output_value, 33 * MOB);
        assert_eq!(change_value, 967 * MOB - MINIMUM_FEE as i64);

        add_block_with_db_txos(
            &mut ledger_db,
            &wallet_db,
            &[output_txo_id, change_txo_id],
            &[KeyImage::from(for_alice_key_image)],
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Now we'll process these Txos and verify that the TXO was "spent."
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, 14, &logger);

        // We should now have 3 txos for this account - one spent, one change (minted),
        // and one minted (destined for alice).
        let txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 3);

        // Check that we have 2 spendable (1 is orphaned)
        let spendable: Vec<&TxoDetails> =
            txos.iter().filter(|f| f.txo.key_image.is_some()).collect();
        assert_eq!(spendable.len(), 2);

        // Check that we have one spent - went from [Received, Unspent] -> [Received,
        // Spent]
        let spent: Vec<&TxoDetails> = txos
            .iter()
            .filter_map(|f| {
                f.received_to_account.clone().map(|a| {
                    if a.txo_status == TXO_STATUS_SPENT {
                        Some(f)
                    } else {
                        None
                    }
                })
            })
            .filter_map(|t| t)
            .collect();
        assert_eq!(spent.len(), 1);
        assert_eq!(
            spent[0].txo.key_image,
            Some(mc_util_serial::encode(&for_alice_key_image))
        );
        assert_eq!(spent[0].txo.spent_block_index.clone().unwrap(), 13);
        assert_eq!(spent[0].minted_from_account, None);
        // The spent Txo was not secreted from any account in this wallet - it should be
        // received to the account in this wallet.
        assert!(spent[0].minted_from_account.clone().is_none(),);
        assert_eq!(
            spent[0].received_to_account.clone().unwrap().txo_type,
            TXO_TYPE_RECEIVED
        );
        assert_eq!(
            spent[0].received_to_account.clone().unwrap().txo_status,
            TXO_STATUS_SPENT
        );

        // Check that we have one orphaned - went from [Minted, Secreted] -> [Minted,
        // Orphaned]
        let orphaned: Vec<&TxoDetails> = txos
            .iter()
            .filter_map(|f| {
                f.minted_from_account.clone().map(|a| {
                    if a.txo_status == TXO_STATUS_ORPHANED {
                        Some(f)
                    } else {
                        None
                    }
                })
            })
            .filter_map(|t| t)
            .collect();
        assert_eq!(orphaned.len(), 1);
        assert!(orphaned[0].txo.key_image.is_none());
        assert_eq!(orphaned[0].txo.received_block_index.clone().unwrap(), 13);
        assert!(orphaned[0].minted_from_account.is_some());
        assert!(orphaned[0].received_to_account.is_some());

        // Check that we have one unspent (change) - went from [Minted, Secreted] ->
        // [Minted, Unspent]
        let unspent: Vec<&TxoDetails> = txos
            .iter()
            .filter_map(|f| {
                f.minted_from_account.clone().map(|a| {
                    if a.txo_status == TXO_STATUS_UNSPENT {
                        Some(f)
                    } else {
                        None
                    }
                })
            })
            .filter_map(|t| t)
            .collect();
        assert_eq!(unspent.len(), 1);
        assert_eq!(unspent[0].txo.received_block_index.clone().unwrap(), 13);
        // Store the key image for when we spend this Txo below
        let for_bob_key_image: KeyImage =
            mc_util_serial::decode(&unspent[0].txo.key_image.clone().unwrap()).unwrap();

        // Note: To receive at Subaddress 4, we need to add an assigned subaddress
        // (currently this Txo is be orphaned). We add thrice, because currently
        // assigned subaddress is at 1.
        for _ in 0..3 {
            AssignedSubaddress::create_next_for_account(
                &alice_account_id.to_string(),
                "",
                &ledger_db,
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();
        }

        let alice_account =
            Account::get(&alice_account_id, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(alice_account.next_block_index, 14);
        assert_eq!(alice_account.next_subaddress_index, 5);

        // Scan for alice to pick up the orphaned Txo
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, 14, &logger);

        // Check that a transaction log entry was created for each received TxOut (note:
        // we are not creating submit logs in this test)
        let transaction_logs = TransactionLog::list_all(
            &alice_account_id.to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(transaction_logs.len(), 3);

        // Verify that there are two unspent txos - the one that was previously
        // orphaned, and change.
        let unspent = Txo::list_by_status(
            &alice_account_id.to_string(),
            TXO_STATUS_UNSPENT,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent.len(), 2);

        // The type should still be "minted"
        let minted = Txo::list_by_type(
            &alice_account_id.to_string(),
            TXO_TYPE_MINTED,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(minted.len(), 2);

        let updated_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // There are now 3 total Txos for our account
        assert_eq!(updated_txos.len(), 3);

        // Verify that there is one change Txo in our current Txos
        let change: Vec<&TxoDetails> = updated_txos
            .iter()
            .filter(|f| {
                if let Some(addr) = &f.received_to_assigned_subaddress {
                    addr.subaddress_index == DEFAULT_CHANGE_SUBADDRESS_INDEX as i64
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let ((output_txo_id, output_value), (change_txo_id, change_value)) =
            create_test_minted_and_change_txos(
                alice_account_key.clone(),
                bob_account_key.subaddress(0),
                72 * MOB as u64,
                wallet_db.clone(),
                ledger_db.clone(),
                logger.clone(),
            );
        assert_eq!(output_value, 72 * MOB);
        assert_eq!(change_value, 928 * MOB - (2 * MINIMUM_FEE as i64));

        // Add the minted Txos to the ledger
        add_block_with_db_txos(
            &mut ledger_db,
            &wallet_db,
            &[output_txo_id, change_txo_id],
            &[KeyImage::from(for_bob_key_image)],
        );

        // Process the latest block for Bob (note, Bob is starting to sync from block 0)
        let _bob_account =
            manually_sync_account(&ledger_db, &wallet_db, &bob_account_id, 15, &logger);
        // Process the latest block for Alice
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, 15, &logger);

        // We should now have 1 txo in Bob's account.
        let txos = Txo::list_for_account(
            &AccountID::from(&bob_account_key).to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        let bob_txo = txos[0].clone();
        assert_eq!(bob_txo.txo.subaddress_index.unwrap(), 0);
        assert!(bob_txo.txo.key_image.is_some());
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account
        // [100, 200, 300, ... 2000]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                (100 * MOB * i) as u64, // 100.0 MOB * i
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        // Greedily take smallest to exact value
        let txos_for_value = Txo::select_unspent_txos_for_value(
            &account_id_hex.to_string(),
            300 * MOB as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value));
        assert_eq!(
            result_set,
            HashSet::<i64>::from_iter(vec![100 * MOB, 200 * MOB])
        );

        // Once we include the fee, we need another txo
        let txos_for_value = Txo::select_unspent_txos_for_value(
            &account_id_hex.to_string(),
            300 * MOB as u64 + MINIMUM_FEE,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value));
        assert_eq!(
            result_set,
            HashSet::<i64>::from_iter(vec![100 * MOB, 200 * MOB, 300 * MOB])
        );

        // Setting max spendable value gives us insufficient funds - only allows 100
        let res = Txo::select_unspent_txos_for_value(
            &account_id_hex.to_string(),
            300 * MOB as u64 + MINIMUM_FEE,
            Some(200 * MOB),
            &wallet_db.get_conn().unwrap(),
        );
        match res {
            Err(WalletDbError::InsufficientFundsUnderMaxSpendable(_)) => {}
            Ok(_) => panic!("Should error with InsufficientFundsUnderMaxSpendable"),
            Err(_) => panic!("Should error with InsufficientFundsUnderMaxSpendable"),
        }

        // sum(300..1800) to get a window where we had to increase past the smallest
        // txos, and also fill up all 16 input slots.
        let txos_for_value = Txo::select_unspent_txos_for_value(
            &account_id_hex.to_string(),
            16800 * MOB as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value));
        assert_eq!(
            result_set,
            HashSet::<i64>::from_iter(vec![
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account. Total value is 2000, but max can spend is
        // 1600 [100, 100, ... 100]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                (100 * MOB) as u64,
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        let res = Txo::select_unspent_txos_for_value(
            &account_id_hex.to_string(), // FIXME: WS-11 - take AccountID
            1800 * MOB as u64,
            None,
            &wallet_db.get_conn().unwrap(),
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

    #[test_with_logger]
    fn test_create_minted(logger: Logger) {
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Process the txos in the ledger into the DB
        sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID::from(&src_account).to_string(),
            &logger,
        )
        .unwrap();

        let recipient =
            AccountKey::from(&RootIdentity::from_random(&mut rng)).subaddress(rng.next_u64());

        let ((output_txo_id, output_value), (change_txo_id, change_value)) =
            create_test_minted_and_change_txos(
                src_account.clone(),
                recipient,
                1 * MOB as u64,
                wallet_db.clone(),
                ledger_db,
                logger,
            );

        assert_eq!(output_value, 1 * MOB);
        let minted_txo_details = Txo::get(&output_txo_id, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(minted_txo_details.txo.value, output_value);
        assert_eq!(
            minted_txo_details.minted_from_account.unwrap().txo_status,
            TXO_STATUS_SECRETED
        );
        assert!(minted_txo_details.received_to_assigned_subaddress.is_none());

        assert_eq!(change_value, 4999 * MOB - MINIMUM_FEE as i64);
        let change_txo_details = Txo::get(&change_txo_id, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(change_txo_details.txo.value, change_value);
        assert_eq!(
            change_txo_details.minted_from_account.unwrap().txo_status,
            TXO_STATUS_SECRETED
        );
        assert!(change_txo_details.received_to_assigned_subaddress.is_none()); // Note: This gets updated on sync
    }

    // Test that the confirmation number validates correctly.
    #[test_with_logger]
    fn test_validate_confirmation(logger: Logger) {
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Start sync thread
        log::info!(logger, "Starting sync thread");
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        log::info!(logger, "Creating a random sender account");
        let sender_account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64, 80 * MOB as u64, 90 * MOB as u64],
            &mut rng,
        );
        let sender_account_id = AccountID::from(&sender_account_key);

        // Create TxProposal from the sender account, which contains the Confirmation
        // Number
        log::info!(logger, "Creating transaction builder");
        let mut builder: WalletTransactionBuilder<MockFogPubkeyResolver> =
            WalletTransactionBuilder::new(
                AccountID::from(&sender_account_key).to_string(),
                wallet_db.clone(),
                ledger_db.clone(),
                get_resolver_factory(&mut rng).unwrap(),
                logger.clone(),
            );
        builder
            .add_recipient(recipient_account_key.default_subaddress(), 50 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build().unwrap();

        // Sleep to make sure that the foreign keys exist
        std::thread::sleep(Duration::from_secs(3));

        // Let's log this submitted Tx for the sender, which will create_minted for the
        // sent Txo
        log::info!(logger, "Logging submitted transaction");
        let tx_log = TransactionLog::log_submitted(
            proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &sender_account_id.to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Now we need to let this txo hit the ledger, which will update sender and
        // receiver
        log::info!(logger, "Adding block from submitted");
        add_block_with_tx_proposal(&mut ledger_db, proposal.clone());

        // Now let our sync thread catch up for both sender and receiver
        log::info!(logger, "Manually syncing account");
        wait_for_sync(&ledger_db, &wallet_db, &recipient_account_id, 16);
        wait_for_sync(&ledger_db, &wallet_db, &sender_account_id, 16);

        // Then let's make sure we received the Txo on the recipient account
        log::info!(logger, "Listing all Txos for recipient account");
        let txos = Txo::list_for_account(
            &recipient_account_id.to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        let received_txo = txos[0].clone();

        // Note: Because this txo is both received and sent, between two different
        // accounts, its confirmation number does get updated. Typically, received txos
        // have None for the confirmation number.
        assert!(received_txo.txo.confirmation.is_some());

        // Get the txo from the sent perspective
        log::info!(logger, "Listing all Txos for sender account");
        let sender_txos = Txo::list_for_account(
            &sender_account_id.to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // We seeded with 3 received (70, 80, 90), we have a change txo, and a secreted
        // Txo (50)
        assert_eq!(sender_txos.len(), 5);

        // Get the associated Txos with the transaction log
        log::info!(logger, "Getting associated Txos with the transaction");
        let associated = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();
        let sent_outputs = associated.outputs;
        assert_eq!(sent_outputs.len(), 1);
        let sent_txo_details =
            Txo::get(&sent_outputs[0].txo_id_hex, &wallet_db.get_conn().unwrap()).unwrap();

        // These two txos should actually be the same txo, and the account_txo_status is
        // what differentiates them.
        assert_eq!(sent_txo_details.txo, received_txo.txo);

        assert!(sent_txo_details.txo.confirmation.is_some());
        let confirmation: TxOutConfirmationNumber =
            mc_util_serial::decode(&sent_txo_details.txo.confirmation.unwrap()).unwrap();
        log::info!(logger, "Validating the confirmation number");
        let verified = Txo::validate_confirmation(
            &AccountID::from(&recipient_account_key),
            &received_txo.txo.txo_id_hex,
            &confirmation,
            &wallet_db.get_conn().unwrap(),
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
        let (account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Seed Txos
        let mut src_txos = Vec::new();
        for i in 0..10 {
            let (_txo_id, txo, _key_image) =
                create_test_received_txo(&account_key, i, i * MOB as u64, i, &mut rng, &wallet_db);
            src_txos.push(txo);
        }
        let pubkeys: Vec<&CompressedRistrettoPublic> =
            src_txos.iter().map(|t| &t.public_key).collect();

        let txos_and_status =
            Txo::select_by_public_key(&account_id, &pubkeys, &wallet_db.get_conn().unwrap())
                .expect("Could not get txos by public keys");
        assert_eq!(txos_and_status.len(), 10);

        let txos_and_status =
            Txo::select_by_public_key(&account_id, &pubkeys[0..5], &wallet_db.get_conn().unwrap())
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let account = Account::get(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();

        // Create some txos.
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            0
        );
        for _ in 0..10 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                (100 * MOB) as u64, // 100.0 MOB * i
                (144) as u64,
                &mut rng,
                &wallet_db,
            );
        }
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            10
        );

        // Delete the account. No Txos are left.
        account.delete(&wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            0
        );
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
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        if fragmented {
            let (_txo_id, _txo, _key_image) =
                create_test_received_txo(&account_key, 0, 28922973268924, 15, &mut rng, &wallet_db);

            for i in 1..=15 {
                let (_txo_id, _txo, _key_image) =
                    create_test_received_txo(&account_key, i, 10000000000, i, &mut rng, &wallet_db);
            }

            for i in 1..=20 {
                let (_txo_id, _txo, _key_image) =
                    create_test_received_txo(&account_key, i, 1000000000, i, &mut rng, &wallet_db);
            }

            for i in 1..=500 {
                let (_txo_id, _txo, _key_image) =
                    create_test_received_txo(&account_key, i, 100000000, i, &mut rng, &wallet_db);
            }
        } else {
            for i in 1..=20 {
                let (_txo_id, _txo, _key_image) =
                    create_test_received_txo(&account_key, i, i as u64, i, &mut rng, &wallet_db);
            }
        }

        (account_id, wallet_db)
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_equals_max_spendable_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_unspent_txos_for_value(
            &account_id.to_string(),
            200 as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 16);
        let sum: i64 = result.iter().map(|x| x.value).sum();
        assert_eq!(200 as i64, sum);
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_over_max_spendable_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_unspent_txos_for_value(
            &account_id.to_string(),
            201 as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        );

        assert!(result.is_err());
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_under_max_spendable_in_account_selects_dust(
        logger: Logger,
    ) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_unspent_txos_for_value(
            &account_id.to_string(),
            3 as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        let sum: i64 = result.iter().map(|x| x.value).sum();
        assert_eq!(3 as i64, sum);
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_over_total_mob_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_unspent_txos_for_value(
            &account_id.to_string(),
            500 as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        );
        assert!(result.is_err());
    }

    #[test_with_logger]
    fn test_select_unspent_txos_for_value_selects_correct_subset_of_txos_when_fragmented(
        logger: Logger,
    ) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, true);

        let result = Txo::select_unspent_txos_for_value(
            &account_id.to_string(),
            12400000000 as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 16);
        let sum: i64 = result.iter().map(|x| x.value).sum();
        assert_eq!(12400000000 as i64, sum);
    }

    // FIXME: once we have create_minted, then select_txos test with no
    // FIXME: test update txo after tombstone block is exceeded
    // FIXME: test update txo after it has landed via key_image update
    // FIXME: test any_failed and are_all_spent
    // FIXME: test for selecting utxos from multiple subaddresses in one account
    // FIXME: test for one TXO belonging to multiple accounts with get
    // FIXME: test create_received for various permutations of multiple accounts
}
