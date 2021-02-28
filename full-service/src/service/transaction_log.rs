// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction logs.

use crate::{
    db::{
        account::AccountID,
        models::TransactionLog,
        transaction_log::{AssociatedTxos, TransactionLogModel},
    },
    error::WalletServiceError,
    WalletService,
};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

use diesel::connection::Connection;

/// Trait defining the ways in which the wallet can interact with and manage
/// transaction logs.
pub trait TransactionLogService {
    /// List all transactions associated with the given Account ID.
    fn list_transaction_logs(
        &self,
        account_id: &AccountID,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;

    /// Get a specific transaction log.
    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos), WalletServiceError>;

    /// Get all transaction logs for a given block.
    fn get_all_transaction_logs_for_block(
        &self,
        block_index: u64,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;

    /// Get all transaction logs ordered by finalized_block_index.
    fn get_all_transaction_logs_ordered_by_block(
        &self,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;
}

impl<T, FPR> TransactionLogService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_transaction_logs(
        &self,
        account_id: &AccountID,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        Ok(TransactionLog::list_all(
            &account_id.to_string(),
            &self.wallet_db.get_conn()?,
        )?)
    }

    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos), WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<(TransactionLog, AssociatedTxos), WalletServiceError, _>(|| {
                let transaction_log = TransactionLog::get(transaction_id_hex, &conn)?;
                let associated = transaction_log.get_associated_txos(&conn)?;

                Ok((transaction_log, associated))
            })?,
        )
    }

    fn get_all_transaction_logs_for_block(
        &self,
        block_index: u64,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError, _>(
                || {
                    let transaction_logs =
                        TransactionLog::get_all_for_block_index(block_index, &conn)?;
                    let mut res: Vec<(TransactionLog, AssociatedTxos)> = Vec::new();
                    for transaction_log in transaction_logs {
                        res.push((
                            transaction_log.clone(),
                            transaction_log.get_associated_txos(&conn)?,
                        ));
                    }
                    Ok(res)
                },
            )?,
        )
    }

    fn get_all_transaction_logs_ordered_by_block(
        &self,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError, _>(
                || {
                    let transaction_logs = TransactionLog::get_all_ordered_by_block_index(&conn)?;
                    let mut res: Vec<(TransactionLog, AssociatedTxos)> = Vec::new();
                    for transaction_log in transaction_logs {
                        res.push((
                            transaction_log.clone(),
                            transaction_log.get_associated_txos(&conn)?,
                        ));
                    }
                    Ok(res)
                },
            )?,
        )
    }
}
