// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::{AccountKey, PublicAddress, DEFAULT_SUBADDRESS_INDEX};
use mc_common::logger::{log, Logger};
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::RistrettoPublic;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::ring_signature::KeyImage;
use mc_transaction_core::tx::{Tx, TxOut};
use std::iter::FromIterator;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;

use crate::error::WalletDbError;
use crate::models::{
    Account, AccountTxoStatus, AssignedSubaddress, NewAccount, NewAccountTxoStatus,
    NewAssignedSubaddress, NewTransactionLog, NewTransactionTxoType, NewTxo, TransactionLog,
    TransactionTxoType, Txo,
};
// Schema Tables
use crate::schema::account_txo_statuses as schema_account_txo_statuses;
use crate::schema::accounts as schema_accounts;
use crate::schema::assigned_subaddresses as schema_assigned_subaddresses;
use crate::schema::transaction_logs as schema_transaction_logs;
use crate::schema::transaction_txo_types as schema_transaction_txo_types;
use crate::schema::txos as schema_txos;

// Query Objects
use crate::schema::account_txo_statuses::dsl::account_txo_statuses as dsl_account_txo_statuses;
use crate::schema::accounts::dsl::accounts as dsl_accounts;
use crate::schema::txos::dsl::txos as dsl_txos;

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

#[derive(Debug)]
pub struct AccountID(String);

impl From<&AccountKey> for AccountID {
    fn from(src: &AccountKey) -> AccountID {
        let main_subaddress = src.subaddress(DEFAULT_SUBADDRESS_INDEX);
        /// The account ID is derived from the contents of the account key
        #[derive(Digestible)]
        struct ConstAccountData {
            /// The public address of the main subaddress for this account
            pub address: PublicAddress,
        }
        let const_data = ConstAccountData {
            address: main_subaddress.clone(),
        };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"account_data");
        Self(hex::encode(temp))
    }
}

impl AccountID {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug)]
pub struct TxoID(String);

impl From<&TxOut> for TxoID {
    fn from(src: &TxOut) -> TxoID {
        /// The txo ID is derived from the contents of the txo
        #[derive(Digestible)]
        struct ConstTxoData {
            pub txo: TxOut,
        }
        let const_data = ConstTxoData { txo: src.clone() };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"txo_data");
        Self(hex::encode(temp))
    }
}

impl TxoID {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug)]
pub struct TransactionID(String);

impl From<&Tx> for TransactionID {
    fn from(src: &Tx) -> TransactionID {
        /// The txo ID is derived from the contents of the txo
        #[derive(Digestible)]
        struct ConstTransactionData {
            pub tx: Tx,
        }
        let const_data = ConstTransactionData { tx: src.clone() };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"transaction_data");
        Self(hex::encode(temp))
    }
}

