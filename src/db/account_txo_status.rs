// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the AccountTxoStatus model.

use crate::{
    db::models::{AccountTxoStatus, NewAccountTxoStatus},
    error::WalletDbError,
};

use diesel::{
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

    fn set_unspent(
        &self,
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
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::NotFound(format!(
                "{:?}",
                (account_id_hex, txo_id_hex)
            ))),
            Err(e) => Err(e.into()),
        }
    }

    fn set_unspent(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::account_txo_statuses::txo_status;

        diesel::update(self)
            .set(txo_status.eq("unspent"))
            .execute(conn)?;
        Ok(())
    }
}
