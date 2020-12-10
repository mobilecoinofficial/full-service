// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Transaction model.

use crate::{
    db::{
        b58_encode,
        models::{
            Account, NewTransactionLog, NewTransactionTxoType, TransactionLog, TransactionTxoType,
            Txo,
        },
        txo::{TxoID, TxoModel},
    },
    error::WalletDbError,
};

use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::tx::Tx;

use chrono::Utc;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};

#[derive(Debug)]
pub struct TransactionID(String);

// TransactionID is formed from the contents of the transaction when sent
impl From<&Tx> for TransactionID {
    fn from(src: &Tx) -> TransactionID {
        let temp: [u8; 32] = src.digest32::<MerlinTranscript>(b"transaction_data");
        Self(hex::encode(temp))
    }
}

// TransactionID is formed from the received TxoID when received
impl From<String> for TransactionID {
    fn from(src: String) -> TransactionID {
        Self(src)
    }
}

impl TransactionID {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub trait TransactionLogModel {
    /// Get a transaction log from the TransactionId.
    fn get(
        transaction_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Get the Txos associated with a given TransactionId, grouped according to their type.
    ///
    /// Returns:
    /// * (inputs, outputs, change)
    fn get_associated_txos(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(Vec<String>, Vec<String>, Vec<String>), WalletDbError>;

    /// Select the TransactionLogs associated with a given TxoId.
    fn select_for_txo(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// List all TransactionLogs and their associated Txos for a given account.
    ///
    /// Returns:
    /// * Vec(TransactionLog, inputs, outputs, change)
    fn list_all(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(TransactionLog, Vec<String>, Vec<String>, Vec<String>)>, WalletDbError>;

    /// Update the transactions associated with a Txo for a given blockheight.
    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Log a received transaction.
    fn log_received(
        subaddress_to_output_txo_ids: &HashMap<i64, Vec<String>>,
        account: &Account,
        block_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Log a submitted transaction.
    ///
    /// When submitting a transaction, we store relevant information to the transaction logs,
    /// and we also track information about each of the txos involved in the transaction.
    ///
    /// Note: We expect transactions created with this wallet to have one recipient, with the
    ///       rest of the minted txos designated as change. Other wallets may choose to behave
    ///       differently, but our TransactionLogs Table assumes this behavior.
    fn log_submitted(
        tx_proposal: TxProposal,
        block_height: u64,
        comment: String,
        account_id_hex: Option<&str>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;
}

impl TransactionLogModel for TransactionLog {
    fn get(
        transaction_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError> {
        use crate::db::schema::transaction_logs::dsl::{
            transaction_id_hex as dsl_transaction_id_hex, transaction_logs,
        };

        match transaction_logs
            .filter(dsl_transaction_id_hex.eq(transaction_id_hex))
            .get_result::<TransactionLog>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::NotFound(transaction_id_hex.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_associated_txos(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(Vec<String>, Vec<String>, Vec<String>), WalletDbError> {
        use crate::db::schema::transaction_logs;
        use crate::db::schema::transaction_txo_types;

        // FIXME: use group_by rather than the processing below:
        // https://docs.diesel.rs/diesel/associations/trait.GroupedBy.html
        let transaction_txos: Vec<(TransactionLog, TransactionTxoType)> = transaction_logs::table
            .inner_join(
                transaction_txo_types::table.on(transaction_logs::transaction_id_hex
                    .eq(transaction_txo_types::transaction_id_hex)
                    .and(transaction_logs::transaction_id_hex.eq(&self.transaction_id_hex))),
            )
            .select((
                transaction_logs::all_columns,
                transaction_txo_types::all_columns,
            ))
            .load(conn)?;

        let mut inputs: Vec<String> = Vec::new();
        let mut outputs: Vec<String> = Vec::new();
        let mut change: Vec<String> = Vec::new();

        for (_transaction, transaction_txo_type) in transaction_txos {
            if transaction_txo_type.transaction_txo_type == "input" {
                inputs.push(transaction_txo_type.txo_id_hex);
            } else if transaction_txo_type.transaction_txo_type == "output" {
                outputs.push(transaction_txo_type.txo_id_hex);
            } else if transaction_txo_type.transaction_txo_type == "change" {
                change.push(transaction_txo_type.txo_id_hex);
            }
        }

        Ok((inputs, outputs, change))
    }

    fn select_for_txo(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError> {
        use crate::db::schema::transaction_logs;
        use crate::db::schema::transaction_txo_types;

        Ok(transaction_logs::table
            .inner_join(
                transaction_txo_types::table.on(transaction_logs::transaction_id_hex
                    .eq(transaction_txo_types::transaction_id_hex)
                    .and(transaction_txo_types::txo_id_hex.eq(txo_id_hex))),
            )
            .select(transaction_logs::all_columns)
            .load(conn)?)
    }

    fn list_all(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(TransactionLog, Vec<String>, Vec<String>, Vec<String>)>, WalletDbError> {
        use crate::db::schema::transaction_logs;
        use crate::db::schema::transaction_txo_types;

        // FIXME: use group_by rather than the processing below:
        // https://docs.diesel.rs/diesel/associations/trait.GroupedBy.html
        let transactions: Vec<(TransactionLog, TransactionTxoType)> = transaction_logs::table
            .inner_join(
                transaction_txo_types::table.on(transaction_logs::transaction_id_hex
                    .eq(transaction_txo_types::transaction_id_hex)
                    .and(transaction_logs::account_id_hex.eq(account_id_hex))),
            )
            .select((
                transaction_logs::all_columns,
                transaction_txo_types::all_columns,
            ))
            .load(conn)?;

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

    // FIXME: We may be doing n^2 work here
    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_logs::dsl::{transaction_id_hex, transaction_logs};

        let associated_transaction_logs = Self::select_for_txo(txo_id_hex, conn)?;

        for transaction_log in associated_transaction_logs {
            let (inputs, _outputs, _change) = transaction_log.get_associated_txos(conn)?;

            // Only update transaction_log status if proposed or pending
            if transaction_log.status == "succeeded" || transaction_log.status == "failed" {
                continue;
            }

            // Check whether all the inputs have been spent or if any failed, and update accordingly
            if Txo::are_all_spent(&inputs, conn)? {
                // FIXME: do we want to store "submitted_block_height" to disambiguate block_height?
                diesel::update(
                    transaction_logs
                        .filter(transaction_id_hex.eq(&transaction_log.transaction_id_hex)),
                )
                .set((
                    crate::db::schema::transaction_logs::status.eq("succeeded"),
                    crate::db::schema::transaction_logs::block_height.eq(cur_block_height),
                ))
                .execute(conn)?;
            } else if Txo::any_failed(&inputs, cur_block_height, conn)? {
                // FIXME: Do we want to store and update the "failed_block_height" as min(tombstones)?
                diesel::update(
                    transaction_logs
                        .filter(transaction_id_hex.eq(&transaction_log.transaction_id_hex)),
                )
                .set(crate::db::schema::transaction_logs::status.eq("failed"))
                .execute(conn)?;
            }
        }

        Ok(())
    }

    fn log_received(
        subaddress_to_output_txo_ids: &HashMap<i64, Vec<String>>,
        account: &Account,
        block_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_txo_types;

        for (subaddress_index, output_txo_ids) in subaddress_to_output_txo_ids {
            let txos = Txo::select_by_id(&output_txo_ids, conn)?;
            for (txo, _account_txo_status) in txos {
                let transaction_id = TransactionID::from(txo.txo_id_hex.clone());

                // Check that we haven't already logged this transaction on a previous sync
                match TransactionLog::get(&transaction_id.to_string(), conn) {
                    Ok(_) => continue, // We've already processed this transaction on a previous sync
                    Err(WalletDbError::NotFound(_)) => {} // Insert below
                    Err(e) => return Err(e.into()),
                }

                // Get the public address for the subaddress that received these TXOs
                let account_key: AccountKey =
                    mc_util_serial::decode(&account.encrypted_account_key)?;
                let b58_subaddress = if *subaddress_index >= 0 {
                    let subaddress = account_key.subaddress(*subaddress_index as u64);
                    b58_encode(&subaddress)?
                } else {
                    // If not matched to an existing subaddress, empty string as NULL
                    "".to_string()
                };

                // Create a TransactionLogs entry for every TXO
                let new_transaction_log = NewTransactionLog {
                    transaction_id_hex: &transaction_id.to_string(),
                    account_id_hex: &account.account_id_hex,
                    recipient_public_address_b58: "", // NULL for received
                    assigned_subaddress_b58: &b58_subaddress,
                    value: txo.value,
                    fee: None, // Impossible to recover fee from received transaction
                    status: "succeeded",
                    sent_time: None, // NULL for received
                    block_height: block_height as i64,
                    comment: "", // NULL for received
                    direction: "received",
                    tx: None, // NULL for received
                };

                diesel::insert_into(crate::db::schema::transaction_logs::table)
                    .values(&new_transaction_log)
                    .execute(conn)?;

                // Create an entry per TXO for the TransactionTxoTypes
                let new_transaction_txo = NewTransactionTxoType {
                    transaction_id_hex: &transaction_id.to_string(),
                    txo_id_hex: &txo.txo_id_hex,
                    transaction_txo_type: "output",
                };
                // Note: SQLite backend does not support batch insert, so within iter is fine
                diesel::insert_into(transaction_txo_types::table)
                    .values(&new_transaction_txo)
                    .execute(conn)?;
            }
        }

        Ok(())
    }

    fn log_submitted(
        tx_proposal: TxProposal,
        block_height: u64,
        comment: String,
        account_id_hex: Option<&str>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        // Store the txo_id -> transaction_txo_type
        let mut txo_ids: Vec<(String, String)> = Vec::new();

        // Verify that the TxProposal is well-formed according to our assumptions about
        // how to store the sent data in our wallet.
        if tx_proposal.tx.prefix.outputs.len() - 1 != tx_proposal.outlays.len() {
            return Err(WalletDbError::UnexpectedNumberOfChangeOutputs);
        }

        // First update all inputs to "pending." They will remain pending until their key_image
        // hits the ledger.
        for utxo in tx_proposal.utxos.iter() {
            let txo_id = TxoID::from(&utxo.tx_out);
            Txo::update_to_pending(&txo_id, conn)?;
            txo_ids.push((txo_id.to_string(), "input".to_string()));
        }

        // Next, add all of our minted outputs to the Txo Table
        let (recipient_address, transaction_value) = {
            let mut recipient_address = None;
            let mut value_sum = 0;
            for (i, output) in tx_proposal.tx.prefix.outputs.iter().enumerate() {
                let (output_recipient, txo_id, output_value, transaction_txo_type) =
                    Txo::create_minted(account_id_hex, &output, &tx_proposal, i, conn)?;

                // Currently, the wallet enforces only one recipient per TransactionLog.
                if let Some(found_recipient) = output_recipient {
                    if let Some(cur_recipient) = recipient_address.clone() {
                        if found_recipient != cur_recipient {
                            return Err(WalletDbError::MultipleRecipientsInTransaction);
                        }
                    } else {
                        recipient_address = Some(found_recipient);
                    }
                }

                value_sum += output_value;
                txo_ids.push((txo_id, transaction_txo_type.to_string()));
            }
            (recipient_address, value_sum)
        };

        if let Some(recipient) = recipient_address {
            let transaction_id = TransactionID::from(&tx_proposal.tx);
            // Create a TransactionLogs entry
            let new_transaction_log = NewTransactionLog {
                transaction_id_hex: &transaction_id.to_string(),
                account_id_hex: &account_id_hex.unwrap_or(""), // Can be empty str if submitting an "unowned" proposal
                recipient_public_address_b58: &b58_encode(&recipient)?,
                assigned_subaddress_b58: "", // NULL for sent
                value: transaction_value as i64,
                fee: Some(tx_proposal.tx.prefix.fee as i64),
                status: "pending",
                sent_time: Some(Utc::now().timestamp()),
                block_height: block_height as i64, // FIXME: is this going to do what we want? It's
                // submitted block height, but not necessarily when it hits the ledger - would we
                // update when we see a key_image from this transaction?
                comment: &comment,
                direction: "sent",
                tx: Some(mc_util_serial::encode(&tx_proposal.tx)),
            };

            diesel::insert_into(crate::db::schema::transaction_logs::table)
                .values(&new_transaction_log)
                .execute(conn)?;

            // Create an entry per TXO for the TransactionTxoTypes
            for (txo_id_hex, transaction_txo_type) in txo_ids {
                let new_transaction_txo = NewTransactionTxoType {
                    transaction_id_hex: &transaction_id.to_string(),
                    txo_id_hex: &txo_id_hex,
                    transaction_txo_type: &transaction_txo_type,
                };
                diesel::insert_into(crate::db::schema::transaction_txo_types::table)
                    .values(&new_transaction_txo)
                    .execute(conn)?;
            }

            Ok(transaction_id.to_string())
        } else {
            Err(WalletDbError::TransactionLacksRecipient)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::account::AccountModel,
        test_utils::{create_test_received_txo, WalletDbTestContext, MOB},
    };
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_log_received(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);

        // Populate our DB with some received txos in the same block
        let mut synced: HashMap<i64, Vec<String>> = HashMap::default();
        for i in 1..20 {
            let (txo_hex, _txo, _key_image) = create_test_received_txo(
                &account_key,
                0, // All to the same subaddress
                (100 * i * MOB) as u64,
                144,
                &mut rng,
                &wallet_db,
            );
            if synced.is_empty() {
                synced.insert(0, Vec::new());
            }
            synced.get_mut(&0).unwrap().push(txo_hex);
        }

        // Now we'll ingest them.
        let (account_id, _address) = Account::create(
            &account_key,
            0,
            1,
            2,
            0,
            1,
            "",
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();
        TransactionLog::log_received(&synced, &account, 144, &wallet_db.get_conn().unwrap())
            .unwrap();

        for (_subaddress, txos) in synced.iter() {
            for txo_id in txos {
                let transaction_logs =
                    TransactionLog::select_for_txo(txo_id, &wallet_db.get_conn().unwrap()).unwrap();
                // There should be one TransactionLog per received txo
                assert_eq!(transaction_logs.len(), 1);

                assert_eq!(&transaction_logs[0].transaction_id_hex, txo_id);

                let (txo, _status, _assigned_subaddress) =
                    Txo::get(&account_id, txo_id, &wallet_db.get_conn().unwrap()).unwrap();
                assert_eq!(transaction_logs[0].value, txo.value);

                // Make the sure the types are correct - all received should be "output"
                let (inputs, outputs, change) = transaction_logs[0]
                    .get_associated_txos(&wallet_db.get_conn().unwrap())
                    .unwrap();
                assert_eq!(inputs.len(), 0);
                assert_eq!(outputs.len(), 1);
                assert_eq!(change.len(), 0);
            }
        }
    }

    // $[test_with_logger]
    // fn test_log_minted(logger: Logger) {
    //     let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
    //
    //     let db_test_context = WalletDbTestContext::default();
    //     let wallet_db = db_test_context.get_db_instance(logger);
    //
    //
    //     // Random recipient
    //     let recipient = Accountkey::from(RootIdentity::from_random(rng))
    //         .account_kehy
    //         .subaddress(rng.next_u64());
    // }
}
