// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use chrono::prelude::Utc;
use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::logger::{log, Logger};
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::ring_signature::KeyImage;
use mc_transaction_core::tx::Tx;
use std::iter::FromIterator;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    RunQueryDsl,
};

use crate::db_models::txo::{TxoID, TxoModel};
use crate::error::WalletDbError;
use crate::models::{
    Account, AccountTxoStatus, AssignedSubaddress, NewAccountTxoStatus, NewAssignedSubaddress,
    NewTransactionLog, NewTransactionTxoType, NewTxo, TransactionLog, TransactionTxoType, Txo,
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
use crate::schema::transaction_logs::dsl::transaction_logs as dsl_transaction_logs;
use crate::schema::txos::dsl::txos as dsl_txos;

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

#[derive(Debug)]
pub struct TransactionID(String);

// TransactionID is formed from the contents of the transaction when sent
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

// TransactionID is formed from the received TxoIDs when received
impl From<&Vec<String>> for TransactionID {
    fn from(src: &Vec<String>) -> TransactionID {
        /// The txo ID is derived from the contents of the txo
        #[derive(Digestible)]
        struct ConstTransactionData {
            pub txo_ids: Vec<String>,
        }
        let const_data = ConstTransactionData {
            txo_ids: src.clone(),
        };
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

    pub fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: &str,
    ) -> Result<(String, i64), WalletDbError> {
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
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&subaddress_entry)
            .execute(&conn)?;
        // Update the next subaddress index for the account
        // Note: we also update the first_block back to 0 to scan from the beginning of the
        //       ledger for this new subaddress.
        // FIXME: pass in a "sync from" block rather than 0
        let sync_from = 0;
        diesel::update(dsl_accounts.find(account_id_hex))
            .set((
                schema_accounts::next_subaddress_index.eq(subaddress_index + 1),
                schema_accounts::next_block.eq(sync_from),
            ))
            .execute(&conn)?;

        // Update the next subaddress index for the account
        diesel::update(dsl_accounts.find(account_id_hex))
            .set((schema_accounts::next_subaddress_index.eq(subaddress_index + 1),))
            .execute(&conn)?;

        // FIXME: Currently getting 2020-11-30 04:15:15.758107 UTC ERRO error syncing monitor 873b295eb48dccd3612b64ec69d5dc26c0923aa3545a783c6423354bea5de3e8:
        // WalletDb(DieselError(DatabaseError(UniqueViolation, "UNIQUE constraint failed: transaction_logs.transaction_id_hex"))),
        // mc.app: wallet-service, mc.module: mc_wallet_service::sync, mc.src: src/sync.rs:282
        //        when creating a new subaddress to try to pick up orphaned transactions

        // FIXME: When adding multiple addresses for syncing, possibly due to error above, the
        //        syncing stops, and the account is stuck at 0
        Ok((subaddress_b58, subaddress_index))
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

        // FIXME: Maybe we don't want to expose this in the balance
        let secreted: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("secreted"))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;

        let orphaned: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(schema_account_txo_statuses::txo_status.eq("orphaned"))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;

        let results = HashMap::from_iter(vec![
            ("unspent".to_string(), unspent),
            ("pending".to_string(), pending),
            ("spent".to_string(), spent),
            ("secreted".to_string(), secreted),
            ("orphaned".to_string(), orphaned),
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
                    .and(schema_txos::subaddress_index.is_not_null())
                    .and(schema_txos::key_image.is_not_null()) // Could technically recreate with subaddress
                    .and(schema_txos::value.lt(max_spendable_value))),
            )
            .select(schema_txos::all_columns)
            .order_by(schema_txos::value.desc())
            .load(&conn)?;

        // println!("\x1b[1;34mselected the following txos {:?}\x1b[0m", results);

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
                // Update the transaction status if the txos are all spent
                self.update_transaction_status(
                    &matches[0].txo_id_hex,
                    spent_block_height as u64,
                    &conn,
                )?;
            }
        }
        diesel::update(dsl_accounts.find(account_id_hex))
            .set(schema_accounts::next_block.eq(spent_block_height + 1))
            .execute(&conn)?;
        Ok(())
    }

    fn update_transaction_status(
        &self,
        txo_id_hex: &str,
        cur_block_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        // Get associated transaction IDs from txo - FIXME: make own function on model
        let associated_transactions: Vec<TransactionLog> = schema_transaction_logs::table
            .inner_join(
                schema_transaction_txo_types::table.on(schema_transaction_logs::transaction_id_hex
                    .eq(schema_transaction_txo_types::transaction_id_hex)
                    .and(schema_transaction_txo_types::txo_id_hex.eq(txo_id_hex))),
            )
            .select(schema_transaction_logs::all_columns)
            .load(conn)?;

        for transaction in associated_transactions {
            let (transaction, inputs, _outputs, _change) =
                self.get_transaction(&transaction.transaction_id_hex, conn)?;

            // Only update if proposed or pending
            if transaction.status == "succeeded" || transaction.status == "failed" {
                continue;
            }

            let num_inputs = inputs.len();
            let mut spent_count = 0;
            let mut tombstone_exceeded_count = 0;
            for input_id in inputs {
                let (txo, txo_status, _assigned_subaddress) =
                    Txo::get(&transaction.account_id_hex, &input_id, conn)?;
                if txo_status.txo_status == "spent" {
                    spent_count += 1;
                } else {
                    if let Some(tombstone) = txo.spent_tombstone_block_height {
                        if cur_block_height > tombstone as u64 {
                            tombstone_exceeded_count += 1;
                        }
                    }
                }
            }
            if spent_count == num_inputs {
                // FIXME: Also update the block height where it succeeded
                diesel::update(dsl_transaction_logs.find(&transaction.transaction_id_hex))
                    .set(schema_transaction_logs::status.eq("succeeded"))
                    .execute(conn)?;
            } else if tombstone_exceeded_count > 0 {
                // FIXME: Also update the block height where it failed
                diesel::update(dsl_transaction_logs.find(&transaction.transaction_id_hex))
                    .set(schema_transaction_logs::status.eq("failed"))
                    .execute(conn)?;
            }
        }

        Ok(())
    }

    pub fn log_received_transactions(
        &self,
        subaddress_to_output_txo_ids: HashMap<i64, Vec<String>>,
        account: &Account,
        block_height: u64,
    ) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        for (subaddress_index, output_txo_ids) in subaddress_to_output_txo_ids {
            let transaction_id = TransactionID::from(&output_txo_ids.to_vec());

            // Check that we haven't already logged this transaction on a previous sync
            match dsl_transaction_logs
                .find(&transaction_id.to_string())
                .first::<TransactionLog>(&conn)
            {
                Ok(_) => continue, // We've already processed this transaction on a previous sync
                Err(diesel::result::Error::NotFound) => {} // Insert below
                Err(e) => return Err(e.into()),
            }

            // FIXME: should move onto model
            let txos = schema_txos::table
                .select(schema_txos::all_columns)
                .filter(schema_txos::txo_id_hex.eq_any(output_txo_ids))
                .load::<Txo>(&conn)?;

            let transaction_value: i64 = txos.iter().map(|t| t.value).sum();

            // Get the public address for the subaddress that received these TXOs
            let account_key: AccountKey = mc_util_serial::decode(&account.encrypted_account_key)?;
            let b58_subaddress = if subaddress_index >= 0 {
                let subaddress = account_key.subaddress(subaddress_index as u64);
                b58_encode(&subaddress)?
            } else {
                // If not matched to an existing subaddress, empty string as NULL
                "".to_string()
            };

            // Create a TransactionLogs entry
            let new_transaction_log = NewTransactionLog {
                transaction_id_hex: &transaction_id.to_string(),
                account_id_hex: &account.account_id_hex,
                recipient_public_address_b58: "", // NULL for received
                assigned_subaddress_b58: &b58_subaddress,
                value: transaction_value,
                fee: None, // Impossible to recover fee from received transaction
                status: "succeeded",
                sent_time: "", // NULL for received
                block_height: block_height as i64,
                comment: "", // NULL for received
                direction: "received",
                tx: None, // NULL for received
            };

            diesel::insert_into(schema_transaction_logs::table)
                .values(&new_transaction_log)
                .execute(&conn)?;

            // Create an entry per TXO for the TransactionTxoTypes
            for txo in txos {
                let new_transaction_txo = NewTransactionTxoType {
                    transaction_id_hex: &transaction_id.to_string(),
                    txo_id_hex: &txo.txo_id_hex,
                    transaction_txo_type: "output",
                };
                diesel::insert_into(schema_transaction_txo_types::table)
                    .values(&new_transaction_txo)
                    .execute(&conn)?;
            }
        }

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

        // Verify that the TxProposal is well-formed according to our assumptions about
        // how to store the sent data in our wallet.
        if tx_proposal.tx.prefix.outputs.len() - 1 != tx_proposal.outlays.len() {
            return Err(WalletDbError::UnexpectedNumberOfChangeOutputs);
        }

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

                // FIXME: currently only have the proofs for outlays, not change - we likely don't
                //        need to prove to ourself that we sent that change.
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
                    // This is the change output. Note: there should only be one change output
                    // per transaction, based on how we construct transactions. If we change
                    // how we construct transactions, these assumptions will change, and should be
                    // reflected in the TxProposal.
                    // println!(
                    //     "\x1b[1;36m GOT CHANGE with value {:?}\x1b[0m",
                    //     tx_proposal.change_value
                    // );
                    (tx_proposal.change_value, None, None)
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

                let encoded_proof = proof
                    .map(|p| mc_util_serial::encode(&tx_proposal.outlay_confirmation_numbers[p]));

                // println!(
                //     "\x1b[1;33m SETTING VALUE FOR THIS OUTPUT TO {:?}\x1b[0m",
                //     value
                // );
                let new_txo = NewTxo {
                    txo_id_hex: &txo_id.to_string(),
                    value: value as i64,
                    target_key: &mc_util_serial::encode(&output.target_key),
                    public_key: &mc_util_serial::encode(&output.public_key),
                    e_fog_hint: &mc_util_serial::encode(&output.e_fog_hint),
                    txo: &mc_util_serial::encode(output),
                    subaddress_index: None, // Minted set subaddress_index to None. Once received, overrides.
                    key_image: None,        // Only the recipient can calculate the KeyImage
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
                    txo_status: "secreted", // We cannot track spent status for minted TXOs unless change
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
                assigned_subaddress_b58: "", // NULL for sent
                value: transaction_value as i64,
                fee: Some(tx_proposal.tx.prefix.fee as i64),
                status: "pending",
                sent_time: &Utc::now().to_string(),
                block_height: block_height as i64, // FIXME: is this going to do what we want? It's
                // submitted block height, but not necessarily when it hits the ledger - would we
                // update when we see a key_image from this transaction?
                comment: &comment,
                direction: "sent",
                tx: Some(mc_util_serial::encode(&tx_proposal.tx)),
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

    // FIXME: hack to get around current db access design
    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        Ok(self.pool.get()?)
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

    // FIXME: DRY - these will be refactored to live on the models
    /// Returns (Transaction, Inputs, Outputs, Change)
    pub fn get_transaction(
        &self,
        transaction_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(TransactionLog, Vec<String>, Vec<String>, Vec<String>), WalletDbError> {
        // FIXME: use group_by rather than the processing below:
        // https://docs.diesel.rs/diesel/associations/trait.GroupedBy.html
        let transaction_txos: Vec<(TransactionLog, TransactionTxoType)> =
            schema_transaction_logs::table
                .inner_join(
                    schema_transaction_txo_types::table.on(
                        schema_transaction_logs::transaction_id_hex
                            .eq(schema_transaction_txo_types::transaction_id_hex)
                            .and(
                                schema_transaction_logs::transaction_id_hex.eq(transaction_id_hex),
                            ),
                    ),
                )
                .select((
                    schema_transaction_logs::all_columns,
                    schema_transaction_txo_types::all_columns,
                ))
                .load(conn)?;

        let mut inputs: Vec<String> = Vec::new();
        let mut outputs: Vec<String> = Vec::new();
        let mut change: Vec<String> = Vec::new();

        let transaction = transaction_txos[0].0.clone();
        for (_transaction, transaction_txo_type) in transaction_txos {
            if transaction_txo_type.transaction_txo_type == "input" {
                inputs.push(transaction_txo_type.txo_id_hex);
            } else if transaction_txo_type.transaction_txo_type == "output" {
                outputs.push(transaction_txo_type.txo_id_hex);
            } else if transaction_txo_type.transaction_txo_type == "change" {
                change.push(transaction_txo_type.txo_id_hex);
            }
        }

        Ok((transaction, inputs, outputs, change))
    }
}
