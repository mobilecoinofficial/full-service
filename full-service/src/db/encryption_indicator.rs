// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Locked model.
//!
//! There are two "locked" states for a DB, in order to support a mode in which you can
//! still submit transactions without accounts (and therefore without a password).

use crate::{
    db::{
        account::AccountModel,
        models::{Account, EncryptionIndicator},
    },
    error::WalletDbError,
};

use crate::db::models::NewEncryptionIndicator;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    QueryDsl, RunQueryDsl,
};

#[derive(Debug)]
pub enum EncryptionState {
    Empty,
    Encrypted,
    Unencrypted,
}

pub trait EncryptionModel {
    fn get_encryption_state(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<EncryptionState, WalletDbError>;

    fn verify_password(
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    fn set_password_hash(
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl EncryptionModel for EncryptionIndicator {
    fn get_encryption_state(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<EncryptionState, WalletDbError> {
        use crate::db::schema::encryption_indicators::dsl::encryption_indicators;

        match encryption_indicators
            .find(true)
            .first::<EncryptionIndicator>(conn)
        {
            Ok(_) => Ok(EncryptionState::Encrypted),
            Err(diesel::result::Error::NotFound) => {
                if Account::list_all(conn)?.is_empty() {
                    Ok(EncryptionState::Empty)
                } else {
                    Ok(EncryptionState::Unencrypted)
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn verify_password(
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        use crate::db::schema::encryption_indicators;

        Ok(conn.transaction::<bool, WalletDbError, _>(|| {
            let indicator_rows: Vec<EncryptionIndicator> = encryption_indicators::table
                .select(encryption_indicators::all_columns)
                .load::<EncryptionIndicator>(conn)?;
            if indicator_rows.is_empty() {
                Err(WalletDbError::SetPassword)
            } else if indicator_rows.len() > 1 {
                Err(WalletDbError::BadEncryptionState)
            } else {
                if let Some(hash) = indicator_rows[0].verification_value.clone() {
                    Ok(hash == password_hash) // FIXME do decryption
                } else {
                    Err(WalletDbError::BadEncryptionState)
                }
            }
        })?)
    }

    fn set_password_hash(
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::encryption_indicators as encryption_table;
        use crate::db::schema::encryption_indicators::dsl::encryption_indicators;

        Ok(conn.transaction::<(), WalletDbError, _>(|| {
            let password_hash_insertable = password_hash.to_vec();
            let new_indicator = NewEncryptionIndicator {
                encrypted: true,
                verification_value: Some(&password_hash_insertable), // FIXME: do encryption
            };

            // Delete the whole table (should only be one row)
            diesel::delete(encryption_table::table).execute(conn)?;

            diesel::insert_into(encryption_table::table)
                .values(new_indicator)
                .execute(conn)?;
            Ok(())
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};

    // The wallet DB should be LockedState::Empty on startup of a new DB.
    #[test_with_logger]
    fn test_locked_startup(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let conn = wallet_db.get_conn().unwrap();
        match EncryptionIndicator::get_encryption_state(&conn).unwrap() {
            EncryptionState::Empty => {}
            EncryptionState::Encrypted => panic!("Should not be locked on startup if empty"),
            EncryptionState::Unencrypted => panic!("Should not be unlocked on startup"),
        }
    }

    // The set password should verify.
    #[test_with_logger]
    fn test_setting_and_verifying_password(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let mut password_hash = [1u8; 32];
        EncryptionIndicator::set_password_hash(&password_hash, &wallet_db.get_conn().unwrap())
            .unwrap();
        assert!(EncryptionIndicator::verify_password(
            &password_hash,
            &wallet_db.get_conn().unwrap()
        )
        .unwrap());
    }
}
