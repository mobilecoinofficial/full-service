// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Transaction model.

use chrono::Utc;
use diesel::prelude::*;
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{tx::Tx, Amount, TokenId};
use std::fmt;

use crate::db::{
    account::{AccountID, AccountModel},
    models::{
        Account, NewTransactionLog, NewTransactionTxoType, TransactionLog, TransactionTxoType, Txo,
        TXO_USED_AS_CHANGE, TXO_USED_AS_INPUT, TXO_USED_AS_OUTPUT,
    },
    txo::{TxoID, TxoModel},
    Conn, WalletDbError,
};

#[derive(Debug)]
pub struct TransactionID(pub String);

// TransactionID is formed from the contents of the transaction when sent
impl From<&Tx> for TransactionID {
    fn from(src: &Tx) -> TransactionID {
        let temp: [u8; 32] = src.digest32::<MerlinTranscript>(b"transaction_data");
        Self(hex::encode(temp))
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

#[derive(Debug, PartialEq)]
pub enum TxStatus {
    Built,
    Pending,
    Succeeded,
    Failed,
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxStatus::Built => write!(f, "built"),
            TxStatus::Pending => write!(f, "pending"),
            TxStatus::Succeeded => write!(f, "succeeded"),
            TxStatus::Failed => write!(f, "failed"),
        }
    }
}

pub trait TransactionLogModel {
    /// Get a transaction log from the TransactionId.
    fn get(id: &TransactionID, conn: &Conn) -> Result<TransactionLog, WalletDbError>;

    /// Get all transaction logs for the given block index.
    fn get_all_for_block_index(
        block_index: u64,
        conn: &Conn,
    ) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// Get all transaction logs ordered by finalized_block_index.
    fn get_all_ordered_by_block_index(conn: &Conn) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// Get the Txos associated with a given TransactionId, grouped according to
    /// their type.
    ///
    /// Returns:
    /// * AssoiatedTxos(inputs, outputs, change)
    fn get_associated_txos(&self, conn: &Conn) -> Result<AssociatedTxos, WalletDbError>;

    /// Select the TransactionLogs associated with a given TxoId.
    fn select_for_txo(txo_id_hex: &str, conn: &Conn) -> Result<Vec<TransactionLog>, WalletDbError>;

