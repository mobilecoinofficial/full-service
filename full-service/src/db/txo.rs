// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Txo model.

use diesel::{
    dsl::{count, exists, not},
    prelude::*,
};
use mc_account_keys::{AccountKey, PublicAddress, CHANGE_SUBADDRESS_INDEX};
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    constants::MAX_INPUTS,
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{TxOut, TxOutConfirmationNumber},
    Amount, Token,
};
use std::{fmt, str::FromStr};

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, NewTransactionOutputTxo, NewTxo, Txo},
        transaction_log::{TransactionID, TxoType},
        Conn, WalletDbError,
    },
    util::b58::b58_encode_public_address,
};

#[derive(Debug, PartialEq)]
pub enum TxoStatus {
    // The txo has been received at a known subaddress index, but the key image cannot
    // be derived (usually because this is a view only account)
    Unverified,
    // The txo has been received at a known subaddress index with a known key image, has not been
    // spent, and is not part of a pending transaction
    Unspent,
    // The txo is part of a pending transaction
    Pending,
    // The txo has a known spent block index
    Spent,
    // The txo has been received but the subaddress index and key image cannot be determined. This
    // happens typically when an account is imported but all subaddresses it was using were not
    // recreated
    Orphaned,
}

impl fmt::Display for TxoStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxoStatus::Unverified => write!(f, "unverified"),
            TxoStatus::Unspent => write!(f, "unspent"),
            TxoStatus::Pending => write!(f, "pending"),
            TxoStatus::Spent => write!(f, "spent"),
            TxoStatus::Orphaned => write!(f, "orphaned"),
        }
    }
}

impl FromStr for TxoStatus {
    type Err = WalletDbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unverified" => Ok(TxoStatus::Unverified),
            "unspent" => Ok(TxoStatus::Unspent),
            "pending" => Ok(TxoStatus::Pending),
            "spent" => Ok(TxoStatus::Spent),
            "orphaned" => Ok(TxoStatus::Orphaned),
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

impl fmt::Display for TxoID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedTxProposalOutput {
    /// The recipient of this TxOut - None if change
    pub recipient: Option<PublicAddress>,
    pub txo_id_hex: String,
    pub value: i64,
}

pub struct SpendableTxosResult {
    pub spendable_txos: Vec<Txo>,
    pub max_spendable_in_wallet: u128,
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
        subaddress_index: Option<u64>,
        key_image: Option<KeyImage>,
        amount: Amount,
        received_block_index: u64,
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<String, WalletDbError>;