impl TransactionID {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
    logger: Logger,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>, logger: Logger) -> Self {
        Self { pool, logger }
    }

    pub fn new_from_url(database_url: &str, logger: Logger) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool, logger))
    }

    /// Create a new account.
    pub fn create_account(
        &self,
        account_key: &AccountKey,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        first_block: u64,
        next_block: u64,
        name: &str,
    ) -> Result<(String, String), WalletDbError> {
        let conn = self.pool.get()?;

        let main_subaddress = account_key.subaddress(main_subaddress_index);
        let account_id = AccountID::from(account_key);

        // FIXME: It's concerning to lose a bit of precision in casting to i64
        let new_account = NewAccount {
            account_id_hex: &account_id.to_string(),
            encrypted_account_key: &mc_util_serial::encode(account_key), // FIXME: add encryption
            main_subaddress_index: main_subaddress_index as i64,
            change_subaddress_index: change_subaddress_index as i64,
            next_subaddress_index: next_subaddress_index as i64,
            first_block: first_block as i64,
            next_block: next_block as i64,
            name,
        };

        diesel::insert_into(schema_accounts::table)
            .values(&new_account)
            .execute(&conn)?;

        // Insert the assigned subaddresses for main and change
        let main_subaddress_b58 = b58_encode(&main_subaddress)?;
        let main_subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &main_subaddress_b58,
            account_id_hex: &account_id.to_string(),
            address_book_entry: None, // FIXME: Address Book Entry if details provided, or None always for main?
            public_address: &mc_util_serial::encode(&main_subaddress),
            subaddress_index: main_subaddress_index as i64,
            comment: "Main",
            expected_value: None,
            subaddress_spend_key: &mc_util_serial::encode(main_subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&main_subaddress_entry)
            .execute(&conn)?;

        let change_subaddress = account_key.subaddress(change_subaddress_index);
        let change_subaddress_b58 = b58_encode(&change_subaddress)?;
        let change_subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &change_subaddress_b58,
            account_id_hex: &account_id.to_string(),
            address_book_entry: None, // FIXME: Address Book Entry if details provided, or None always for main?
            public_address: &mc_util_serial::encode(&change_subaddress),
            subaddress_index: change_subaddress_index as i64,
            comment: "Change",
            expected_value: None,
            subaddress_spend_key: &mc_util_serial::encode(change_subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&change_subaddress_entry)
            .execute(&conn)?;

        Ok((account_id.to_string(), main_subaddress_b58))
    }

    /// List all accounts.
    pub fn list_accounts(&self) -> Result<Vec<Account>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<Account> = schema_accounts::table
            .select(schema_accounts::all_columns)
            .load::<Account>(&conn)?;
        Ok(results)
    }

    /// Get a specific account
    pub fn get_account(&self, account_id_hex: &str) -> Result<Account, WalletDbError> {
        let conn = self.pool.get()?;

        match dsl_accounts
            .find(account_id_hex)
            .get_result::<Account>(&conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::NotFound(account_id_hex.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Update an account.
    /// The only updatable field is the name. Any other desired update requires adding
    /// a new account, and deleting the existing if desired.
    pub fn update_account_name(
        &self,
        account_id_hex: &str,
        new_name: String,
    ) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        diesel::update(dsl_accounts.find(account_id_hex))
            .set(schema_accounts::name.eq(new_name))
            .execute(&conn)?;
        Ok(())
    }

    /// Delete an account.
    pub fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        diesel::delete(dsl_accounts.find(account_id_hex)).execute(&conn)?;
        Ok(())
    }

    pub fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: &str,
    ) -> Result<String, WalletDbError> {
        let conn = self.pool.get()?;

        let account: Account = match dsl_accounts
            .find(account_id_hex)
            .get_result::<Account>(&conn)
        {
            Ok(a) => a,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::NotFound(account_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let account_key: AccountKey = mc_util_serial::decode(&account.encrypted_account_key)?;
        let subaddress_index = account.next_subaddress_index;
        let subaddress = account_key.subaddress(subaddress_index as u64);

        let subaddress_b58 = b58_encode(&subaddress)?;
        let subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &subaddress_b58,
            account_id_hex,
            address_book_entry: None, // FIXME: Address Book Entry if details provided, or None always for main?
            public_address: &mc_util_serial::encode(&subaddress),
            subaddress_index: subaddress_index as i64,
            comment,
            expected_value: None, // FIXME: rethink if we need this
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&subaddress_entry)
            .execute(&conn)?;

        // Update the next subaddress index for the account
        diesel::update(dsl_accounts.find(account_id_hex))
            .set(schema_accounts::next_subaddress_index.eq(subaddress_index + 1))
            .execute(&conn)?;

        Ok(subaddress_b58)
    }

    /// Create a TXO entry
    pub fn create_received_txo(
        &self,
        txo: TxOut,
        subaddress_index: u64,
        key_image: KeyImage,
        value: u64,
        received_block_height: i64,
        account_id_hex: &str,
    ) -> Result<String, WalletDbError> {
        let conn = self.pool.get()?;

        let txo_id = TxoID::from(&txo);

        let key_image_bytes = mc_util_serial::encode(&key_image);
        let new_txo = NewTxo {
            txo_id_hex: &txo_id.to_string(),
            value: value as i64,
            target_key: &mc_util_serial::encode(&txo.target_key),
            public_key: &mc_util_serial::encode(&txo.public_key),
            e_fog_hint: &mc_util_serial::encode(&txo.e_fog_hint),
            txo: &mc_util_serial::encode(&txo),
            subaddress_index: subaddress_index as i64,
            key_image: Some(&key_image_bytes),
            received_block_height: Some(received_block_height as i64),
            spent_tombstone_block_height: None,
            spent_block_height: None,
            proof: None,
        };

        diesel::insert_into(schema_txos::table)
            .values(&new_txo)
            .execute(&conn)?;

        let new_account_txo_status = NewAccountTxoStatus {
            account_id_hex: &account_id_hex,
            txo_id_hex: &txo_id.to_string(),
            txo_status: "unspent",
            txo_type: "received",
        };

        diesel::insert_into(schema_account_txo_statuses::table)
            .values(&new_account_txo_status)
            .execute(&conn)?;

        Ok(txo_id.to_string())
    }

    /// List all txos for a given account.
    pub fn list_txos(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<(Txo, AccountTxoStatus)> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))),
            )
            .select((
                schema_txos::all_columns,
                schema_account_txo_statuses::all_columns,
            ))
            .load(&conn)?;
        Ok(results)
    }

    pub fn list_txos_by_status(
        &self,
        account_id_hex: &str,
    ) -> Result<HashMap<String, Vec<Txo>>, WalletDbError> {
        let conn = self.pool.get()?;

        // FIXME: don't do 4 queries
        let unspent: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("unspent"))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;

        let pending: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("pending"))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;

        let spent: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("spent"))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;

        let unknown: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("unknown"))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;
        let results = HashMap::from_iter(vec![
            ("unspent".to_string(), unspent),
            ("pending".to_string(), pending),
            ("spent".to_string(), spent),
            ("unknown".to_string(), unknown),
        ]);
        Ok(results)
    }

    pub fn select_txos_by_id(
        &self,
        account_id_hex: &str,
        txo_ids: &Vec<String>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError> {
        let conn = self.pool.get()?;

        let mut results: Vec<(Txo, AccountTxoStatus)> = Vec::new();
        for txo_id in txo_ids {
            match dsl_txos.find(txo_id).get_result::<Txo>(&conn) {
                Ok(txo) => {
                    // Check that this txo is indeed owned by the account we think it is
                    match dsl_account_txo_statuses
                        .find((account_id_hex, txo_id))
                        .get_result::<AccountTxoStatus>(&conn)
                    {
                        Ok(status) => {
                            results.push((txo, status));
                        }
                        Err(diesel::result::Error::NotFound) => {
                            return Err(WalletDbError::NotFound(format!(
                                "Txo({:?}) found, but does not belong to Account({:?})",
                                txo_id, account_id_hex
                            )));
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Err(diesel::result::Error::NotFound) => {
                    return Err(WalletDbError::NotFound(txo_id.to_string()));
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Ok(results)
    }

    pub fn select_unspent_txos_for_value(
        &self,
        account_id_hex: &str,
        max_spendable_value: i64,
    ) -> Result<Vec<Txo>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("unspent"))
                    .and(schema_txos::value.lt(max_spendable_value))),
            )
            .select(schema_txos::all_columns)
            .order_by(schema_txos::value.desc())
            .load(&conn)?;

        Ok(results)
    }

    /// List all subaddresses for a given account.
    pub fn list_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<AssignedSubaddress>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<AssignedSubaddress> = schema_accounts::table
            .inner_join(
                schema_assigned_subaddresses::table.on(schema_accounts::account_id_hex
                    .eq(schema_assigned_subaddresses::account_id_hex)
                    .and(schema_accounts::account_id_hex.eq(account_id_hex))),
            )
            .select(schema_assigned_subaddresses::all_columns)
            .load(&conn)?;

        Ok(results)
    }

    pub fn get_subaddress_index_by_subaddress_spend_public_key(
        &self,
        subaddress_spend_public_key: &RistrettoPublic,
    ) -> Result<(i64, String), WalletDbError> {
        let conn = self.pool.get()?;

        let matches = schema_assigned_subaddresses::table
            .select((
                schema_assigned_subaddresses::subaddress_index,
                schema_assigned_subaddresses::account_id_hex,
            ))
            .filter(
                schema_assigned_subaddresses::subaddress_spend_key
                    .eq(mc_util_serial::encode(subaddress_spend_public_key)),
            )
            .load::<(i64, String)>(&conn)?;

        if matches.len() == 0 {
            Err(WalletDbError::NotFound(format!(
                "{:?}",
                subaddress_spend_public_key
            )))
        } else if matches.len() > 1 {
            Err(WalletDbError::DuplicateEntries(format!(
                "{:?}",
                subaddress_spend_public_key
            )))
        } else {
            Ok(matches[0].clone())
        }
    }

    pub fn update_spent_and_increment_next_block(
        &self,
        account_id_hex: &str,
        spent_block_height: i64,
        key_images: Vec<KeyImage>,
    ) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        for key_image in key_images {
            // Get the txo by key_image
            let matches = schema_txos::table
                .select(schema_txos::all_columns)
                .filter(schema_txos::key_image.eq(mc_util_serial::encode(&key_image)))
                .load::<Txo>(&conn)?;

            if matches.len() == 0 {
                // Not Found is ok - this means it's a key_image not associated with any of our txos
                continue;
            } else if matches.len() > 1 {
                return Err(WalletDbError::DuplicateEntries(format!(
                    "Key Image: {:?}",
                    key_image
                )));
            } else {
                // Update the TXO
                log::trace!(
                    self.logger,
                    "Updating spent for account {:?} at block height {:?} with key_image {:?}",
                    account_id_hex,
                    spent_block_height,
                    key_image
                );
                diesel::update(dsl_txos.find(&matches[0].txo_id_hex))
                    .set(schema_txos::spent_block_height.eq(Some(spent_block_height)))
                    .execute(&conn)?;

                // Update the AccountTxoStatus
                diesel::update(
                    dsl_account_txo_statuses.find((account_id_hex, &matches[0].txo_id_hex)),
                )
                .set(schema_account_txo_statuses::txo_status.eq("spent".to_string()))
                .execute(&conn)?;

                // FIXME: make sure the path for all txo_statuses and txo_types exist and are tested
            }
        }
        diesel::update(dsl_accounts.find(account_id_hex))
            .set(schema_accounts::next_block.eq(spent_block_height + 1))
            .execute(&conn)?;
        Ok(())
    }

    /// When submitting a transaction, we store relevant information to the transaction logs,
    /// and we also track information about each of the txos involved in the transaction.
    ///
    /// Note: We expect transactions created with this wallet to have one recipient, with the
    ///       rest of the minted txos designated as change. Other wallets may choose to behave
    ///       differently, but our TransactionLogs Table assumes this behavior.
    pub fn log_submitted_transaction(
        &self,
        tx_proposal: TxProposal,
        block_height: u64,
        comment: String,
    ) -> Result<String, WalletDbError> {
        let conn = self.pool.get()?;

        // FIXME: batch all these updates to make this an atomic operation

        // Store the txo_id -> transaction_txo_type
        let mut txo_ids: Vec<(String, &str)> = Vec::new();

        // First update all inputs to "pending." They will remain pending until their key_image
        // hits the ledger.
        let account_id = {
            let mut account_id = None;
            for utxo in tx_proposal.utxos.iter() {
                // Get the associated TxoID
                let txo_id = TxoID::from(&utxo.tx_out);

                // FIXME: Update the list rather than iterating, querying, etc
                // Find the account associated with this Txo
                let matches = schema_account_txo_statuses::table
                    .select(schema_account_txo_statuses::account_id_hex)
                    .filter(schema_account_txo_statuses::txo_id_hex.eq(txo_id.to_string()))
                    .load::<String>(&conn)?;

                if matches.is_empty() {
                    return Err(WalletDbError::NotFound(txo_id.to_string()));
                } else if matches.len() > 1 {
                    return Err(WalletDbError::DuplicateEntries(txo_id.to_string()));
                } else {
                    let txo_account_id = matches[0].clone();

                    if let Some(current_account_id) = account_id.as_ref() {
                        if current_account_id != &txo_account_id {
                            // Not currently possible to construct a transaction from multiple accounts
                            return Err(WalletDbError::MultipleAccountIDsInTransaction);
                        }
                    } else {
                        account_id = Some(txo_account_id.clone());
                    }

                    // Update the status
                    diesel::update(
                        dsl_account_txo_statuses
                            .find((txo_account_id.clone(), &txo_id.to_string())),
                    )
                    .set(schema_account_txo_statuses::txo_status.eq("pending".to_string()))
                    .execute(&conn)?;

                    txo_ids.push((txo_id.to_string(), "input"));
                }
            }
            account_id
        };

        // Sanity Check
        let account_id_hex = if let Some(account_id) = account_id {
            account_id
        } else {
            return Err(WalletDbError::TransactionLacksAccount);
        };

        // Next, add all of our minted outputs to the Txo Table
        let (recipient_address, transaction_value) = {
            let mut recipient_address = None;
            let mut value_sum = 0;
            for (i, output) in tx_proposal.tx.prefix.outputs.iter().enumerate() {
                let txo_id = TxoID::from(output);

                // FIXME: currently only have the value and proofs for outlays, not change - will need
                //        to amend what's saved in the TxProposal to include change as outlays
                let (value, proof, outlay_receiver) = if let Some(outlay_index) = tx_proposal
                    .outlay_index_to_tx_out_index
                    .iter()
                    .find_map(|(k, &v)| if v == i { Some(k) } else { None })
                {
                    let outlay = &tx_proposal.outlays[outlay_index.clone()];
                    (
                        outlay.value,
                        Some(outlay_index.clone()),
                        Some(outlay.receiver.clone()),
                    )
                } else {
                    (0, None, None)
                };

                // Update receiver, transaction_value, and transaction_txo_type, if outlay was found
                if let Some(receiver) = outlay_receiver {
                    if let Some(current_recipient) = recipient_address.as_ref() {
                        if current_recipient != &receiver {
                            // FIXME: we may not want to error here, but instead log a null for recipient?
                            //        or else just make two entries?
                            return Err(WalletDbError::MultipleRecipientsInTransaction);
                        }
                    } else {
                        recipient_address = Some(receiver);
                    }
                    value_sum += value;
                    txo_ids.push((txo_id.to_string(), "output"));
                } else {
                    // If not in an outlay, this output is change FIXME: Is this a good enough check?
                    txo_ids.push((txo_id.to_string(), "change"));
                }

                // FIXME: Not yet looking up subaddress_index for recipient in AssignedSubaddresses Table
                // FIXME: It's also not clear to me what this value should be for minted TXOs, and
                //        I'm assuming for change it should be the change subaddress?
                let subaddress_index = 0;

                let encoded_proof = proof
                    .map(|p| mc_util_serial::encode(&tx_proposal.outlay_confirmation_numbers[p]));

                let new_txo = NewTxo {
                    txo_id_hex: &txo_id.to_string(),
                    value: value as i64,
                    target_key: &mc_util_serial::encode(&output.target_key),
                    public_key: &mc_util_serial::encode(&output.public_key),
                    e_fog_hint: &mc_util_serial::encode(&output.e_fog_hint),
                    txo: &mc_util_serial::encode(output),
                    subaddress_index: subaddress_index as i64,
                    key_image: None, // Only the recipient can calculate the KeyImage
                    received_block_height: None,
                    spent_tombstone_block_height: Some(
                        tx_proposal.tx.prefix.tombstone_block as i64,
                    ),
                    spent_block_height: None,
                    proof: encoded_proof.as_ref(),
                };

                diesel::insert_into(schema_txos::table)
                    .values(&new_txo)
                    .execute(&conn)?;

                let new_account_txo_status = NewAccountTxoStatus {
                    account_id_hex: &account_id_hex,
                    txo_id_hex: &txo_id.to_string(),
                    txo_status: "unknown", // We cannot track spent status for minted TXOs unless change
                    txo_type: "minted",
                };

                diesel::insert_into(schema_account_txo_statuses::table)
                    .values(&new_account_txo_status)
                    .execute(&conn)?;
            }
            (recipient_address, value_sum)
        };

        if let Some(recipient) = recipient_address {
            let transaction_id = TransactionID::from(&tx_proposal.tx);
            // Create a TransactionLogs entry
            let new_transaction_log = NewTransactionLog {
                transaction_id_hex: &transaction_id.to_string(),
                account_id_hex: &account_id_hex.to_string(),
                recipient_public_address_b58: &b58_encode(&recipient)?,
                assigned_subaddress_b58: "", // FIXME get this from looking up the subaddress, or is this the same as recipient?
                value: transaction_value as i64,
                fee: Some(tx_proposal.tx.prefix.fee as i64),
                status: "pending",
                sent_time: "",                     // FIXME: what format do we want?
                block_height: block_height as i64, // FIXME: is this going to do what we want? It's
                // submitted block height, but not necessarily when it hits the ledger - would we
                // update when we see a key_image from this transaction?
                comment: &comment,
                direction: "sent",
            };

            diesel::insert_into(schema_transaction_logs::table)
                .values(&new_transaction_log)
                .execute(&conn)?;

            // Create an entry per TXO for the TransactionTxoTypes
            for (txo_id_hex, transaction_txo_type) in txo_ids {
                let new_transaction_txo = NewTransactionTxoType {
                    transaction_id_hex: &transaction_id.to_string(),
                    txo_id_hex: &txo_id_hex,
                    transaction_txo_type,
                };
                diesel::insert_into(schema_transaction_txo_types::table)
                    .values(&new_transaction_txo)
                    .execute(&conn)?;
            }

            Ok(transaction_id.to_string())
        } else {
            Err(WalletDbError::TransactionLacksRecipient)
        }
    }

    /// Returns vector of (Transaction, Inputs, Outputs, Change)
    pub fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<(TransactionLog, Vec<String>, Vec<String>, Vec<String>)>, WalletDbError> {
        let conn = self.pool.get()?;

        // FIXME: use group_by rather than the processing below:
        // https://docs.diesel.rs/diesel/associations/trait.GroupedBy.html
        let transactions: Vec<(TransactionLog, TransactionTxoType)> =
            schema_transaction_logs::table
                .inner_join(
                    schema_transaction_txo_types::table.on(
                        schema_transaction_logs::transaction_id_hex
                            .eq(schema_transaction_txo_types::transaction_id_hex)
                            .and(schema_transaction_logs::account_id_hex.eq(account_id_hex)),
                    ),
                )
                .select((
                    schema_transaction_logs::all_columns,
                    schema_transaction_txo_types::all_columns,
                ))
                .load(&conn)?;

        #[derive(Clone)]
        struct TransactionContents {
            transaction_log: TransactionLog,
            inputs: Vec<String>,
            outputs: Vec<String>,
            change: Vec<String>,
        }
        let mut results: HashMap<String, TransactionContents> = HashMap::default();
        for (transaction, transaction_txo_type) in transactions {
            if results.get(&transaction.transaction_id_hex).is_none() {
                results.insert(
                    transaction.transaction_id_hex.clone(),
                    TransactionContents {
                        transaction_log: transaction.clone(),
                        inputs: Vec::new(),
                        outputs: Vec::new(),
                        change: Vec::new(),
                    },
                );
            };

            let mut entry = results.get_mut(&transaction.transaction_id_hex).unwrap();

            entry.transaction_log = transaction;
            if transaction_txo_type.transaction_txo_type == "input" {
                entry.inputs.push(transaction_txo_type.txo_id_hex);
            } else if transaction_txo_type.transaction_txo_type == "output" {
                entry.outputs.push(transaction_txo_type.txo_id_hex);
            } else if transaction_txo_type.transaction_txo_type == "change" {
                entry.change.push(transaction_txo_type.txo_id_hex);
            }
        }
        Ok(results
            .values()
            .cloned()
            .map(|t| (t.transaction_log, t.inputs, t.outputs, t.change))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_transaction_core::encrypted_fog_hint::EncryptedFogHint;
    use mc_transaction_core::onetime_keys::recover_public_subaddress_spend_key;
    use mc_transaction_core::ring_signature::KeyImage;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashSet;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);
        let (account_id_hex, _public_address_b58) = walletdb
            .create_account(&account_key, 0, 1, 2, 0, 1, "Alice's Main Account")
            .unwrap();

        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

        let acc = walletdb.get_account(&account_id_hex).unwrap();
        let expected_account = Account {
            account_id_hex: account_id_hex.clone(),
            encrypted_account_key: mc_util_serial::encode(&account_key),
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block: 0,
            next_block: 1,
            name: "Alice's Main Account".to_string(),
        };
        assert_eq!(expected_account, acc);

        // Verify that the subaddress table entries were updated for main and change
        let subaddresses = walletdb.list_subaddresses(&account_id_hex).unwrap();
        assert_eq!(subaddresses.len(), 2);
        let subaddress_indices: HashSet<i64> =
            HashSet::from_iter(subaddresses.iter().map(|s| s.subaddress_index));
        assert!(subaddress_indices.get(&0).is_some());
        assert!(subaddress_indices.get(&1).is_some());

        // Verify that we can get the correct subaddress index from the spend public key
        let main_subaddress = account_key.subaddress(0);
        let (retrieved_index, retrieved_acocunt_id_hex) = walletdb
            .get_subaddress_index_by_subaddress_spend_public_key(main_subaddress.spend_public_key())
            .unwrap();
        assert_eq!(retrieved_index, 0);
        assert_eq!(retrieved_acocunt_id_hex, account_id_hex);

        // Add another account with no name, scanning from later
        let account_key_secondary = AccountKey::from(&RootIdentity::from_random(&mut rng));
        let (account_id_hex_secondary, _public_address_b58_secondary) = walletdb
            .create_account(&account_key_secondary, 0, 1, 2, 50, 51, "")
            .unwrap();
        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 2);

        let acc_secondary = walletdb.get_account(&account_id_hex_secondary).unwrap();
        let mut expected_account_secondary = Account {
            account_id_hex: account_id_hex_secondary.clone(),
            encrypted_account_key: mc_util_serial::encode(&account_key_secondary),
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block: 50,
            next_block: 51,
            name: "".to_string(),
        };
        assert_eq!(expected_account_secondary, acc_secondary);

        // Update the name for the secondary account
        walletdb
            .update_account_name(
                &account_id_hex_secondary,
                "Alice's Secondary Account".to_string(),
            )
            .unwrap();
        let acc_secondary2 = walletdb.get_account(&account_id_hex_secondary).unwrap();
        expected_account_secondary.name = "Alice's Secondary Account".to_string();
        assert_eq!(expected_account_secondary, acc_secondary2);

        // Delete the secondary account
        walletdb.delete_account(&account_id_hex_secondary).unwrap();

        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

        // Attempt to get the deleted account
        let res = walletdb.get_account(&account_id_hex_secondary);
        match res {
            Ok(_) => panic!("Should have deleted account"),
            Err(WalletDbError::NotFound(s)) => assert_eq!(s, account_id_hex_secondary.to_string()),
            Err(_) => panic!("Should error with NotFound but got {:?}", res),
        }
    }

    #[test_with_logger]
    fn test_received_tx_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);
        let (account_id_hex, _public_address_b58) = walletdb
            .create_account(&account_key, 0, 1, 2, 0, 1, "Alice's Main Account")
            .unwrap();

        // FIXME: get recipient via the assigned subaddresses table, not directly
        let recipient = account_key.subaddress(0);

        // Create TXO for the account
        let tx_private_key = RistrettoPrivate::from_random(&mut rng);
        let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
        let value = 10;
        let txo = TxOut::new(value, &recipient, &tx_private_key, hint).unwrap();

        // Get KeyImage from the onetime private key
        let key_image = KeyImage::from(&tx_private_key);

        // Sanity check: Ensure that we can recover the subaddress
        // FIXME: Assert that the public address and the subaddress spend key was added to the
        //        assigned_subaddresses table
        let _subaddress_index = recover_public_subaddress_spend_key(
            account_key.view_private_key(),
            &RistrettoPublic::try_from(&txo.target_key).unwrap(),
            &RistrettoPublic::try_from(&txo.public_key).unwrap(),
        );
        let subaddress_index = 0;

        let received_block_height = 144;

        let txo_hex = walletdb
            .create_received_txo(
                txo.clone(),
                subaddress_index,
                key_image,
                value,
                received_block_height,
                &account_id_hex,
            )
            .unwrap();

        let txos = walletdb.list_txos(&account_id_hex).unwrap();
        assert_eq!(txos.len(), 1);

        let expected_txo = Txo {
            txo_id_hex: txo_hex.clone(),
            value: value as i64,
            target_key: mc_util_serial::encode(&txo.target_key),
            public_key: mc_util_serial::encode(&txo.public_key),
            e_fog_hint: mc_util_serial::encode(&txo.e_fog_hint),
            txo: mc_util_serial::encode(&txo),
            subaddress_index: subaddress_index as i64,
            key_image: Some(mc_util_serial::encode(&key_image)),
            received_block_height: Some(received_block_height as i64),
            spent_tombstone_block_height: None,
            spent_block_height: None,
            proof: None,
        };
        // Verify that the statuses table was updated correctly
        let expected_txo_status = AccountTxoStatus {
            account_id_hex: account_id_hex.clone(),
            txo_id_hex: txo_hex,
            txo_status: "unspent".to_string(),
            txo_type: "received".to_string(),
        };
        assert_eq!(txos[0].0, expected_txo);
        assert_eq!(txos[0].1, expected_txo_status);

        // Verify that the status filter works as well
        let balances = walletdb.list_txos_by_status(&account_id_hex).unwrap();
        assert_eq!(balances["unspent"].len(), 1);

        // Now we'll "spend" the TXO
        // FIXME TODO: construct transaction proposal to spend it, maybe needs a helper in test_utils
        // self.update_submitted_transaction(tx_proposal)?;

        // Now we'll process the ledger and verify that the TXO was spent
        let spent_block_height = 365;

        walletdb
            .update_spent_and_increment_next_block(
                &account_id_hex,
                spent_block_height,
                vec![key_image],
            )
            .unwrap();

        let txos = walletdb.list_txos(&account_id_hex).unwrap();
        assert_eq!(txos.len(), 1);
        assert_eq!(
            txos[0].0.spent_block_height.unwrap(),
            spent_block_height as i64
        );
        assert_eq!(txos[0].1.txo_status, "spent".to_string());

        // Verify that the next block height is + 1
        let account = walletdb.get_account(&account_id_hex).unwrap();
        assert_eq!(account.next_block, spent_block_height + 1);

        // Verify that there are no unspent txos
        let balances = walletdb.list_txos_by_status(&account_id_hex).unwrap();
        assert!(balances["unspent"].is_empty());
    }
}
