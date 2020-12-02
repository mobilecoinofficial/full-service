// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Transaction model.

use crate::{
    db_models::txo::TxoModel,
    error::WalletDbError,
    models::{TransactionLog, TransactionTxoType, Txo},
};

use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::tx::Tx;

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

pub trait TransactionLogModel {
    fn get(
        transaction_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError>;

    fn get_associated_txos(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(Vec<String>, Vec<String>, Vec<String>), WalletDbError>;

    fn select_for_txo(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TransactionLog>, WalletDbError>;

    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl TransactionLogModel for TransactionLog {
    fn get(
        transaction_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<TransactionLog, WalletDbError> {
        use crate::schema::transaction_logs::dsl::transaction_logs;

        match transaction_logs
            .find(transaction_id_hex)
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
        use crate::schema::transaction_logs;
        use crate::schema::transaction_txo_types;

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
        use crate::schema::transaction_logs;
        use crate::schema::transaction_txo_types;

        Ok(transaction_logs::table
            .inner_join(
                transaction_txo_types::table.on(transaction_logs::transaction_id_hex
                    .eq(transaction_txo_types::transaction_id_hex)
                    .and(transaction_txo_types::txo_id_hex.eq(txo_id_hex))),
            )
            .select(transaction_logs::all_columns)
            .load(conn)?)
    }

    // FIXME: We may be doing n^2 work here
    fn update_transactions_associated_to_txo(
        txo_id_hex: &str,
        cur_block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::schema::transaction_logs::dsl::transaction_logs;

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
                diesel::update(transaction_logs.find(&transaction_log.transaction_id_hex))
                    .set((
                        crate::schema::transaction_logs::status.eq("succeeded"),
                        crate::schema::transaction_logs::block_height.eq(cur_block_height),
                    ))
                    .execute(conn)?;
            } else if Txo::any_failed(&inputs, cur_block_height, conn)? {
                // FIXME: Do we want to store and update the "failed_block_height" as min(tombstones)?
                diesel::update(transaction_logs.find(&transaction_log.transaction_id_hex))
                    .set(crate::schema::transaction_logs::status.eq("failed"))
                    .execute(conn)?;
            }
        }

        Ok(())
    }
}
