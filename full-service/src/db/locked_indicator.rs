// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Locked model.

use crate::{db::models::LockedIndicator, error::WalletDbError};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    QueryDsl, RunQueryDsl,
};

pub trait LockedModel {
    fn is_locked(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    // fn set_locked(
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;
}

impl LockedModel for LockedIndicator {
    fn is_locked(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        use crate::db::schema::locked_indicators::dsl::locked_indicators;

        match locked_indicators.find(true).first::<LockedIndicator>(conn) {
            Ok(_) => Ok(true),
            Err(diesel::result::Error::NotFound) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    // fn set_locked(
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

    // The wallet DB should not be locked on startup of a new DB
    #[test_with_logger]
    fn test_locked_startup(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let conn = wallet_db.get_conn().unwrap();
        assert!(!LockedIndicator::is_locked(&conn).unwrap());
    }
}
