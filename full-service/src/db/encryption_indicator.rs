// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Locked model.
//!
//! There are two "locked" states for a DB, in order to support a mode in which you can
//! still submit transactions without accounts (and therefore without a password).

use crate::{
    db::{
        account::AccountModel,
        models::{Account, EncryptionIndicator, NewEncryptionIndicator},
    },
    error::WalletDbError,
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    QueryDsl, RunQueryDsl,
};

#[derive(Debug)]
pub enum EncryptionState {
    /// Database has never been locked, and set_password should be called.
    Empty,
    /// Database is encrypted.
    Encrypted,
    /// Database is unencrypted. This is for databases that existed before we added encryption.
    Unencrypted,
}

pub trait EncryptionModel {
    /// Gets the current encryption state for the database, as indicated by the indicator table
    fn get_encryption_state(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<EncryptionState, WalletDbError>;

    /// Verifies that the encrypted value in the database matches the expected value, which
    /// should have been obtained by encrypting the indicator value with some password.
    fn verify_encrypted_value(
        expected_val: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError>;

    /// Sets the verification value for the encryption indicator in the database.
    fn set_verification_value(
        encrypted_verification_value: &[u8],
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

    fn verify_encrypted_value(
        expected_val: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<bool, WalletDbError> {
        use crate::db::schema::encryption_indicators;

        Ok(conn.transaction::<bool, WalletDbError, _>(|| {
            let indicator_rows: Vec<EncryptionIndicator> = encryption_indicators::table
                .select(encryption_indicators::all_columns)
                .load::<EncryptionIndicator>(conn)?;
            if indicator_rows.is_empty() {
                Err(WalletDbError::MissingEncryptionIndicator)
            } else if indicator_rows.len() > 1 {
                Err(WalletDbError::BadEncryptionState)
            } else if let Some(current_val) = indicator_rows[0].verification_value.clone() {
                Ok(current_val == expected_val)
            } else {
                Err(WalletDbError::BadEncryptionState)
            }
        })?)
    }

    fn set_verification_value(
        encrypted_verification_value: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::encryption_indicators as encryption_table;

        Ok(conn.transaction::<(), WalletDbError, _>(|| {
            let _verification_val_insertable = encrypted_verification_value.to_vec();
            let new_indicator = NewEncryptionIndicator {
                encrypted: true,
                verification_value: Some(encrypted_verification_value),
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

    // The correct verification value should verify.
    #[test_with_logger]
    fn test_setting_and_verifying_value(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());

        let encrypted_verification_value = [1u8; 32];
        EncryptionIndicator::set_verification_value(
            &encrypted_verification_value,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert!(EncryptionIndicator::verify_encrypted_value(
            &encrypted_verification_value,
            &wallet_db.get_conn().unwrap()
        )
        .unwrap());
    }

    // An incorrect verification value should fail.
    #[test_with_logger]
    fn test_invalid_does_not_verify(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());

        let encrypted_verification_value = [1u8; 32];
        EncryptionIndicator::set_verification_value(
            &encrypted_verification_value,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert!(!EncryptionIndicator::verify_encrypted_value(
            &[2u8; 32],
            &wallet_db.get_conn().unwrap()
        )
        .unwrap());
    }
}
