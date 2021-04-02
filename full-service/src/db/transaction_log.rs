// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Transaction model.

use crate::db::{
    account::{AccountID, AccountModel},
    b58_encode,
    models::{
        Account, NewTransactionLog, NewTransactionTxoType, TransactionLog, TransactionTxoType, Txo,
        TXO_USED_AS_CHANGE, TXO_USED_AS_INPUT, TXO_USED_AS_OUTPUT, TX_DIRECTION_RECEIVED,
        TX_DIRECTION_SENT, TX_STATUS_BUILT, TX_STATUS_FAILED, TX_STATUS_PENDING,
        TX_STATUS_SUCCEEDED,
    },
    txo::{TxoID, TxoModel},
};

use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::tx::Tx;

use crate::db::WalletDbError;
use chrono::Utc;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use std::fmt;

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
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub change: Vec<String>,
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
        account_id_hex: Option<&str>,
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
        use crate::db::schema::{transaction_logs, transaction_txo_types};

        // FIXME: WS-29 - use group_by rather than the processing below:
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
            match transaction_txo_type.transaction_txo_type.as_str() {
                TXO_USED_AS_INPUT => inputs.push(transaction_txo_type.txo_id_hex),
                TXO_USED_AS_OUTPUT => outputs.push(transaction_txo_type.txo_id_hex),
                TXO_USED_AS_CHANGE => change.push(transaction_txo_type.txo_id_hex),
                _ => {
                    return Err(WalletDbError::UnexpectedTransactionTxoType(
                        transaction_txo_type.transaction_txo_type,
                    ))
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
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types};

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

            let entry = results.get_mut(&transaction.transaction_id_hex).unwrap();

            if entry.transaction_log != transaction {
                return Err(WalletDbError::TransactionMismatch);
            }

            match transaction_txo_type.transaction_txo_type.as_str() {
                TXO_USED_AS_INPUT => entry.inputs.push(transaction_txo_type.txo_id_hex),
                TXO_USED_AS_OUTPUT => entry.outputs.push(transaction_txo_type.txo_id_hex),
                TXO_USED_AS_CHANGE => entry.change.push(transaction_txo_type.txo_id_hex),
                _ => {
                    return Err(WalletDbError::UnexpectedTransactionTxoType(
                        transaction_txo_type.transaction_txo_type,
                    ))
                }
            }
        }
        Ok(results
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
            .collect())
    }

    // FIXME: WS-30 - We may be doing n^2 work here
    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_logs::dsl::{transaction_id_hex, transaction_logs};

        Ok(conn.transaction::<(), WalletDbError, _>(|| {
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
                if Txo::are_all_spent(&associated.inputs, conn)? {
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
                } else if Txo::any_failed(&associated.inputs, cur_block_index, conn)? {
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
        })?)
    }

    fn log_received(
        subaddress_to_output_txo_ids: &HashMap<i64, Vec<String>>,
        account: &Account,
        block_index: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_txo_types;

        Ok(conn.transaction::<(), WalletDbError, _>(|| {
            for (subaddress_index, output_txo_ids) in subaddress_to_output_txo_ids {
                let txos = Txo::select_by_id(&output_txo_ids, conn)?;
                for (txo, _account_txo_status) in txos {
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
                    let b58_subaddress = b58_encode(&subaddress)?;
                    let assigned_subaddress_b58: Option<&str> = if *subaddress_index >= 0 {
                        Some(&b58_subaddress)
                    } else {
                        None
                    };

                    // Create a TransactionLogs entry for every TXO
                    let new_transaction_log = NewTransactionLog {
                        transaction_id_hex: &transaction_id.to_string(),
                        account_id_hex: Some(&account.account_id_hex),
                        recipient_public_address_b58: "", // NULL for received
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
        })?)
    }

    fn log_submitted(
        tx_proposal: TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: Option<&str>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        if let Some(a_id) = account_id_hex {
            match Account::get(&AccountID(a_id.to_string()), &conn) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        let transaction_log_id = conn.transaction::<String, WalletDbError, _>(|| {
            // Store the txo_id_hex -> transaction_txo_type
            let mut txo_ids: Vec<(String, String)> = Vec::new();

            // Verify that the TxProposal is well-formed according to our assumptions about
            // how to store the sent data in our wallet.
            if tx_proposal.tx.prefix.outputs.len() - tx_proposal.outlays.len() > 1 {
                return Err(WalletDbError::UnexpectedNumberOfChangeOutputs);
            }

            // First update all inputs to "pending." They will remain pending until their
            // key_image hits the ledger or their tombstone block is exceeded.
            for utxo in tx_proposal.utxos.iter() {
                let txo_id = TxoID::from(&utxo.tx_out);
                Txo::update_to_pending(&txo_id, conn)?;
                txo_ids.push((txo_id.to_string(), TXO_USED_AS_INPUT.to_string()));
            }

            // Next, add all of our minted outputs to the Txo Table
            let recipient_address = {
                let mut recipient_address = None;
                for (i, output) in tx_proposal.tx.prefix.outputs.iter().enumerate() {
                    let processed_output =
                        Txo::create_minted(account_id_hex, &output, &tx_proposal, i, conn)?;

                    // Currently, the wallet enforces only one recipient per TransactionLog.
                    if let Some(found_recipient) = processed_output.recipient {
                        if let Some(cur_recipient) = recipient_address.clone() {
                            if found_recipient != cur_recipient {
                                return Err(WalletDbError::MultipleRecipientsInTransaction);
                            }
                        } else {
                            recipient_address = Some(found_recipient);
                        }
                    }

                    txo_ids.push((
                        processed_output.txo_id_hex,
                        processed_output.txo_type.to_string(),
                    ));
                }
                recipient_address
            };

            let transaction_value = tx_proposal
                .outlays
                .iter()
                .map(|o| o.value as u128)
                .sum::<u128>();
            if transaction_value > i64::MAX as u128 {
                return Err(WalletDbError::TransactionValueExceedsMax);
            }

            if let Some(recipient) = recipient_address {
                let transaction_id = TransactionID::from(&tx_proposal.tx);
                let tx = mc_util_serial::encode(&tx_proposal.tx);

                // Create a TransactionLogs entry
                let new_transaction_log = NewTransactionLog {
                    transaction_id_hex: &transaction_id.to_string(),
                    account_id_hex, // Can be null if submitting an "unowned" proposal.
                    recipient_public_address_b58: &b58_encode(&recipient)?,
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
                Ok(transaction_id.to_string())
            } else {
                Err(WalletDbError::TransactionLacksRecipient)
            }
        })?;
        Ok(TransactionLog::get(&transaction_log_id, conn)?)
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
    use super::*;
    use crate::{
        db::{
            account::{AccountID, AccountModel},
            models::{TXO_STATUS_PENDING, TXO_STATUS_SECRETED, TXO_TYPE_MINTED, TXO_TYPE_RECEIVED},
        },
        service::sync::SyncThread,
        test_utils::{
            builder_for_random_recipient, create_test_received_txo, get_test_ledger,
            random_account_with_seed_values, WalletDbTestContext, MOB,
        },
    };
    use mc_account_keys::{PublicAddress, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_transaction_core::constants::MINIMUM_FEE;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

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
                assert_eq!(transaction_logs[0].value, txo_details.txo.value);

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

        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            Some(&AccountID::from(&account_key).to_string()),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(
            tx_log.account_id_hex,
            Some(AccountID::from(&account_key).to_string())
        );
        assert_eq!(
            tx_log.recipient_public_address_b58,
            b58_encode(&recipient).unwrap()
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
        assert_eq!(tx_log.comment, "");
        assert_eq!(tx_log.direction, TX_DIRECTION_SENT);
        let tx: Tx = mc_util_serial::decode(&tx_log.clone().tx.unwrap()).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Get associated Txos
        let associated = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        // Assert inputs are as expected
        assert_eq!(associated.inputs.len(), 1);
        let input_details =
            Txo::get(&associated.inputs[0], &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(input_details.txo.value, 70 * MOB);
        assert_eq!(
            input_details
                .received_to_account
                .clone()
                .unwrap()
                .txo_status,
            TXO_STATUS_PENDING
        ); // Should now be pending
        assert_eq!(
            input_details.received_to_account.clone().unwrap().txo_type,
            TXO_TYPE_RECEIVED
        );
        assert_eq!(
            input_details
                .received_to_assigned_subaddress
                .unwrap()
                .subaddress_index,
            0
        );
        assert!(input_details.minted_from_account.is_none());

        // Assert outputs are as expected
        assert_eq!(associated.outputs.len(), 1);
        let output_details =
            Txo::get(&associated.outputs[0], &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(output_details.txo.value, 50 * MOB);
        assert_eq!(
            output_details
                .minted_from_account
                .clone()
                .unwrap()
                .txo_status,
            TXO_STATUS_SECRETED
        );
        assert_eq!(
            output_details.minted_from_account.clone().unwrap().txo_type,
            TXO_TYPE_MINTED
        );
        assert!(output_details.received_to_account.is_none());
        assert!(output_details.received_to_assigned_subaddress.is_none());

        // Assert change is as expected
        assert_eq!(associated.change.len(), 1);
        let change_details =
            Txo::get(&associated.change[0], &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(change_details.txo.value, 19990000000000); // 19.99 * MOB
        assert_eq!(
            change_details
                .minted_from_account
                .clone()
                .unwrap()
                .txo_status,
            TXO_STATUS_SECRETED
        ); // Note, change becomes "unspent" once scanned
        assert_eq!(
            change_details.minted_from_account.clone().unwrap().txo_type,
            TXO_TYPE_MINTED
        ); // Note, becomes "received" once scanned
        assert!(change_details.received_to_account.is_none()); // Note, gets filled in once scanned
        assert!(change_details.received_to_assigned_subaddress.is_none()); // Note, gets filled in once scanned

        // FIXME: add the change txo above to the ledger, and then scan and
        // verify the above statements
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
            Some(&AccountID::from(&account_key).to_string()),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(
            tx_log.account_id_hex,
            Some(AccountID::from(&account_key).to_string())
        );
        assert_eq!(
            tx_log.recipient_public_address_b58,
            b58_encode(&recipient).unwrap()
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

    // FIXME: WS-9 - test log_submitted for transaction value > i64::Max
    // FIXME: test_log_submitted to self and then scan
    // FIXME: test_log_submitted for recovered

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
}
