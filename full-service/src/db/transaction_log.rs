// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Transaction model.

use chrono::Utc;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::tx::Tx;
use std::fmt;

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{
            Account, NewTransactionLog, NewTransactionTxoType, TransactionLog, TransactionTxoType,
            Txo, TXO_USED_AS_CHANGE, TXO_USED_AS_INPUT, TXO_USED_AS_OUTPUT, TX_DIRECTION_RECEIVED,
            TX_DIRECTION_SENT, TX_STATUS_BUILT, TX_STATUS_FAILED, TX_STATUS_PENDING,
            TX_STATUS_SUCCEEDED,
        },
        txo::{TxoID, TxoModel},
        WalletDbError,
    },
    util::b58::b58_encode_public_address,
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

impl fmt::Display for TransactionID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct AssociatedTxos {
    pub inputs: Vec<Txo>,
    pub outputs: Vec<Txo>,
    pub change: Vec<Txo>,
}

pub trait TransactionLogModel {
    /// Get a transaction log from the TransactionId.
    fn get(
        transaction_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Get all transaction logs for the given block index.
    fn get_all_for_block_index(
        block_index: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// Get all transaction logs ordered by finalized_block_index.
    fn get_all_ordered_by_block_index(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// Get the Txos associated with a given TransactionId, grouped according to
    /// their type.
    ///
    /// Returns:
    /// * AssoiatedTxos(inputs, outputs, change)
    fn get_associated_txos(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssociatedTxos, WalletDbError>;

    /// Select the TransactionLogs associated with a given TxoId.
    fn select_for_txo(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// List all TransactionLogs and their associated Txos for a given account.
    ///
    /// Returns:
    /// * Vec(TransactionLog, AssociatedTxos(inputs, outputs, change))
    fn list_all(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletDbError>;

    /// Update the transactions associated with a Txo for a given block index.
    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Log a received transaction.
    fn log_received(
        subaddress_to_output_txo_ids: &HashMap<i64, Vec<String>>,
        account: &Account,
        block_index: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

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
    fn log_submitted(
        tx_proposal: TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Remove all logs for an account
    fn delete_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
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
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::TransactionLogNotFound(
                transaction_id_hex.to_string(),
            )),
            Err(e) => Err(e.into()),
        }
    }

    fn get_all_for_block_index(
        block_index: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError> {
        use crate::db::schema::transaction_logs::{
            all_columns, dsl::transaction_logs, finalized_block_index,
        };

        let matches: Vec<TransactionLog> = transaction_logs
            .select(all_columns)
            .filter(finalized_block_index.eq(block_index as i64))
            .load::<TransactionLog>(conn)?;

        Ok(matches)
    }

    fn get_all_ordered_by_block_index(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError> {
        use crate::db::schema::transaction_logs::{
            all_columns, dsl::transaction_logs, finalized_block_index,
        };

        let matches = transaction_logs
            .select(all_columns)
            .order_by(finalized_block_index.asc())
            .load(conn)?;

        Ok(matches)
    }

    fn get_associated_txos(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssociatedTxos, WalletDbError> {
        use crate::db::schema::{transaction_txo_types, txos};

        // FIXME: WS-29 - use group_by rather than the processing below:
        // https://docs.diesel.rs/diesel/associations/trait.GroupedBy.html
        let transaction_txos: Vec<(TransactionTxoType, Txo)> = transaction_txo_types::table
            .inner_join(txos::table.on(transaction_txo_types::txo_id_hex.eq(txos::txo_id_hex)))
            .filter(transaction_txo_types::transaction_id_hex.eq(&self.transaction_id_hex))
            .select((transaction_txo_types::all_columns, txos::all_columns))
            .load(conn)?;

        let mut inputs: Vec<Txo> = Vec::new();
        let mut outputs: Vec<Txo> = Vec::new();
        let mut change: Vec<Txo> = Vec::new();

        for (transaction_txo_type, txo) in transaction_txos {
            match transaction_txo_type.transaction_txo_type.as_str() {
                TXO_USED_AS_INPUT => inputs.push(txo),
                TXO_USED_AS_OUTPUT => outputs.push(txo),
                TXO_USED_AS_CHANGE => change.push(txo),
                _ => {
                    return Err(WalletDbError::UnexpectedTransactionTxoType(
                        transaction_txo_type.transaction_txo_type,
                    ));
                }
            }
        }

        Ok(AssociatedTxos {
            inputs,
            outputs,
            change,
        })
    }

    fn select_for_txo(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types};

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
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types, txos};

        // Query for all transaction logs for the account, as well as associated txos.
        // This is accomplished via a double-join through the
        // transaction_txo_types table.
        // TODO: investigate simplifying the database structure around this.
        let transactions_query = transaction_logs::table
            .filter(transaction_logs::account_id_hex.eq(account_id_hex))
            .inner_join(transaction_txo_types::table.on(
                transaction_logs::transaction_id_hex.eq(transaction_txo_types::transaction_id_hex),
            ))
            .inner_join(txos::table.on(transaction_txo_types::txo_id_hex.eq(txos::txo_id_hex)))
            .select((
                transaction_logs::all_columns,
                transaction_txo_types::all_columns,
                txos::all_columns,
            ))
            .order(transaction_logs::id);

        let transactions: Vec<(TransactionLog, TransactionTxoType, Txo)> =
            if let (Some(o), Some(l)) = (offset, limit) {
                transactions_query.offset(o).limit(l).load(conn)?
            } else {
                transactions_query.load(conn)?
            };

        #[derive(Clone)]
        struct TransactionContents {
            transaction_log: TransactionLog,
            inputs: Vec<Txo>,
            outputs: Vec<Txo>,
            change: Vec<Txo>,
        }
        let mut results: HashMap<String, TransactionContents> = HashMap::default();
        for (transaction, transaction_txo_type, txo) in transactions {
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

            let entry = results.get_mut(&transaction.transaction_id_hex).unwrap();

            if entry.transaction_log != transaction {
                return Err(WalletDbError::TransactionMismatch);
            }

            match transaction_txo_type.transaction_txo_type.as_str() {
                TXO_USED_AS_INPUT => entry.inputs.push(txo),
                TXO_USED_AS_OUTPUT => entry.outputs.push(txo),
                TXO_USED_AS_CHANGE => entry.change.push(txo),
                _ => {
                    return Err(WalletDbError::UnexpectedTransactionTxoType(
                        transaction_txo_type.transaction_txo_type,
                    ));
                }
            }
        }

        let mut results: Vec<(TransactionLog, AssociatedTxos)> = results
            .values()
            .cloned()
            .map(|t| {
                (
                    t.transaction_log,
                    AssociatedTxos {
                        inputs: t.inputs,
                        outputs: t.outputs,
                        change: t.change,
                    },
                )
            })
            .collect();

        results.sort_by_key(|r| r.0.id);
        Ok(results)
    }

    // FIXME: WS-30 - We may be doing n^2 work here
    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_logs::dsl::{transaction_id_hex, transaction_logs};

        let associated_transaction_logs = Self::select_for_txo(txo_id_hex, conn)?;

        for transaction_log in associated_transaction_logs {
            let associated = transaction_log.get_associated_txos(conn)?;

            // Only update transaction_log status if built or pending
            if transaction_log.status != TX_STATUS_BUILT
                && transaction_log.status != TX_STATUS_PENDING
            {
                continue;
            }

            // Check whether all the inputs have been spent or if any failed, and update
            // accordingly
            let input_txo_ids: Vec<String> = associated
                .inputs
                .iter()
                .map(|t| t.txo_id_hex.clone())
                .collect();
            if Txo::are_all_spent(&input_txo_ids, conn)? {
                diesel::update(
                    transaction_logs
                        .filter(transaction_id_hex.eq(&transaction_log.transaction_id_hex)),
                )
                .set((
                    crate::db::schema::transaction_logs::status.eq(TX_STATUS_SUCCEEDED),
                    crate::db::schema::transaction_logs::finalized_block_index
                        .eq(Some(cur_block_index)),
                ))
                .execute(conn)?;
            } else if Txo::any_failed(&input_txo_ids, cur_block_index, conn)? {
                // FIXME: WS-18, WS-17 - Do we want to store and update the "failed_block_index"
                // as min(tombstones)?
                diesel::update(
                    transaction_logs
                        .filter(transaction_id_hex.eq(&transaction_log.transaction_id_hex)),
                )
                .set(crate::db::schema::transaction_logs::status.eq(TX_STATUS_FAILED))
                .execute(conn)?;
            }
        }
        Ok(())
    }

    fn log_received(
        subaddress_to_output_txo_ids: &HashMap<i64, Vec<String>>,
        account: &Account,
        block_index: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_txo_types;

        for (subaddress_index, output_txo_ids) in subaddress_to_output_txo_ids {
            let txos = Txo::select_by_id(output_txo_ids, conn)?;
            for txo in txos {
                let transaction_id = TransactionID::from(txo.txo_id_hex.clone());

                // Check that we haven't already logged this transaction on a previous sync
                match TransactionLog::get(&transaction_id.to_string(), conn) {
                    Ok(_) => continue, // Processed this transaction on a previous sync.
                    Err(WalletDbError::TransactionLogNotFound(_)) => {} // Insert below
                    Err(e) => return Err(e),
                }

                // Get the public address for the subaddress that received these TXOs
                let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
                let subaddress = account_key.subaddress(*subaddress_index as u64);
                let b58_subaddress = b58_encode_public_address(&subaddress)?;
                let assigned_subaddress_b58: Option<&str> = if *subaddress_index >= 0 {
                    Some(&b58_subaddress)
                } else {
                    None
                };

                // Create a TransactionLogs entry for every TXO
                let new_transaction_log = NewTransactionLog {
                    transaction_id_hex: &transaction_id.to_string(),
                    account_id_hex: &account.account_id_hex,
                    assigned_subaddress_b58,
                    value: txo.value,
                    fee: None, // Impossible to recover fee from received transaction
                    status: TX_STATUS_SUCCEEDED,
                    sent_time: None, // NULL for received
                    submitted_block_index: None,
                    finalized_block_index: Some(block_index as i64),
                    comment: "", // NULL for received
                    direction: TX_DIRECTION_RECEIVED,
                    tx: None, // NULL for received
                };
                diesel::insert_into(crate::db::schema::transaction_logs::table)
                    .values(&new_transaction_log)
                    .execute(conn)?;

                // Create an entry per TXO for the TransactionTxoTypes
                let new_transaction_txo = NewTransactionTxoType {
                    transaction_id_hex: &transaction_id.to_string(),
                    txo_id_hex: &txo.txo_id_hex,
                    transaction_txo_type: TXO_USED_AS_OUTPUT,
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
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        // Store the txo_id_hex -> transaction_txo_type
        let mut txo_ids: Vec<(String, String)> = Vec::new();

        // Verify that the TxProposal is well-formed according to our assumptions about
        // how to store the sent data in our wallet (num_output_TXOs = num_outlays +
        // change_TXO).
        if tx_proposal.tx.prefix.outputs.len() - tx_proposal.outlays.len() > 1 {
            return Err(WalletDbError::UnexpectedNumberOfChangeOutputs);
        }

        // First update all inputs to "pending." They will remain pending until their
        // key_image hits the ledger or their tombstone block is exceeded.
        for utxo in tx_proposal.utxos.iter() {
            let txo_id = TxoID::from(&utxo.tx_out);
            let txo = Txo::get(&txo_id.to_string(), conn)?;
            txo.update_to_pending(tx_proposal.tx.prefix.tombstone_block as i64, conn)?;
            txo_ids.push((txo_id.to_string(), TXO_USED_AS_INPUT.to_string()));
        }

        // Next, add all of our minted outputs to the Txo Table
        for (i, output) in tx_proposal.tx.prefix.outputs.iter().enumerate() {
            let processed_output =
                Txo::create_minted(account_id_hex, output, &tx_proposal, i, conn)?;
            txo_ids.push((
                processed_output.txo_id_hex,
                processed_output.txo_type.to_string(),
            ));
        }

        // Enforce maximum value.
        let transaction_value = tx_proposal
            .outlays
            .iter()
            .map(|o| o.value as u128)
            .sum::<u128>();
        if transaction_value > u64::MAX as u128 {
            return Err(WalletDbError::TransactionValueExceedsMax);
        }

        let transaction_id = TransactionID::from(&tx_proposal.tx);
        let tx = mc_util_serial::encode(&tx_proposal.tx);

        // Create a TransactionLogs entry
        let new_transaction_log = NewTransactionLog {
            transaction_id_hex: &transaction_id.to_string(),
            account_id_hex, // Can be null if submitting an "unowned" proposal.
            assigned_subaddress_b58: None, // NULL for sent
            value: transaction_value as i64,
            fee: Some(tx_proposal.tx.prefix.fee as i64),
            status: TX_STATUS_PENDING,
            sent_time: Some(Utc::now().timestamp()),
            submitted_block_index: Some(block_index as i64),
            finalized_block_index: None,
            comment: &comment,
            direction: TX_DIRECTION_SENT,
            tx: Some(&tx),
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
        TransactionLog::get(&transaction_id.to_string(), conn)
    }

    fn delete_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{
            transaction_logs as cols, transaction_logs::dsl::transaction_logs,
            transaction_txo_types as types_cols, transaction_txo_types::dsl::transaction_txo_types,
        };

        let results: Vec<String> = transaction_logs
            .filter(cols::account_id_hex.eq(account_id_hex))
            .select(cols::transaction_id_hex)
            .load(conn)?;

        for transaction_id_hex in results.iter() {
            diesel::delete(
                transaction_txo_types.filter(types_cols::transaction_id_hex.eq(transaction_id_hex)),
            )
            .execute(conn)?;
        }

        diesel::delete(transaction_logs.filter(cols::account_id_hex.eq(account_id_hex)))
            .execute(conn)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{PublicAddress, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{constants::MINIMUM_FEE, ring_signature::KeyImage};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        db::account::{AccountID, AccountModel, DEFAULT_CHANGE_SUBADDRESS_INDEX},
        service::{sync::SyncThread, transaction_builder::WalletTransactionBuilder},
        test_utils::{
            add_block_with_tx_outs, builder_for_random_recipient, create_test_received_txo,
            get_resolver_factory, get_test_ledger, manually_sync_account,
            random_account_with_seed_values, WalletDbTestContext, MOB,
        },
    };

    use super::*;

    #[test_with_logger]
    fn test_log_received(logger: Logger) {
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
        let account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();

        // Populate our DB with some received txos in the same block
        let mut synced: HashMap<i64, Vec<String>> = HashMap::default();
        for i in 1..20 {
            let (txo_id_hex, _txo, _key_image) = create_test_received_txo(
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
            synced.get_mut(&0).unwrap().push(txo_id_hex);
        }

        // Now we'll ingest them.
        TransactionLog::log_received(&synced, &account, 144, &wallet_db.get_conn().unwrap())
            .unwrap();

        for (_subaddress, txos) in synced.iter() {
            for txo_id_hex in txos {
                let transaction_logs =
                    TransactionLog::select_for_txo(txo_id_hex, &wallet_db.get_conn().unwrap())
                        .unwrap();
                // There should be one TransactionLog per received txo
                assert_eq!(transaction_logs.len(), 1);

                assert_eq!(&transaction_logs[0].transaction_id_hex, txo_id_hex);

                let txo_details = Txo::get(txo_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
                assert_eq!(transaction_logs[0].value, txo_details.value);

                // Make the sure the types are correct - all received should be TXO_OUTPUT
                let associated = transaction_logs[0]
                    .get_associated_txos(&wallet_db.get_conn().unwrap())
                    .unwrap();
                assert_eq!(associated.inputs.len(), 0);
                assert_eq!(associated.outputs.len(), 1);
                assert_eq!(associated.change.len(), 0);
            }
        }
    }

    #[test_with_logger]
    // Test the happy path for log_submitted. When a transaction is submitted to the
    // MobileCoin network, several things must happen for Full-Service to
    // maintain accurate transaction history.
    //
    // 1. The minted TXO(s) were created in the txos table
    // 2. The spent TXO(s) are marked as pending
    // 3. The change TXO(s) are marked as minted, secreted
    // 4. The transaction_log is created and added to the transaction_log table
    // 5. Once the change is received, it is marked as minted, unspent
    fn test_log_submitted(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64],
            &mut rng,
        );

        // Build a transaction
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);
        builder
            .add_recipient(recipient.clone(), 50 * MOB as u64)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(None).unwrap();
        let tx_proposal = builder.build().unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // The log's account ID matches the account_id which submitted the tx
        assert_eq!(
            tx_log.account_id_hex,
            AccountID::from(&account_key).to_string()
        );
        // No assigned subaddress for sent
        assert_eq!(tx_log.assigned_subaddress_b58, None);
        // Value is the amount sent, not including fee and change
        assert_eq!(tx_log.value, 50 * MOB);
        // Fee exists for submitted
        assert_eq!(tx_log.fee, Some(MINIMUM_FEE as i64));
        // Created and sent transaction is "pending" until it lands
        assert_eq!(tx_log.status, TX_STATUS_PENDING);
        assert!(tx_log.sent_time.unwrap() > 0);
        assert_eq!(
            tx_log.submitted_block_index,
            Some(ledger_db.num_blocks().unwrap() as i64)
        );
        // There is no comment for this submission
        assert_eq!(tx_log.comment, "");
        // Tx direction is "sent"
        assert_eq!(tx_log.direction, TX_DIRECTION_SENT);

        // The tx in the log matches the tx in the proposal
        let tx: Tx = mc_util_serial::decode(&tx_log.clone().tx.unwrap()).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Check the associated_txos for this transaction_log are as expected
        let associated_txos = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        // There is one associated input TXO to this transaction, and it is now pending.
        assert_eq!(associated_txos.inputs.len(), 1);
        let input_details = Txo::get(
            &associated_txos.inputs[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details.value, 70 * MOB);
        assert!(input_details.is_pending()); // Should now be pending
        assert!(input_details.is_received());
        assert_eq!(input_details.subaddress_index.unwrap(), 0);
        assert!(!input_details.is_minted());

        // There is one associated output TXO to this transaction, and its recipient
        // is the destination addr
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].recipient_public_address_b58,
            b58_encode_public_address(&recipient).unwrap()
        );
        let output_details = Txo::get(
            &associated_txos.outputs[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(output_details.value, 50 * MOB);

        // We cannot know any details about the received_to_account for this TXO, as it
        // was sent out of the wallet
        assert!(output_details.is_minted());
        assert!(!output_details.is_received());
        assert!(output_details.subaddress_index.is_none());

        // Assert change is as expected
        assert_eq!(associated_txos.change.len(), 1);
        let change_details = Txo::get(
            &associated_txos.change[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(change_details.value, 20 * MOB - MINIMUM_FEE as i64);

        // Note, this will still be marked as not change until the txo
        // appears on the ledger and the account syncs.
        // change becomes unspent once scanned
        assert!(change_details.is_minted());
        assert!(!change_details.is_received());
        assert!(change_details.subaddress_index.is_none()); // this gets filled once scanned

        // Now - we will add the change TXO to the ledger, so we can scan and verify
        add_block_with_tx_outs(
            &mut ledger_db,
            &[mc_util_serial::decode(&change_details.txo).unwrap()],
            &[KeyImage::from(rng.next_u64())],
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
        let _sync = manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(tx_log.account_id_hex.to_string()),
            14,
            &logger,
        );

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert!(updated_change_details.is_minted());
        assert!(updated_change_details.is_unspent());
        assert_eq!(
            updated_change_details.received_account_id_hex.unwrap(),
            tx_log.account_id_hex
        );
        assert_eq!(
            updated_change_details.subaddress_index,
            Some(DEFAULT_CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    #[test_with_logger]
    fn test_log_submitted_no_change(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![100 * MOB as u64, 200 * MOB as u64],
            &mut rng,
        );

        // Build a transaction
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);
        // Add outlays all to the same recipient, so that we exceed u64::MAX in this tx
        let value = 100 * MOB as u64 - MINIMUM_FEE;
        builder.add_recipient(recipient.clone(), value).unwrap();

        builder.set_tombstone(0).unwrap();
        builder.select_txos(None).unwrap();
        let tx_proposal = builder.build().unwrap();

        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(
            tx_log.account_id_hex,
            AccountID::from(&account_key).to_string()
        );
        let associated_txos = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].recipient_public_address_b58,
            b58_encode_public_address(&recipient).unwrap()
        );
        // No assigned subaddress for sent
        assert_eq!(tx_log.assigned_subaddress_b58, None);
        // Value is the amount sent, not including fee and change
        assert_eq!(tx_log.value, value as i64);
        // Fee exists for submitted
        assert_eq!(tx_log.fee, Some(MINIMUM_FEE as i64));
        // Created and sent transaction is "pending" until it lands
        assert_eq!(tx_log.status, TX_STATUS_PENDING);
        assert!(tx_log.sent_time.unwrap() > 0);
        assert_eq!(
            tx_log.submitted_block_index,
            Some(ledger_db.num_blocks().unwrap() as i64)
        );
        assert_eq!(tx_log.comment, "");
        assert_eq!(tx_log.direction, TX_DIRECTION_SENT);
        let tx: Tx = mc_util_serial::decode(&tx_log.clone().tx.unwrap()).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Get associated Txos
        let associated = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();
        assert_eq!(associated.inputs.len(), 1);
        assert_eq!(associated.outputs.len(), 1);
        assert_eq!(associated.change.len(), 0);
    }

    #[test_with_logger]
    fn test_delete_transaction_logs_for_account(logger: Logger) {
        use crate::db::schema::{transaction_logs, transaction_txo_types};
        use diesel::dsl::count_star;

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());

        // Populate our DB with some received txos in the same block.
        // Do this for two different accounts.
        let mut account_ids: Vec<AccountID> = Vec::new();
        for _ in 0..2 {
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
            let account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();

            let mut synced: HashMap<i64, Vec<String>> = HashMap::default();
            for i in 1..=10 {
                let (txo_id_hex, _txo, _key_image) = create_test_received_txo(
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
                synced.get_mut(&0).unwrap().push(txo_id_hex);
            }

            // Ingest relevant txos.
            TransactionLog::log_received(&synced, &account, 144, &wallet_db.get_conn().unwrap())
                .unwrap();

            account_ids.push(account_id);
        }

        // Check that we created transaction_logs and transaction_txo_types entries.
        assert_eq!(
            Ok(20),
            transaction_logs::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
        assert_eq!(
            Ok(20),
            transaction_txo_types::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );

        // Delete the transaction logs for one account.
        let result = TransactionLog::delete_all_for_account(
            &account_ids[0].to_string(),
            &wallet_db.get_conn().unwrap(),
        );
        assert!(result.is_ok());

        // For the given account, the transaction logs and the txo types are deleted.
        assert_eq!(
            Ok(10),
            transaction_logs::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
        assert_eq!(
            Ok(10),
            transaction_txo_types::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
    }

    // Test that transaction logging can handle submitting a value greater than
    // i64::Max Note: i64::Max is 9_223_372_036_854_775_807, or about 9.2M MOB.
    // The biggest MOB amount that can be represented on chain is u64::MAX,
    // 18_446_744_073_709_551_615, or about 18M MOB.
    //
    // This test confirms that submitting a transaction_log for < u64::Max, but >
    // i64::Max succeeds
    #[test_with_logger]
    fn test_log_submitted_big_int(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![7_000_000 * MOB as u64, 14_000_000 * MOB as u64],
            &mut rng,
        );

        // Build a transaction for > i64::Max
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);
        builder
            .add_recipient(recipient.clone(), 10_000_000 * MOB as u64)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(None).unwrap();
        let tx_proposal = builder.build().unwrap();

        assert_eq!(tx_proposal.outlays[0].value, 10_000_000_000_000_000_000);

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(tx_log.value as u64, 10_000_000 * MOB as u64);
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
    #[test_with_logger]
    fn test_log_submitted_to_self(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![7 * MOB as u64, 8 * MOB as u64],
            &mut rng,
        );

        let mut builder = WalletTransactionBuilder::new(
            AccountID::from(&account_key).to_string(),
            wallet_db.clone(),
            ledger_db.clone(),
            get_resolver_factory(&mut rng).unwrap(),
            logger.clone(),
        );
        // Add self at main subaddress as the recipient
        builder
            .add_recipient(account_key.subaddress(0), 12 * MOB as u64)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(None).unwrap();
        let tx_proposal = builder.build().unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Get the associated txos for this transaction
        let associated_txos = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        // There are two input TXOs to this transaction, and they are both now pending.
        assert_eq!(associated_txos.inputs.len(), 2);
        let input_details0 = Txo::get(
            &associated_txos.inputs[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details0.value, 7 * MOB);

        assert!(input_details0.is_pending());
        assert!(input_details0.is_received());
        assert_eq!(input_details0.subaddress_index, Some(0 as i64));
        assert!(!input_details0.is_minted());

        let input_details1 = Txo::get(
            &associated_txos.inputs[1].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details1.value, 8 * MOB);

        assert!(input_details1.is_pending());
        assert!(input_details1.is_received());
        assert_eq!(input_details1.subaddress_index, Some(0 as i64));
        assert!(!input_details1.is_minted());

        // There is one associated output TXO to this transaction, and its recipient
        // is our own address
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].recipient_public_address_b58,
            b58_encode_public_address(&account_key.subaddress(0)).unwrap()
        );
        let output_details = Txo::get(
            &associated_txos.outputs[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(output_details.value, 12 * MOB);

        // The output type is "minted"
        assert!(output_details.is_minted());
        // We cannot know any details about the received_to_account for this TXO (until
        // it is scanned)
        assert!(!output_details.is_received());
        assert!(output_details.subaddress_index.is_none());

        // Assert change is as expected
        assert_eq!(associated_txos.change.len(), 1);
        let change_details = Txo::get(
            &associated_txos.change[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        // Change = (8 + 7) - 12 - fee
        assert_eq!(change_details.value, 3 * MOB - MINIMUM_FEE as i64);
        assert!(change_details.is_minted());
        assert!(!change_details.is_received());
        assert!(change_details.subaddress_index.is_none());

        // Now - we will add the spent Txos, outputs, and change to the ledger, so we
        // can scan and verify
        add_block_with_tx_outs(
            &mut ledger_db,
            &[
                mc_util_serial::decode(&change_details.txo).unwrap(),
                mc_util_serial::decode(&output_details.txo).unwrap(),
            ],
            &[
                mc_util_serial::decode(&input_details0.key_image.unwrap()).unwrap(),
                mc_util_serial::decode(&input_details1.key_image.unwrap()).unwrap(),
            ],
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 15);
        let _sync = manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(tx_log.account_id_hex.to_string()),
            15,
            &logger,
        );

        // Get the Input Txos again
        let updated_input_details0 = Txo::get(
            &associated_txos.inputs[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let updated_input_details1 = Txo::get(
            &associated_txos.inputs[1].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // We cannot know where these inputs were minted from (unless we had sent them
        // to ourselves, which we did not for this test). The outputs were sent
        // to ourselves, so will be testing that case, in "output" form.
        assert!(!updated_input_details0.is_minted());
        assert!(!updated_input_details1.is_minted());

        // The inputs are now spent
        assert!(updated_input_details0.is_spent());
        assert!(updated_input_details1.is_spent());

        // The received_to account is ourself, which is the same as the account
        // account_id_hex in the transaction log. The type is "Received"
        assert_eq!(
            updated_input_details0.received_account_id_hex,
            Some(tx_log.account_id_hex.clone())
        );
        assert!(updated_input_details0.is_received());
        assert_eq!(updated_input_details0.subaddress_index, Some(0 as i64));

        assert_eq!(
            updated_input_details1.received_account_id_hex,
            Some(tx_log.account_id_hex.clone())
        );
        assert!(updated_input_details1.is_received());
        assert_eq!(updated_input_details1.subaddress_index, Some(0 as i64));

        // Get the output txo again
        let updated_output_details = Txo::get(
            &associated_txos.outputs[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        // The minted from account is ourself, and it is unspent, minted
        assert!(updated_output_details.is_unspent());
        assert!(updated_output_details.is_minted());

        // The received to account is ourself, and it is unspent, minted
        assert_eq!(
            updated_output_details.received_account_id_hex,
            Some(tx_log.account_id_hex.clone())
        );

        // Received to main subaddress
        assert_eq!(updated_output_details.subaddress_index, Some(0 as i64));

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert!(updated_change_details.is_unspent());
        assert!(updated_change_details.is_minted());
        assert_eq!(
            updated_change_details.received_account_id_hex,
            Some(tx_log.account_id_hex)
        );
        assert_eq!(
            updated_change_details.subaddress_index,
            Some(DEFAULT_CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    // FIXME: test_log_submitted for recovered
    // FIXME: test_log_submitted offline flow
}
