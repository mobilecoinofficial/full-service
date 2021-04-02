// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the AccountTxoStatus model.

use crate::db::models::{
    AccountTxoStatus, NewAccountTxoStatus, TXO_STATUS_ORPHANED, TXO_STATUS_UNSPENT,
};

use crate::db::WalletDbError;
use diesel::{
    debug_query,
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};

pub trait AccountTxoStatusModel {
    fn create(
        account_id_hex: &str,
        txo_id_hex: &str,
        txo_status: &str,
        txo_type: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    fn get(
        account_id_hex: &str,
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AccountTxoStatus, WalletDbError>;

    fn get_all_associated_accounts(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<AccountTxoStatus>, WalletDbError>;

    fn get_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<AccountTxoStatus>, WalletDbError>;

    fn set_unspent(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    fn set_orphaned(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    fn delete_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl AccountTxoStatusModel for AccountTxoStatus {
    fn create(
        account_id_hex: &str,
        txo_id_hex: &str,
        txo_status: &str,
        txo_type: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::account_txo_statuses;

        let new_account_txo_status = NewAccountTxoStatus {
            account_id_hex,
            txo_id_hex,
            txo_status,
            txo_type,
        };

        diesel::insert_into(account_txo_statuses::table)
            .values(&new_account_txo_status)
            .execute(conn)?;

        Ok(())
    }

    fn get(
        account_id_hex: &str,
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AccountTxoStatus, WalletDbError> {
        use crate::db::schema::account_txo_statuses::dsl::account_txo_statuses;

        match account_txo_statuses
            .find((account_id_hex, &txo_id_hex))
            .get_result::<AccountTxoStatus>(conn)
        {
            Ok(t) => Ok(t),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::AccountTxoStatusNotFound(
                format!("({}, {})", account_id_hex, txo_id_hex),
            )),
            Err(e) => Err(e.into()),
        }
    }

    fn get_all_associated_accounts(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<AccountTxoStatus>, WalletDbError> {
        use crate::db::schema::{
            account_txo_statuses as cols, account_txo_statuses::dsl::account_txo_statuses,
        };

        let results: Vec<AccountTxoStatus> = account_txo_statuses
            .filter(cols::txo_id_hex.eq(txo_id_hex))
            .select(cols::all_columns)
            .load(conn)?;

        Ok(results)
    }

    fn get_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<AccountTxoStatus>, WalletDbError> {
        use crate::db::schema::{
            account_txo_statuses as cols, account_txo_statuses::dsl::account_txo_statuses,
        };

        let results = account_txo_statuses
            .filter(cols::account_id_hex.eq(account_id_hex))
            .select(cols::all_columns)
            .load(conn);

        match results {
            Ok(r) => Ok(r),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::AccountTxoStatusNotFound(
                format!("({})", account_id_hex),
            )),
            Err(e) => Err(e.into()),
        }
    }

    fn set_unspent(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::account_txo_statuses::txo_status;

        diesel::update(self)
            .set(txo_status.eq(TXO_STATUS_UNSPENT))
            .execute(conn)?;
        Ok(())
    }

    fn set_orphaned(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::account_txo_statuses::txo_status;

        diesel::update(self)
            .set(txo_status.eq(TXO_STATUS_ORPHANED))
            .execute(conn)?;
        Ok(())
    }

    fn delete_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{
            account_txo_statuses as cols, account_txo_statuses::dsl::account_txo_statuses,
        };
        use diesel::sqlite::Sqlite;

        let results: Vec<AccountTxoStatus> = account_txo_statuses
            .filter(cols::account_id_hex.eq(account_id_hex))
            .select(cols::all_columns)
            .load(conn)?;

        let res =
            diesel::delete(account_txo_statuses.filter(cols::account_id_hex.eq(account_id_hex)))
                .execute(conn)?;

        Ok(())
    }
}