    /// List all TransactionLogs and their associated Txos for a given account.
    ///
    /// Returns:
    /// * Vec(TransactionLog, AssociatedTxos(inputs, outputs, change))
    fn list_all(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletDbError>;

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
        conn: &Conn,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Remove all logs for an account
    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError>;

    fn update_tx_logs_associated_with_txo_to_succeeded(
        txo_id_hex: &str,
        finalized_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn update_tx_logs_associated_with_txos_to_failed(
        txos: &[Txo],
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn status(&self) -> TxStatus;
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

    fn get(id: &TransactionID, conn: &Conn) -> Result<TransactionLog, WalletDbError> {
        use crate::db::schema::transaction_logs::dsl::{id as dsl_id, transaction_logs};

        match transaction_logs
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

    fn get_all_for_block_index(
        block_index: u64,
        conn: &Conn,
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

    fn get_all_ordered_by_block_index(conn: &Conn) -> Result<Vec<TransactionLog>, WalletDbError> {
        use crate::db::schema::transaction_logs::{
            all_columns, dsl::transaction_logs, finalized_block_index,
        };

        let matches = transaction_logs
            .select(all_columns)
            .order_by(finalized_block_index.asc())
            .load(conn)?;

        Ok(matches)
    }

    fn get_associated_txos(&self, conn: &Conn) -> Result<AssociatedTxos, WalletDbError> {
        use crate::db::schema::{transaction_txo_types, txos};

        // FIXME: WS-29 - use group_by rather than the processing below:
        // https://docs.diesel.rs/diesel/associations/trait.GroupedBy.html
        let transaction_txos: Vec<(TransactionTxoType, Txo)> = transaction_txo_types::table
            .inner_join(txos::table.on(transaction_txo_types::txo_id_hex.eq(txos::txo_id_hex)))
            .filter(transaction_txo_types::transaction_log_id.eq(&self.id))
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

    fn select_for_txo(txo_id_hex: &str, conn: &Conn) -> Result<Vec<TransactionLog>, WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types};

        Ok(transaction_logs::table
            .inner_join(
                transaction_txo_types::table
                    .on(transaction_logs::id.eq(transaction_txo_types::transaction_log_id)),
            )
            .filter(transaction_txo_types::txo_id_hex.eq(txo_id_hex))
            .select(transaction_logs::all_columns)
            .load(conn)?)
    }

    fn list_all(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types, txos};

        // Query for all transaction logs for the account, as well as associated txos.
        // This is accomplished via a double-join through the
        // transaction_txo_types table.
        // TODO: investigate simplifying the database structure around this.
        let mut transactions_query = transaction_logs::table
            .into_boxed()
            .filter(transaction_logs::account_id_hex.eq(account_id_hex))
            .inner_join(
                transaction_txo_types::table
                    .on(transaction_logs::id.eq(transaction_txo_types::transaction_log_id)),
            )
            .inner_join(txos::table.on(transaction_txo_types::txo_id_hex.eq(txos::txo_id_hex)))
            .select((
                transaction_logs::all_columns,
                transaction_txo_types::all_columns,
                txos::all_columns,
            ))
            .order(transaction_logs::id);

        if let (Some(o), Some(l)) = (offset, limit) {
            transactions_query = transactions_query.offset(o as i64).limit(l as i64);
        }

        if let Some(min_block_index) = min_block_index {
            transactions_query = transactions_query
                .filter(transaction_logs::finalized_block_index.ge(min_block_index as i64));
        }

        if let Some(max_block_index) = max_block_index {
            transactions_query = transactions_query
                .filter(transaction_logs::finalized_block_index.le(max_block_index as i64));
        }

        let transactions: Vec<(TransactionLog, TransactionTxoType, Txo)> =
            transactions_query.load(conn)?;

        #[derive(Clone)]
        struct TransactionContents {
            transaction_log: TransactionLog,
            inputs: Vec<Txo>,
            outputs: Vec<Txo>,
            change: Vec<Txo>,
        }
        let mut results: HashMap<String, TransactionContents> = HashMap::default();
        for (transaction, transaction_txo_type, txo) in transactions {
            if results.get(&transaction.id).is_none() {
                results.insert(
                    transaction.id.clone(),
                    TransactionContents {
                        transaction_log: transaction.clone(),
                        inputs: Vec::new(),
                        outputs: Vec::new(),
                        change: Vec::new(),
                    },
                );
            };

            let entry = results.get_mut(&transaction.id).unwrap();

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

        results.sort_by_key(|r| r.0.id.clone());
        Ok(results)
    }

    fn log_submitted(
        tx_proposal: TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: &Conn,
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
            txo.update_to_pending(tx_proposal.tx.prefix.tombstone_block, conn)?;
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

        let transaction_log_id = TransactionID::from(&tx_proposal.tx);
        let tx = mc_util_serial::encode(&tx_proposal.tx);

        let new_transaction_log = NewTransactionLog {
            id: &transaction_log_id.to_string(),
            account_id_hex,
            fee_value: tx_proposal.tx.prefix.fee as i64,
            fee_token_id: tx_proposal.tx.prefix.fee_token_id as i64,
            submitted_block_index: Some(block_index as i64),
            tombstone_block_index: None,
            finalized_block_index: None,
            comment: &comment,
            tx: &tx,
            failed: false,
        };

        diesel::insert_into(crate::db::schema::transaction_logs::table)
            .values(&new_transaction_log)
            .execute(conn)?;

        // Create an entry per TXO for the TransactionTxoTypes
        for (txo_id_hex, transaction_txo_type) in txo_ids {
            let new_transaction_txo = NewTransactionTxoType {
                transaction_log_id: &transaction_log_id.to_string(),
                txo_id_hex: &txo_id_hex,
                transaction_txo_type: &transaction_txo_type,
            };
            diesel::insert_into(crate::db::schema::transaction_txo_types::table)
                .values(&new_transaction_txo)
                .execute(conn)?;
        }

        TransactionLog::get(&transaction_log_id, conn)
    }

    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::{
            transaction_logs as cols, transaction_logs::dsl::transaction_logs,
            transaction_txo_types as types_cols, transaction_txo_types::dsl::transaction_txo_types,
        };

        let results: Vec<String> = transaction_logs
            .filter(cols::account_id_hex.eq(account_id_hex))
            .select(cols::id)
            .load(conn)?;

        for transaction_log_id in results.iter() {
            diesel::delete(
                transaction_txo_types.filter(types_cols::transaction_log_id.eq(transaction_log_id)),
            )
            .execute(conn)?;
        }

        diesel::delete(transaction_logs.filter(cols::account_id_hex.eq(account_id_hex)))
            .execute(conn)?;

        Ok(())
    }

    fn update_tx_logs_associated_with_txo_to_succeeded(
        txo_id_hex: &str,
        finalized_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types};

        // Find all transaction logs associated with this txo that have not yet been
        // finalized.
        let transaction_log_ids: Vec<String> = transaction_logs::table
            .inner_join(
                transaction_txo_types::table
                    .on(transaction_logs::id.eq(transaction_txo_types::transaction_log_id)),
            )
            .filter(transaction_txo_types::txo_id_hex.eq(txo_id_hex))
            .filter(transaction_logs::failed.eq(false))
            .filter(transaction_logs::finalized_block_index.is_null())
            .select(transaction_logs::id)
            .load(conn)?;

        diesel::update(
            transaction_logs::table.filter(transaction_logs::id.eq_any(transaction_log_ids)),
        )
        .set((transaction_logs::finalized_block_index.eq(finalized_block_index as i64),))
        .execute(conn)?;

        Ok(())
    }

    fn update_tx_logs_associated_with_txos_to_failed(
        txos: &[Txo],
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{transaction_logs, transaction_txo_types};

        let txo_ids: Vec<String> = txos.iter().map(|txo| txo.txo_id_hex.clone()).collect();

        // Find all transaction_logs that are BUILT or PENDING that are associated
        // with the txo id when it is used as an input.
        // Update the status to FAILED
        let transaction_log_ids: Vec<String> = transaction_logs::table
            .inner_join(
                transaction_txo_types::table
                    .on(transaction_logs::id.eq(transaction_txo_types::transaction_log_id)),
            )
            .filter(transaction_txo_types::txo_id_hex.eq_any(txo_ids))
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
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, CHANGE_SUBADDRESS_INDEX};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Amount, Token};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        db::account::{AccountID, AccountModel},
        service::{sync::SyncThread, transaction_builder::WalletTransactionBuilder},
        test_utils::{
            add_block_with_tx_outs, builder_for_random_recipient, create_test_received_txo,
            get_resolver_factory, get_test_ledger, manually_sync_account,
            random_account_with_seed_values, WalletDbTestContext, MOB,
        },
        util::b58::b58_encode_public_address,
    };

