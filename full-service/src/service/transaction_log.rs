// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction logs.

use crate::{
    db::{
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
    fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;

    fn get_transaction(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos), WalletServiceError>;
}

impl<T, FPR> TransactionLogService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        Ok(TransactionLog::list_all(
            account_id_hex,
            &self.wallet_db.get_conn()?,
        )?)
    }

    fn get_transaction(
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
}
