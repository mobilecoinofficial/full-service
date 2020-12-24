// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Locked model.
//!
//! There are two "locked" states for a DB, in order to support a mode in which you can
//! still submit transactions without accounts (and therefore without a password).

use crate::{
    db::{
        account::AccountModel,
        models::{Account, LockedIndicator},
    },
    error::WalletDbError,
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    QueryDsl, RunQueryDsl,
};

#[derive(Debug)]
pub enum LockedState {
    Empty,
    Locked,
    Unlocked,
}

pub trait LockedModel {
    fn get_locked_state(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<LockedState, WalletDbError>;

    fn verify_password(
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    // fn set_password(
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;
}

impl LockedModel for LockedIndicator {
    fn get_locked_state(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<LockedState, WalletDbError> {
        use crate::db::schema::locked_indicators::dsl::locked_indicators;

        match locked_indicators.find(true).first::<LockedIndicator>(conn) {
            Ok(_) => Ok(LockedState::Locked),
            Err(diesel::result::Error::NotFound) => {
                if Account::list_all(conn)?.is_empty() {
                    Ok(LockedState::Empty)
                } else {
                    Ok(LockedState::Unlocked)
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn verify_password(
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        // FIXME: todo
        Ok(true)
    }

    // fn set_password(
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError> {
    //     use crate::db::schema::locked_indicators::dsl::locked;
    //
    //     Ok(conn.transaction::<(), WalletDbError, _>(|| {
    //         diesel::update(locked.find(0))
    //             .set(crate::db::schema::locked_indicators::locked.eq(1))
    //             .execute(conn)?;
    //         Ok(())
    //     })?)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};

    // The wallet DB should be LockedState::Empty on startup of a new DB
    #[test_with_logger]
    fn test_locked_startup(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let conn = wallet_db.get_conn().unwrap();
        match LockedIndicator::get_locked_state(&conn).unwrap() {
            LockedState::Empty => {}
            LockedState::Locked => panic!("Should not be locked on startup if empty"),
            LockedState::Unlocked => panic!("Should not be unlocked on startup"),
        }
    }
}