    use super::*;

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
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction
        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);
        builder.add_recipient(recipient.clone(), 50 * MOB).unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None, false).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
        )
        .unwrap();

        // The log's account ID matches the account_id which submitted the tx
        assert_eq!(
            tx_log.account_id_hex,
            AccountID::from(&account_key).to_string()
        );
        // // Value is the amount sent, not including fee and change
        // assert_eq!(tx_log.value as u64, 50 * MOB);
        // // Fee exists for submitted
        // assert_eq!(tx_log.fee.unwrap() as u64, Mob::MINIMUM_FEE);
        // Created and sent transaction is "pending" until it lands
        assert_eq!(tx_log.status(), TxStatus::Pending);
        assert_eq!(
            tx_log.submitted_block_index,
            Some(ledger_db.num_blocks().unwrap() as i64)
        );
        // There is no comment for this submission
        assert_eq!(tx_log.comment, "");

        // The tx in the log matches the tx in the proposal
        let tx: Tx = mc_util_serial::decode(&tx_log.clone().tx).unwrap();
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
        assert_eq!(input_details.value as u64, 70 * MOB);
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
        assert_eq!(output_details.value as u64, 50 * MOB);

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
        assert_eq!(change_details.value as u64, 20 * MOB - Mob::MINIMUM_FEE);

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
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    #[test_with_logger]
    fn test_log_submitted_zero_change(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![100 * MOB, 200 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction
        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);
        // Add outlays all to the same recipient, so that we exceed u64::MAX in this tx
        let value = 100 * MOB - Mob::MINIMUM_FEE;
        builder.add_recipient(recipient.clone(), value).unwrap();

        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None, false).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
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
        // // No assigned subaddress for sent
        // assert_eq!(tx_log.assigned_subaddress_b58, None);
        // // Value is the amount sent, not including fee and change
        // assert_eq!(tx_log.value as u64, value);
        // // Fee exists for submitted
        // assert_eq!(tx_log.fee.unwrap() as u64, Mob::MINIMUM_FEE);
        // Created and sent transaction is "pending" until it lands
        assert_eq!(tx_log.status(), TxStatus::Pending);
        assert_eq!(
            tx_log.submitted_block_index.unwrap() as u64,
            ledger_db.num_blocks().unwrap()
        );
        assert_eq!(tx_log.comment, "");
        let tx: Tx = mc_util_serial::decode(&tx_log.clone().tx).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Get associated Txos
        let associated = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();
        assert_eq!(associated.inputs.len(), 1);
        assert_eq!(associated.outputs.len(), 1);
        assert_eq!(associated.change.len(), 1);
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
                "".to_string(),
                "".to_string(),
                "".to_string(),
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();

            let subaddress = account_key.subaddress(0);
            let assigned_subaddress_b58 = Some(b58_encode_public_address(&subaddress).unwrap());

            // Ingest relevant txos.
            for i in 1..=10 {
                let (txo_id_hex, _txo, _key_image) = create_test_received_txo(
                    &account_key,
                    0, // All to the same subaddress
                    Amount::new(100 * i * MOB, Mob::ID),
                    144,
                    &mut rng,
                    &wallet_db,
                );
            }

            account_ids.push(account_id);
        }

        // // Check that we created transaction_logs and transaction_txo_types
        // entries. assert_eq!(
        //     Ok(20),
        //     transaction_logs::table
        //         .select(count_star())
        //         .first(&wallet_db.get_conn().unwrap())
        // );
        // assert_eq!(
        //     Ok(20),
        //     transaction_txo_types::table
        //         .select(count_star())
        //         .first(&wallet_db.get_conn().unwrap())
        // );

        // // Delete the transaction logs for one account.
        // let result = TransactionLog::delete_all_for_account(
        //     &account_ids[0].to_string(),
        //     &wallet_db.get_conn().unwrap(),
        // );
        // assert!(result.is_ok());

        // // For the given account, the transaction logs and the txo types are
        // deleted. assert_eq!(
        //     Ok(10),
        //     transaction_logs::table
        //         .select(count_star())
        //         .first(&wallet_db.get_conn().unwrap())
        // );
        // assert_eq!(
        //     Ok(10),
        //     transaction_txo_types::table
        //         .select(count_star())
        //         .first(&wallet_db.get_conn().unwrap())
        // );
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
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![7_000_000 * MOB, 14_000_000 * MOB],
            &mut rng,
            &logger,
        );

        // Build a transaction for > i64::Max
        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);
        builder
            .add_recipient(recipient.clone(), 10_000_000 * MOB)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None, false).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        assert_eq!(tx_proposal.outlays[0].value, 10_000_000_000_000_000_000);

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
        )
        .unwrap();

        // assert_eq!(tx_log.value as u64, 10_000_000 * MOB);
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
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![7 * MOB, 8 * MOB],
            &mut rng,
            &logger,
        );

        let conn = wallet_db.get_conn().unwrap();
        let mut builder = WalletTransactionBuilder::new(
            AccountID::from(&account_key).to_string(),
            ledger_db.clone(),
            get_resolver_factory(&mut rng).unwrap(),
            logger.clone(),
        );
        // Add self at main subaddress as the recipient
        builder
            .add_recipient(account_key.subaddress(0), 12 * MOB)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None, false).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
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
        assert_eq!(input_details0.value as u64, 8 * MOB);

        assert!(input_details0.is_pending());
        assert!(input_details0.is_received());
        assert_eq!(input_details0.subaddress_index, Some(0));
        assert!(!input_details0.is_minted());

        let input_details1 = Txo::get(
            &associated_txos.inputs[1].txo_id_hex,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details1.value as u64, 7 * MOB);

        assert!(input_details1.is_pending());
        assert!(input_details1.is_received());
        assert_eq!(input_details1.subaddress_index, Some(0));
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
        assert_eq!(output_details.value as u64, 12 * MOB);

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
        assert_eq!(change_details.value as u64, 3 * MOB - Mob::MINIMUM_FEE);
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
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    // FIXME: test_log_submitted for recovered
    // FIXME: test_log_submitted offline flow
}