    /// Processes a TxProposal to create a new minted Txo and a change Txo.
    fn create_minted(
        txo: &TxOut,
        tx_proposal: &TxProposal,
        outlay_index: usize,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    /// Update an existing Txo to spendable by including its subaddress_index
    /// and optionally the key_image in the case of view only accounts.
    fn update_as_received(
        &self,
        account_id_hex: &str,
        subaddress_index: Option<u64>,
        key_image: Option<KeyImage>,
        block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    /// Update a Txo's status to spent
    fn update_spent_block_index(
        txo_id_hex: &str,
        spent_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn update_key_image(
        txo_id_hex: &str,
        key_image: &KeyImage,
        spent_block_index: Option<u64>,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    /// Get all Txos associated with a given account.
    fn list_for_account(
        account_id_hex: &str,
        status: Option<TxoStatus>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    fn list_for_address(
        assigned_subaddress_b58: &str,
        status: Option<TxoStatus>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Get a map from key images to unspent txos for this account.
    fn list_unspent_or_pending_key_images(
        account_id_hex: &str,
        token_id: Option<u64>,
        conn: &Conn,
    ) -> Result<HashMap<KeyImage, String>, WalletDbError>;

    fn list_unspent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    fn list_spent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    fn list_orphaned(
        account_id_hex: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    fn list_pending(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    fn list_unverified(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    fn list_spendable(
        account_id_hex: Option<&str>,
        max_spendable_value: Option<u64>,
        assigned_subaddress_b58: Option<&str>,
        token_id: u64,
        conn: &Conn,
    ) -> Result<SpendableTxosResult, WalletDbError>;

    /// Get the details for a specific Txo.
    ///
    /// Returns:
    /// * Txo
    fn get(txo_id_hex: &str, conn: &Conn) -> Result<Txo, WalletDbError>;

    /// Get several Txos by Txo public_keys
    ///
    /// Returns:
    /// * Vec<Txo>
    fn select_by_public_key(
        public_keys: &[&CompressedRistrettoPublic],
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Select several Txos by their TxoIds
    ///
    /// Returns:
    /// * Vec<(Txo)>
    fn select_by_id(txo_ids: &[String], conn: &Conn) -> Result<Vec<Txo>, WalletDbError>;

    /// Select a set of unspent Txos to reach a given value.
    ///
    /// Returns:
    /// * Vec<Txo>
    fn select_spendable_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        max_spendable_value: Option<u64>,
        token_id: u64,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError>;

    /// Validate a confirmation number for a Txo
    ///
    /// Returns:
    /// * Bool - true if verified
    fn validate_confirmation(
        account_id: &AccountID,
        txo_id_hex: &str,
        confirmation: &TxOutConfirmationNumber,
        conn: &Conn,
    ) -> Result<bool, WalletDbError>;

    fn scrub_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError>;

    /// Delete txos which are not referenced by any account or transaction.
    fn delete_unreferenced(conn: &Conn) -> Result<(), WalletDbError>;

    fn status(&self, conn: &Conn) -> Result<TxoStatus, WalletDbError>;
}

impl TxoModel for Txo {
    fn create_received(
        txo: TxOut,
        subaddress_index: Option<u64>,
        key_image: Option<KeyImage>,
        amount: Amount,
        received_block_index: u64,
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<String, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        let txo_id = TxoID::from(&txo);
        match Txo::get(&txo_id.to_string(), conn) {
            // If we already have this TXO for this account (e.g. from minting in a previous
            // transaction), we need to update it
            Ok(txo) => {
                txo.update_as_received(
                    account_id_hex,
                    subaddress_index,
                    key_image,
                    received_block_index,
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
                    txo: &mc_util_serial::encode(&txo),
                    subaddress_index: subaddress_index.map(|i| i as i64),
                    key_image: key_image_bytes.as_deref(),
                    received_block_index: Some(received_block_index as i64),
                    spent_block_index: None,
                    shared_secret: None,
                    account_id: Some(account_id_hex.to_string()),
                };

                diesel::insert_into(crate::db::schema::txos::table)
                    .values(&new_txo)
                    .execute(conn)?;
            }
            Err(e) => {
                return Err(e);
            }
        };
        Ok(txo_id.to_string())
    }

    fn create_minted(
        output: &TxOut,
        tx_proposal: &TxProposal,
        output_index: usize,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let txo_id = TxoID::from(output);

        let transaction_id: TransactionID = tx_proposal.into();

        let total_input_value: u64 = tx_proposal.utxos.iter().map(|u| u.value).sum();
        let total_output_value: u64 = tx_proposal.outlays.iter().map(|o| o.value).sum();
        let change_value: u64 = total_input_value - total_output_value - tx_proposal.fee();

        // Determine whether this output is an outlay destination, or change.
        let (value, confirmation, outlay_receiver, txo_type) = if let Some(outlay_index) =
            tx_proposal
                .outlay_index_to_tx_out_index
                .iter()
                .find_map(|(k, &v)| if v == output_index { Some(k) } else { None })
        {
            let outlay = &tx_proposal.outlays[*outlay_index];
            (
                outlay.value,
                Some(*outlay_index),
                Some(outlay.receiver.clone()),
                TxoType::Payload,
            )
        } else {
            // This is the change output. Note: there should only be one change
            // output per transaction, based on how we construct
            // transactions. If we change how we construct transactions,
            // these assumptions will change, and should be reflected in the
            // TxProposal.
            (change_value, None, None, TxoType::Change)
        };

        // Update receiver, transaction_value, and transaction_txo_type, if outlay was
        // found.
        let (recipient_public_address_b58, subaddress_index) = if let Some(r) = outlay_receiver {
            (b58_encode_public_address(&r)?, None)
        } else {
            // If not in an outlay, this output is change, according to how we build
            // transactions.
            ("".to_string(), Some(CHANGE_SUBADDRESS_INDEX as i64))
        };

        let encoded_confirmation = confirmation
            .map(|p| mc_util_serial::encode(&tx_proposal.outlay_confirmation_numbers[p]));

        // TODO: Update this to use the txo id of the output we are minting, not
        // defaulting to 0
        let new_txo = NewTxo {
            id: &txo_id.to_string(),
            value: value as i64,
            account_id: None,
            token_id: 0,
            target_key: &mc_util_serial::encode(&output.target_key),
            public_key: &mc_util_serial::encode(&output.public_key),
            e_fog_hint: &mc_util_serial::encode(&output.e_fog_hint),
            txo: &mc_util_serial::encode(output),
            subaddress_index,
            key_image: None, // Only the recipient can calculate the KeyImage
            received_block_index: None,
            spent_block_index: None,
            shared_secret: encoded_confirmation.as_deref(),
        };

        diesel::insert_into(txos::table)
            .values(&new_txo)
            .execute(conn)?;

        let new_transaction_output_txo = NewTransactionOutputTxo {
            transaction_log_id: &transaction_id.to_string(),
            txo_id: &txo_id.to_string(),
            recipient_public_address_b58: &recipient_public_address_b58,
            is_change: txo_type == TxoType::Change,
        };

        diesel::insert_into(crate::db::schema::transaction_output_txos::table)
            .values(&new_transaction_output_txo)
            .execute(conn)?;

        Ok(())
    }

    fn update_as_received(
        &self,
        received_account_id_hex: &str,
        received_subaddress_index: Option<u64>,
        received_key_image: Option<KeyImage>,
        block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let encoded_key_image = received_key_image.map(|k| mc_util_serial::encode(&k));

        diesel::update(self)
            .set((
                txos::account_id.eq(Some(received_account_id_hex)),
                txos::received_block_index.eq(Some(block_index as i64)),
                txos::subaddress_index.eq(received_subaddress_index.map(|i| i as i64)),
                txos::key_image.eq(encoded_key_image),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn update_spent_block_index(
        txo_id_hex: &str,
        spent_block_index: u64,
        conn: &Conn,
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
        conn: &Conn,
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

    fn list_for_account(
        account_id_hex: &str,
        status: Option<TxoStatus>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        if let Some(status) = status {
            match status {
                TxoStatus::Unverified => {
                    return Txo::list_unverified(
                        Some(account_id_hex),
                        None,
                        token_id,
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
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Orphaned => {
                    return Txo::list_orphaned(Some(account_id_hex), token_id, offset, limit, conn)
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

        Ok(query.load(conn)?)
    }

    fn list_for_address(
        assigned_subaddress_b58: &str,
        status: Option<TxoStatus>,
        offset: Option<u64>,
        limit: Option<u64>,
        token_id: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        if let Some(status) = status {
            match status {
                TxoStatus::Unverified => {
                    return Txo::list_unverified(
                        None,
                        Some(assigned_subaddress_b58),
                        token_id,
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
                        offset,
                        limit,
                        conn,
                    )
                }
                TxoStatus::Orphaned => {
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

        let txos: Vec<Txo> = query.load(conn)?;

        Ok(txos)
    }

    fn list_unspent(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
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

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

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

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query.filter(txos::subaddress_index.eq(subaddress.subaddress_index));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        Ok(query.select(txos::all_columns).load(conn)?)
    }

    fn list_unverified(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs, txos};

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_input_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_input_txos::transaction_log_id)),
            );

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

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

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query.filter(txos::subaddress_index.eq(subaddress.subaddress_index));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        Ok(query.load(conn)?)
    }

    fn list_unspent_or_pending_key_images(
        account_id_hex: &str,
        token_id: Option<u64>,
        conn: &Conn,
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

        let results: Vec<(Option<Vec<u8>>, String)> =
            query.select((txos::key_image, txos::id)).load(conn)?;

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
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let mut query = txos::table.into_boxed();

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        query = query.filter(txos::spent_block_index.is_not_null());

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query.filter(txos::subaddress_index.eq(subaddress.subaddress_index));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        Ok(query.load(conn)?)
    }

    fn list_orphaned(
        account_id_hex: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
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

        let txos: Vec<Txo> = query.load(conn)?;

        Ok(txos)
    }

    fn list_pending(
        account_id_hex: Option<&str>,
        assigned_subaddress_b58: Option<&str>,
        token_id: Option<u64>,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
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
            .filter(transaction_logs::finalized_block_index.is_null());

        query = query
            .filter(txos::subaddress_index.is_not_null())
            .filter(txos::spent_block_index.is_null());

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

        if let Some(subaddress_b58) = assigned_subaddress_b58 {
            let subaddress = AssignedSubaddress::get(subaddress_b58, conn)?;
            query = query.filter(txos::subaddress_index.eq(subaddress.subaddress_index));
        }

        if let Some(token_id) = token_id {
            query = query.filter(txos::token_id.eq(token_id as i64));
        }

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        let txos: Vec<Txo> = query.select(txos::all_columns).load(conn)?;

        Ok(txos)
    }

    fn get(txo_id_hex: &str, conn: &Conn) -> Result<Txo, WalletDbError> {
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
        conn: &Conn,
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

    fn select_by_id(txo_ids: &[String], conn: &Conn) -> Result<Vec<Txo>, WalletDbError> {
        use crate::db::schema::txos;

        let txos: Vec<Txo> = txos::table.filter(txos::id.eq_any(txo_ids)).load(conn)?;

        Ok(txos)
    }

    fn list_spendable(
        account_id_hex: Option<&str>,
        max_spendable_value: Option<u64>,
        assigned_subaddress_b58: Option<&str>,
        token_id: u64,
        conn: &Conn,
    ) -> Result<SpendableTxosResult, WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs, txos};

        let mut query = txos::table
            .into_boxed()
            .left_join(transaction_input_txos::table)
            .left_join(
                transaction_logs::table
                    .on(transaction_logs::id.eq(transaction_input_txos::transaction_log_id)),
            );

        if let Some(account_id_hex) = account_id_hex {
            query = query.filter(txos::account_id.eq(account_id_hex));
        }

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
            query = query.filter(txos::subaddress_index.eq(subaddress.subaddress_index));
        }

        if let Some(max_spendable_value) = max_spendable_value {
            query = query.filter(txos::value.le(max_spendable_value as i64));
        }

        let spendable_txos = query
            .select(txos::all_columns)
            .order_by(txos::value.desc())
            .load(conn)?;

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

        if max_spendable_in_wallet > Mob::MINIMUM_FEE as u128 {
            max_spendable_in_wallet -= Mob::MINIMUM_FEE as u128;
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
        target_value: u64,
        max_spendable_value: Option<u64>,
        token_id: u64,
        conn: &Conn,
    ) -> Result<Vec<Txo>, WalletDbError> {
        let SpendableTxosResult {
            mut spendable_txos,
            max_spendable_in_wallet,
        } = Txo::list_spendable(
            Some(account_id_hex),
            max_spendable_value,
            None,
            token_id,
            conn,
        )?;

        if spendable_txos.is_empty() {
            return Err(WalletDbError::NoSpendableTxos);
        }

        // If we're trying to spend more than we have in the wallet, we may need to
        // defrag
        if target_value as u128 > max_spendable_in_wallet + Mob::MINIMUM_FEE as u128 {
            // See if we merged the UTXOs we would be able to spend this amount.
            let total_unspent_value_in_wallet: u128 = spendable_txos
                .iter()
                .map(|utxo| (utxo.value as u64) as u128)
                .sum();

            if total_unspent_value_in_wallet >= (target_value + Mob::MINIMUM_FEE) as u128 {
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
        conn: &Conn,
    ) -> Result<bool, WalletDbError> {
        let txo = Txo::get(txo_id_hex, conn)?;
        let public_key: RistrettoPublic = mc_util_serial::decode(&txo.public_key)?;
        let account = Account::get(account_id, conn)?;
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        Ok(confirmation.validate(&public_key, account_key.view_private_key()))
    }

    fn scrub_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::txos;

        let txos_received_by_account = txos::table.filter(txos::account_id.eq(account_id_hex));

        diesel::update(txos_received_by_account)
            .set(txos::account_id.eq::<Option<String>>(None))
            .execute(conn)?;

        Ok(())
    }

    fn delete_unreferenced(conn: &Conn) -> Result<(), WalletDbError> {
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

    fn status(&self, conn: &Conn) -> Result<TxoStatus, WalletDbError> {
        use crate::db::schema::{
            transaction_input_txos, transaction_logs, transaction_output_txos,
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
            .filter(transaction_logs::tombstone_block_index.is_not_null())
            .filter(transaction_logs::finalized_block_index.is_null())
            .filter(transaction_logs::failed.eq(false))
            .select(count(transaction_logs::id))
            .first(conn)?;

        let pending = num_pending_logs > 0;

        if pending {
            return Ok(TxoStatus::Pending);
        }

        if self.subaddress_index.is_some() && self.key_image.is_some() {
            Ok(TxoStatus::Unspent)
        } else if self.subaddress_index.is_some() {
            Ok(TxoStatus::Unverified)
        } else {
            Ok(TxoStatus::Orphaned)
        }
    }
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{AccountKey, RootIdentity, CHANGE_SUBADDRESS_INDEX};
    use mc_common::{
        logger::{log, test_with_logger, Logger},
        HashSet,
    };
    use mc_crypto_rand::RngCore;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{tokens::Mob, Amount, Token, TokenId};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{iter::FromIterator, time::Duration};

    use crate::{
        db::{
            account::{AccountID, AccountModel},
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
            manually_sync_account, random_account_with_seed_values, WalletDbTestContext, MOB,
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
            "".to_string(),
            "".to_string(),
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
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
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, &logger);

        let txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        // Verify that the Txo is what we expect
        let expected_txo = Txo {
            id: TxoID::from(&for_alice_txo).to_string(),
            value: 1000 * MOB as i64,
            token_id: 0,
            target_key: mc_util_serial::encode(&for_alice_txo.target_key),
            public_key: mc_util_serial::encode(&for_alice_txo.public_key),
            e_fog_hint: mc_util_serial::encode(&for_alice_txo.e_fog_hint),
            txo: mc_util_serial::encode(&for_alice_txo),
            subaddress_index: Some(0),
            key_image: Some(mc_util_serial::encode(&for_alice_key_image)),
            received_block_index: Some(12),
            spent_block_index: None,
            shared_secret: None,
            account_id: Some(alice_account_id.to_string()),
        };

        assert_eq!(expected_txo, txos[0]);

        // Verify that the status filter works as well
        let unspent = Txo::list_unspent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent.len(), 1);

        // Now we'll "spend" the TXO by sending it to ourselves, but at a subaddress we
        // have not yet assigned. At the DB layer, we accomplish this by
        // constructing the output txos, then logging sent and received for this
        // account.
        let transaction_log = create_test_minted_and_change_txos(
            alice_account_key.clone(),
            alice_account_key.subaddress(4),
            33 * MOB,
            wallet_db.clone(),
            ledger_db.clone(),
            logger.clone(),
        );

        let associated_txos = transaction_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();

        assert_eq!(minted_txo.value as u64, 33 * MOB);
        assert_eq!(change_txo.value as u64, 967 * MOB - Mob::MINIMUM_FEE);

        add_block_with_db_txos(
            &mut ledger_db,
            &wallet_db,
            &[minted_txo.id.clone(), change_txo.id.clone()],
            &[KeyImage::from(for_alice_key_image)],
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Now we'll process these Txos and verify that the TXO was "spent."
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, &logger);

        // We should now have 3 txos for this account - one spent, one change (minted),
        // and one minted (destined for alice).
        let txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 3);

        // test spent
        let spent_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            Some(TxoStatus::Spent),
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(spent_txos.len(), 1);

        // test unspent
        let unspent_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            Some(TxoStatus::Unspent),
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
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
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(spent.len(), 1);
        assert_eq!(
            spent[0].key_image,
            Some(mc_util_serial::encode(&for_alice_key_image))
        );
        assert_eq!(spent[0].spent_block_index.clone().unwrap(), 13);

        // Check that we have one orphaned - went from [Minted, Secreted] -> [Minted,
        // Orphaned]
        let orphaned = Txo::list_orphaned(
            Some(&alice_account_id.to_string()),
            Some(0),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(orphaned.len(), 1);
        assert!(orphaned[0].key_image.is_none());
        assert_eq!(orphaned[0].received_block_index.clone().unwrap(), 13);
        assert!(orphaned[0].account_id.is_some());

        // Check that we have one unspent (change) - went from [Minted, Secreted] ->
        // [Minted, Unspent]
        let unspent = Txo::list_unspent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(unspent.len(), 1);
        assert_eq!(unspent[0].received_block_index.clone().unwrap(), 13);
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
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();
        }

        let alice_account =
            Account::get(&alice_account_id, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(alice_account.next_block_index, 14);
        assert_eq!(alice_account.next_subaddress_index, 5);

        // Verify that there are two unspent txos - the one that was previously
        // orphaned, and change.
        let unspent = Txo::list_unspent(
            Some(&alice_account_id.to_string()),
            None,
            Some(0),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        println!("{}", serde_json::to_string_pretty(&unspent).unwrap());
        assert_eq!(unspent.len(), 2);

        let updated_txos = Txo::list_for_account(
            &alice_account_id.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // There are now 3 total Txos for our account
        assert_eq!(updated_txos.len(), 3);

        // Verify that there is one change Txo in our current Txos
        let change: Vec<&Txo> = updated_txos
            .iter()
            .filter(|f| {
                if let Some(subaddress_index) = f.subaddress_index.clone() {
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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let transaction_log = create_test_minted_and_change_txos(
            alice_account_key.clone(),
            bob_account_key.subaddress(0),
            72 * MOB,
            wallet_db.clone(),
            ledger_db.clone(),
            logger.clone(),
        );

        let associated_txos = transaction_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();

        assert_eq!(minted_txo.value as u64, 72 * MOB);
        assert_eq!(change_txo.value as u64, 928 * MOB - (2 * Mob::MINIMUM_FEE));

        // Add the minted Txos to the ledger
        add_block_with_db_txos(
            &mut ledger_db,
            &wallet_db,
            &[minted_txo.id.clone(), change_txo.id.clone()],
            &[KeyImage::from(for_bob_key_image)],
        );

        // Process the latest block for Bob (note, Bob is starting to sync from block 0)
        let _bob_account = manually_sync_account(&ledger_db, &wallet_db, &bob_account_id, &logger);
        // Process the latest block for Alice
        let _alice_account =
            manually_sync_account(&ledger_db, &wallet_db, &alice_account_id, &logger);

        // We should now have 1 txo in Bob's account.
        let txos = Txo::list_for_account(
            &AccountID::from(&bob_account_key).to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        let bob_txo = txos[0].clone();
        assert_eq!(bob_txo.subaddress_index.unwrap(), 0);
        assert!(bob_txo.key_image.is_some());
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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account
        // [100, 200, 300, ... 2000]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new((100 * MOB * i) as u64, Mob::ID), // 100.0 MOB * i
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        // Greedily take smallest to exact value
        let txos_for_value = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            300 * MOB,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value as u64));
        assert_eq!(result_set, HashSet::from_iter([100 * MOB, 200 * MOB]));

        // Once we include the fee, we need another txo
        let txos_for_value = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            300 * MOB + Mob::MINIMUM_FEE,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
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
            300 * MOB + Mob::MINIMUM_FEE,
            Some(200 * MOB),
            0,
            &wallet_db.get_conn().unwrap(),
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
            16800 * MOB,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account
        // [100, 200, 300, ... 2000]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new((100 * MOB * i) as u64, Mob::ID), // 100.0 MOB * i
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        // sum(300..1800) to get a window where we had to increase past the smallest
        // txos, and also fill up all 16 input slots.
        Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            16800 * MOB,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let res = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(),
            16800 * MOB,
            Some(100 * MOB),
            0,
            &wallet_db.get_conn().unwrap(),
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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Create some TXOs for the account. Total value is 2000, but max can spend is
        // 1600 [100, 100, ... 100]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new((100 * MOB) as u64, Mob::ID),
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        let res = Txo::select_spendable_txos_for_value(
            &account_id_hex.to_string(), // FIXME: WS-11 - take AccountID
            1800 * MOB,
            None,
            0,
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
            "".to_string(),
            "".to_string(),
            "".to_string(),
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

        let txos = Txo::list_for_account(
            &AccountID::from(&src_account).to_string(),
            None,
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(txos.len(), 12);

        let transaction_log = create_test_minted_and_change_txos(
            src_account.clone(),
            recipient,
            1 * MOB,
            wallet_db.clone(),
            ledger_db,
            logger,
        );

        let associated_txos = transaction_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        let (minted_txo, _) = associated_txos.outputs.first().unwrap();
        let (change_txo, _) = associated_txos.change.first().unwrap();

        assert_eq!(minted_txo.value as u64, 1 * MOB);
        assert!(minted_txo.account_id.is_none());

        assert_eq!(change_txo.value as u64, 4999 * MOB - Mob::MINIMUM_FEE);
        assert!(change_txo.account_id.is_none());
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
            "".to_string(),
            "".to_string(),
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Start sync thread
        log::info!(logger, "Starting sync thread");
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        log::info!(logger, "Creating a random sender account");
        let sender_account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );
        let sender_account_id = AccountID::from(&sender_account_key);

        // Create TxProposal from the sender account, which contains the Confirmation
        // Number
        log::info!(logger, "Creating transaction builder");
        let conn = wallet_db.get_conn().unwrap();
        let mut builder: WalletTransactionBuilder<MockFogPubkeyResolver> =
            WalletTransactionBuilder::new(
                AccountID::from(&sender_account_key).to_string(),
                ledger_db.clone(),
                get_resolver_factory(&mut rng).unwrap(),
                logger.clone(),
            );
        builder
            .add_recipient(recipient_account_key.default_subaddress(), 50 * MOB)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build(&conn).unwrap();

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
        manually_sync_account(&ledger_db, &wallet_db, &recipient_account_id, &logger);
        manually_sync_account(&ledger_db, &wallet_db, &sender_account_id, &logger);

        // Then let's make sure we received the Txo on the recipient account
        log::info!(logger, "Listing all Txos for recipient account");
        let txos = Txo::list_for_account(
            &recipient_account_id.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 1);

        let received_txo = txos[0].clone();

        // Note: Because this txo is both received and sent, between two different
        // accounts, its confirmation number does get updated. Typically, received txos
        // have None for the confirmation number.
        assert!(received_txo.shared_secret.is_some());

        // Get the txo from the sent perspective
        log::info!(logger, "Listing all Txos for sender account");
        let sender_txos = Txo::list_for_account(
            &sender_account_id.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // We seeded with 3 received (70, 80, 90), and a change txo
        assert_eq!(sender_txos.len(), 4);

        // Get the associated Txos with the transaction log
        log::info!(logger, "Getting associated Txos with the transaction");
        let associated = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();
        let sent_outputs = associated.outputs;
        assert_eq!(sent_outputs.len(), 1);
        let sent_txo_details =
            Txo::get(&sent_outputs[0].0.id, &wallet_db.get_conn().unwrap()).unwrap();

        // These two txos should actually be the same txo, and the account_txo_status is
        // what differentiates them.
        assert_eq!(sent_txo_details, received_txo);

        assert!(sent_txo_details.shared_secret.is_some());
        let confirmation: TxOutConfirmationNumber =
            mc_util_serial::decode(&sent_txo_details.shared_secret.unwrap()).unwrap();
        log::info!(logger, "Validating the confirmation number");
        let verified = Txo::validate_confirmation(
            &AccountID::from(&recipient_account_key),
            &received_txo.id,
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
        let (_account_id, _address) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "",
            "".to_string(),
            "".to_string(),
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
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

        let txos_and_status = Txo::select_by_public_key(&pubkeys, &wallet_db.get_conn().unwrap())
            .expect("Could not get txos by public keys");
        assert_eq!(txos_and_status.len(), 10);

        let txos_and_status =
            Txo::select_by_public_key(&pubkeys[0..5], &wallet_db.get_conn().unwrap())
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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let account = Account::get(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();

        // Create some txos.
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            0
        );
        for _ in 0..10 {
            let (_txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0,
                Amount::new((100 * MOB) as u64, Mob::ID), // 100.0 MOB * i
                (144) as u64,
                &mut rng,
                &wallet_db,
            );
        }
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            10
        );

        let txos = Txo::list_for_account(
            &account_id_hex.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 10);

        // Delete the account. No Txos are left.
        account.delete(&wallet_db.get_conn().unwrap()).unwrap();

        let txos = Txo::list_for_account(
            &account_id_hex.to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(txos.len(), 0);

        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            0
        );
    }

    #[test_with_logger]
    fn test_list_spendable_more_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

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
            "".to_string(),
            &conn,
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
        } = Txo::list_spendable(Some(&account_id.to_string()), None, None, 0, &conn).unwrap();

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
        let conn = wallet_db.get_conn().unwrap();

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
            "".to_string(),
            &conn,
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
        } = Txo::list_spendable(Some(&account_id.to_string()), None, None, 0, &conn).unwrap();

        assert_eq!(spendable_txos.len(), 10);
        assert_eq!(max_spendable_in_wallet as u64, 0);
    }

    #[test_with_logger]
    fn test_list_spendable_max_spendable_value(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

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
            "".to_string(),
            &conn,
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
            &conn,
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
        let conn = wallet_db.get_conn().unwrap();

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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let amount = Amount::new(28922973268924, Mob::ID);

        let (txo, key_image) =
            create_test_txo_for_recipient(&account_key, 1, amount.clone(), &mut rng);

        // create 1 txo with no key image and no subaddress
        Txo::create_received(
            txo.clone(),
            None,
            None,
            amount.clone(),
            15,
            &account_id.to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let txos = Txo::list_unspent(Some(&account_id.to_string()), None, None, None, None, &conn)
            .unwrap();
        assert_eq!(txos.len(), 0);

        // create 1 txo with subaddress, but not key image
        Txo::create_received(
            txo.clone(),
            Some(1),
            None,
            amount.clone(),
            15,
            &account_id.to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let txos = Txo::list_unspent(Some(&account_id.to_string()), None, None, None, None, &conn)
            .unwrap();
        assert_eq!(txos.len(), 0);

        // create 1 txo with key image and subaddress
        Txo::create_received(
            txo.clone(),
            Some(1),
            Some(key_image),
            amount.clone(),
            15,
            &account_id.to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let txos = Txo::list_unspent(Some(&account_id.to_string()), None, None, None, None, &conn)
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
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
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
                    Amount::new(i as u64 * MOB, Mob::ID),
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
                    Amount::new(i as u64 * MOB, TokenId::from(1)),
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
        let target_value: u64 = 200 as u64 * MOB - Mob::MINIMUM_FEE;
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            target_value,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 16);
        let sum: u64 = result.iter().map(|x| x.value as u64).sum();
        assert_eq!(target_value, sum - Mob::MINIMUM_FEE);
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_over_max_spendable_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            201 as u64 * MOB,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
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
            3 as u64,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test_with_logger]
    fn test_select_unspent_txos_target_value_over_total_mob_in_account(logger: Logger) {
        let (account_id, wallet_db) = setup_select_unspent_txos_tests(logger, false);

        let result = Txo::select_spendable_txos_for_value(
            &account_id.to_string(),
            500 as u64 * MOB,
            None,
            0,
            &wallet_db.get_conn().unwrap(),
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
            12400000000 as u64,
            None,
            0,
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
    // FIXME: test for selecting utxos from multiple subaddresses in one account
    // FIXME: test for one TXO belonging to multiple accounts with get
    // FIXME: test create_received for various permutations of multiple accounts
}
