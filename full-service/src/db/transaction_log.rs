// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Transaction model.

use diesel::prelude::*;
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::{tx::Tx, TokenId};
use std::fmt;

use crate::db::{
    account::{AccountID, AccountModel},
    models::{
        Account, NewTransactionInputTxo, NewTransactionLog, TransactionInputTxo, TransactionLog,
        TransactionOutputTxo, Txo,
    },
    txo::{TxoID, TxoModel},
    Conn, WalletDbError,
};

use crate::service::models::tx_proposal::TxProposal;

#[derive(Debug)]
pub struct TransactionID(pub String);

impl From<&TransactionLog> for TransactionID {
    fn from(tx_log: &TransactionLog) -> Self {
        Self(tx_log.id.clone())
    }
}

impl From<&TxProposal> for TransactionID {
    fn from(_tx_proposal: &TxProposal) -> Self {
        Self::from(&_tx_proposal.tx)
    }
}

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

#[derive(Debug, PartialEq)]
pub enum TxStatus {
    // The transaction log has been built but not yet submitted to consensus
    Built,
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

    fn update_submitted_block_index(
        &self,
        submitted_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

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
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletDbError>;

    fn log_built(
        tx_proposal: TxProposal,
        comment: String,
        account_id_hex: &str,
        conn: &Conn,
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
    fn log_submitted(
        tx_proposal: &TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<TransactionLog, WalletDbError>;

    /// Remove all logs for an account
    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError>;

    fn update_pending_associated_with_txo_to_succeeded(
        txo_id_hex: &str,
        finalized_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn update_pending_exceeding_tombstone_block_index_to_failed(
        block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn status(&self) -> TxStatus;

    fn value_for_token_id(&self, token_id: TokenId, conn: &Conn) -> Result<u64, WalletDbError>;

    fn value_map(&self, conn: &Conn) -> Result<ValueMap, WalletDbError>;
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
        use crate::db::schema::{transaction_input_txos, transaction_output_txos, txos};

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
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_logs;

        diesel::update(self)
            .set(transaction_logs::submitted_block_index.eq(Some(submitted_block_index as i64)))
            .execute(conn)?;

        Ok(())
    }

    fn list_all(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletDbError> {
        use crate::db::schema::transaction_logs;

        let mut query = transaction_logs::table
            .into_boxed()
            .filter(transaction_logs::account_id.eq(account_id_hex));

        if let (Some(o), Some(l)) = (offset, limit) {
            query = query.offset(o as i64).limit(l as i64);
        }

        if let Some(min_block_index) = min_block_index {
            query =
                query.filter(transaction_logs::finalized_block_index.ge(min_block_index as i64));
        }

        if let Some(max_block_index) = max_block_index {
            query =
                query.filter(transaction_logs::finalized_block_index.le(max_block_index as i64));
        }

        let transaction_logs: Vec<TransactionLog> = query.order(transaction_logs::id).load(conn)?;

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
        tx_proposal: TxProposal,
        comment: String,
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        // Verify that the TxProposal is well-formed according to our
        // assumptions about how to store the sent data in our wallet
        // (num_output_TXOs = num_outlays + change_TXO).
        // if tx_proposal.tx.prefix.outputs.len() - tx_proposal.outlays.len() > 1 {
        //     return Err(WalletDbError::UnexpectedNumberOfChangeOutputs);
        // }

        let transaction_log_id = TransactionID::from(&tx_proposal);
        let tx = mc_util_serial::encode(&tx_proposal.tx);

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
            // Txo::update_key_image(&txo_id.to_string(), &utxo.key_image, None, conn)?;
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

        TransactionLog::get(&transaction_log_id, conn)
    }

    fn log_submitted(
        tx_proposal: &TxProposal,
        block_index: u64,
        comment: String,
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<TransactionLog, WalletDbError> {
        // Verify that the account exists.
        Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        // // Verify that the TxProposal is well-formed according to our
        // // assumptions about how to store the sent data in our wallet
        // // (num_output_TXOs = num_outlays + change_TXO).
        // if tx_proposal.tx.prefix.outputs.len() - tx_proposal.outlays.len() > 1 {
        //     return Err(WalletDbError::UnexpectedNumberOfChangeOutputs);
        // }

        let transaction_log_id = TransactionID::from(&tx_proposal.tx);
        let tx = mc_util_serial::encode(&tx_proposal.tx);

        match TransactionLog::get(&transaction_log_id, conn) {
            Ok(transaction_log) => {
                transaction_log.update_submitted_block_index(block_index, conn)?;
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

    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::{
            transaction_input_txos, transaction_logs, transaction_output_txos,
        };

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
        txo_id_hex: &str,
        finalized_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{transaction_input_txos, transaction_logs};
        // Find all transaction logs associated with this txo that have not
        // yet been finalized (there should only ever be one).
        // TODO - WHY WON'T THIS WORK?!?!?
        let transaction_log_ids: Vec<String> = transaction_logs::table
            .inner_join(transaction_input_txos::table)
            .filter(transaction_input_txos::txo_id.eq(txo_id_hex))
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

    fn update_pending_exceeding_tombstone_block_index_to_failed(
        block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::transaction_logs;

        diesel::update(
            transaction_logs::table
                .filter(transaction_logs::tombstone_block_index.lt(block_index as i64))
                .filter(transaction_logs::failed.eq(false))
                .filter(transaction_logs::finalized_block_index.is_null()),
        )
        .set((transaction_logs::failed.eq(true),))
        .execute(conn)?;

        Ok(())
    }

    fn value_for_token_id(&self, token_id: TokenId, conn: &Conn) -> Result<u64, WalletDbError> {
        let associated_txos = self.get_associated_txos(conn)?;

        let output_total = associated_txos
            .outputs
            .iter()
            .filter(|(txo, _)| txo.token_id as u64 == *token_id)
            .map(|(txo, _)| txo.value as u64)
            .sum::<u64>();

        Ok(output_total)
    }

    fn value_map(&self, conn: &Conn) -> Result<ValueMap, WalletDbError> {
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
    use mc_account_keys::{PublicAddress, CHANGE_SUBADDRESS_INDEX};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        db::{account::AccountID, transaction_log::TransactionID, txo::TxoStatus},
        service::{sync::SyncThread, transaction_builder::WalletTransactionBuilder},
        test_utils::{
            add_block_from_transaction_log, add_block_with_tx_outs, builder_for_random_recipient,
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
        builder
            .add_recipient(recipient.clone(), 50 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
        )
        .unwrap();

        // The log's account ID matches the account_id which submitted the tx
        assert_eq!(tx_log.account_id, AccountID::from(&account_key).to_string());
        assert_eq!(tx_log.value_for_token_id(Mob::ID, &conn).unwrap(), 50 * MOB);
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
        let tx: Tx = mc_util_serial::decode(&tx_log.clone().tx).unwrap();
        assert_eq!(tx, tx_proposal.tx);

        // Check the associated_txos for this transaction_log are as expected
        let associated_txos = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();

        // There is one associated input TXO to this transaction, and it is now pending.
        assert_eq!(associated_txos.inputs.len(), 1);
        let input_details = Txo::get(
            &associated_txos.inputs[0].id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details.value as u64, 70 * MOB);
        assert_eq!(
            input_details
                .status(&wallet_db.get_conn().unwrap())
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
            &wallet_db.get_conn().unwrap(),
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
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(change_details.value as u64, 20 * MOB - Mob::MINIMUM_FEE);

        // Note, this will still be marked as not change until the txo
        // appears on the ledger and the account syncs.
        // change becomes unspent once scanned
        assert_eq!(
            change_details.subaddress_index,
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );

        add_block_from_transaction_log(&mut ledger_db, &wallet_db.get_conn().unwrap(), &tx_log);

        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
        let _sync = manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(tx_log.account_id.to_string()),
            &logger,
        );

        let updated_tx_log = TransactionLog::get(
            &TransactionID::from(&tx_log),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(updated_tx_log.status(), TxStatus::Succeeded);

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].0.id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(
            updated_change_details.status(&conn).unwrap(),
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
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();

        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
        )
        .unwrap();

        assert_eq!(tx_log.account_id, AccountID::from(&account_key).to_string());
        let associated_txos = tx_log
            .get_associated_txos(&wallet_db.get_conn().unwrap())
            .unwrap();
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].1,
            b58_encode_public_address(&recipient).unwrap()
        );

        assert_eq!(tx_log.value_for_token_id(Mob::ID, &conn).unwrap(), value);
        assert_eq!(tx_log.fee_value as u64, Mob::MINIMUM_FEE);
        assert_eq!(tx_log.fee_token_id as u64, *Mob::ID);
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
        use crate::db::schema::{
            transaction_input_txos, transaction_logs, transaction_output_txos,
        };
        use diesel::dsl::count_star;

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

        let account_id = AccountID::from(&account_key);

        // Build a transaction
        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);
        builder
            .add_recipient(recipient.clone(), 50 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        // Log submitted transaction from tx_proposal
        TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
        )
        .unwrap();

        // Check that we created transaction_logs and transaction_txo_types entries.
        assert_eq!(
            Ok(1),
            transaction_logs::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
        assert_eq!(
            Ok(1),
            transaction_input_txos::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
        assert_eq!(
            Ok(2),
            transaction_output_txos::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );

        // Delete the transaction logs for one account.
        let result = TransactionLog::delete_all_for_account(
            &account_id.to_string(),
            &wallet_db.get_conn().unwrap(),
        );
        assert!(result.is_ok());

        // For the given account, the transaction logs and the txo types are
        // deleted.
        assert_eq!(
            Ok(0),
            transaction_logs::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
        assert_eq!(
            Ok(0),
            transaction_input_txos::table
                .select(count_star())
                .first(&wallet_db.get_conn().unwrap())
        );
        assert_eq!(
            Ok(0),
            transaction_output_txos::table
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
            .add_recipient(recipient.clone(), 10_000_000 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        assert_eq!(tx_proposal.outlays[0].value, 10_000_000_000_000_000_000);

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&account_key).to_string(),
            &conn,
        )
        .unwrap();

        let pmob_value = tx_log.value_for_token_id(Mob::ID, &conn).unwrap();
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
            .add_recipient(account_key.subaddress(0), 12 * MOB, Mob::ID)
            .unwrap();
        builder.set_tombstone(0).unwrap();
        builder.select_txos(&conn, None).unwrap();
        let tx_proposal = builder.build(&conn).unwrap();

        // Log submitted transaction from tx_proposal
        let tx_log = TransactionLog::log_submitted(
            &tx_proposal,
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
            &associated_txos.inputs[0].id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details0.value as u64, 8 * MOB);

        assert_eq!(
            input_details0
                .status(&wallet_db.get_conn().unwrap())
                .unwrap(),
            TxoStatus::Pending
        );
        assert_eq!(input_details0.subaddress_index, Some(0));

        let input_details1 = Txo::get(
            &associated_txos.inputs[1].id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(input_details1.value as u64, 7 * MOB);

        assert_eq!(
            input_details1
                .status(&wallet_db.get_conn().unwrap())
                .unwrap(),
            TxoStatus::Pending
        );
        assert_eq!(input_details1.subaddress_index, Some(0));

        // There is one associated output TXO to this transaction, and its recipient
        // is our own address
        assert_eq!(associated_txos.outputs.len(), 1);
        assert_eq!(
            associated_txos.outputs[0].1,
            b58_encode_public_address(&account_key.subaddress(0)).unwrap()
        );
        let output_details = Txo::get(
            &associated_txos.outputs[0].0.id,
            &wallet_db.get_conn().unwrap(),
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
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        // Change = (8 + 7) - 12 - fee
        assert_eq!(change_details.value as u64, 3 * MOB - Mob::MINIMUM_FEE);
        assert_eq!(
            change_details.subaddress_index,
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );

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
            &AccountID(tx_log.account_id.to_string()),
            &logger,
        );

        // Get the Input Txos again
        let updated_input_details0 = Txo::get(
            &associated_txos.inputs[0].id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let updated_input_details1 = Txo::get(
            &associated_txos.inputs[1].id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // The inputs are now spent
        assert_eq!(
            updated_input_details0
                .status(&wallet_db.get_conn().unwrap())
                .unwrap(),
            TxoStatus::Spent
        );
        assert_eq!(
            updated_input_details1
                .status(&wallet_db.get_conn().unwrap())
                .unwrap(),
            TxoStatus::Spent
        );

        // The received_to account is ourself, which is the same as the account
        // account_id in the transaction log. The type is "Received"
        assert_eq!(
            updated_input_details0.account_id,
            Some(tx_log.account_id.clone())
        );
        assert_eq!(updated_input_details0.subaddress_index, Some(0 as i64));

        assert_eq!(
            updated_input_details1.account_id,
            Some(tx_log.account_id.clone())
        );
        assert_eq!(updated_input_details1.subaddress_index, Some(0 as i64));

        // Get the output txo again
        let updated_output_details = Txo::get(
            &associated_txos.outputs[0].0.id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        // The minted from account is ourself, and it is unspent, minted
        assert_eq!(
            updated_output_details
                .status(&wallet_db.get_conn().unwrap())
                .unwrap(),
            TxoStatus::Unspent
        );

        // The received to account is ourself, and it is unspent, minted
        assert_eq!(
            updated_output_details.account_id,
            Some(tx_log.account_id.clone())
        );

        // Received to main subaddress
        assert_eq!(updated_output_details.subaddress_index, Some(0 as i64));

        // Get the change txo again
        let updated_change_details = Txo::get(
            &associated_txos.change[0].0.id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(
            updated_change_details
                .status(&wallet_db.get_conn().unwrap())
                .unwrap(),
            TxoStatus::Unspent
        );
        assert_eq!(updated_change_details.account_id, Some(tx_log.account_id));
        assert_eq!(
            updated_change_details.subaddress_index,
            Some(CHANGE_SUBADDRESS_INDEX as i64)
        );
    }

    // FIXME: test_log_submitted for recovered
    // FIXME: test_log_submitted offline flow
}
