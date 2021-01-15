// Copyright (c) 2020-2021 MobileCoin Inc.

//! Implementations of DB and DB models.

pub mod account;
pub mod account_txo_status;
pub mod assigned_subaddress;
pub mod encryption_indicator;
pub mod encryption_provider;
pub mod models;
pub mod schema;
pub mod transaction_log;
pub mod txo;

use self::encryption_provider::EncryptionProvider;
use crate::error::WalletDbError;

use mc_account_keys::PublicAddress;
use mc_common::logger::{log, Logger};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use std::{convert::TryFrom, sync::Arc};

/// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

/// Helper method to use our PrintableWrapper when decoding a b58 PublicAddress
pub fn b58_decode(b58_public_address: &str) -> Result<PublicAddress, WalletDbError> {
    let wrapper =
        mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(b58_public_address.to_string())
            .unwrap();
    let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
        let payment_request = wrapper.get_payment_request();
        payment_request.get_public_address()
    } else if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        return Err(WalletDbError::B58Decode);
    };
    Ok(PublicAddress::try_from(pubaddr_proto).unwrap())
}

/// A struct encapsulating a single connection, which can also be used for encryption and logging.
/// Intended to be passed to trait methods in various db::models.
pub struct WalletDbConnManager {
    pub conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    pub encryption_provider: Arc<EncryptionProvider>,
    pub logger: Logger,
}

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
    encryption_provider: Arc<EncryptionProvider>,
    logger: Logger,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>, logger: Logger) -> Self {
        Self {
            pool,
            encryption_provider: Arc::new(EncryptionProvider::new(logger.clone())),
            logger,
        }
    }

    pub fn new_from_url(database_url: &str, logger: Logger) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(10)
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool, logger))
    }

    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        log::info!(
            self.logger,
            "\x1b[1;31m GETTING NEW CONNECTION TO DB, current pool state: {:?}\x1b[0m",
            self.pool.state()
        );
        Ok(self.pool.get()?)
    }

    pub fn get_conn_manager(&self) -> Result<WalletDbConnManager, WalletDbError> {
        Ok(WalletDbConnManager {
            conn: self.get_conn()?,
            encryption_provider: self.encryption_provider.clone(),
            logger: self.logger.clone(),
        })
    }

    /// Set the password hash for the database.
    pub fn set_password_hash(
        &self,
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        Ok(self
            .encryption_provider
            .set_password_hash(password_hash, conn)?)
    }

    /// Get the password hash for the database.
    pub fn get_password_hash(&self) -> Result<Vec<u8>, WalletDbError> {
        Ok(self.encryption_provider.get_password_hash()?)
    }

    /// Check whether the database is currently unlocked.
    pub fn is_unlocked(&self) -> Result<bool, WalletDbError> {
        Ok(self.encryption_provider.is_unlocked()?)
    }

    pub fn unlock(&self, password_hash: &[u8]) -> Result<(), WalletDbError> {
        let conn = self.get_conn()?;
        Ok(self.encryption_provider.unlock(password_hash, &conn)?)
    }

    pub fn change_password(
        &self,
        old_password_hash: &[u8],
        new_password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        Ok(self
            .encryption_provider
            .change_password(old_password_hash, new_password_hash, &conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            encryption_indicator::{EncryptionModel, EncryptionState},
            models::EncryptionIndicator,
        },
        test_utils::WalletDbTestContext,
    };
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    // Initializing DB, setting password, and unlocking should behave as expected.
    #[test_with_logger]
    fn test_encryption_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());

        // Db should be "empty" on sartup
        match EncryptionIndicator::get_encryption_state(&wallet_db.get_conn().unwrap()).unwrap() {
            EncryptionState::Empty => {}
            _ => panic!("DB should be empty on startup"),
        }

        let mut password_hash = [1u8; 32];
        rng.fill_bytes(&mut password_hash);

        wallet_db
            .set_password_hash(&password_hash, &wallet_db.get_conn().unwrap())
            .unwrap();
        assert_eq!(wallet_db.get_password_hash().unwrap(), password_hash);

        match EncryptionIndicator::get_encryption_state(&wallet_db.get_conn().unwrap()).unwrap() {
            EncryptionState::Encrypted => {}
            EncryptionState::Empty => panic!("Should not be empty after setting password"),
            EncryptionState::Unencrypted => {
                panic!("Should not be unencrypted after setting password")
            }
        }

        assert!(wallet_db.is_unlocked().unwrap());

        // To simulate re-opening the DB, we can use new_from_url
        let wallet_db2 = WalletDb::new_from_url(
            &format!("{}/{}", db_test_context.base_url, db_test_context.db_name),
            logger.clone(),
        )
        .unwrap();

        match EncryptionIndicator::get_encryption_state(&wallet_db.get_conn().unwrap()).unwrap() {
            EncryptionState::Encrypted => {}
            EncryptionState::Empty => panic!("Should be encrypted at rest and not empty"),
            EncryptionState::Unencrypted => panic!("Should be encrypted at rest"),
        }

        // DB should not be unlocked on startup
        assert!(!wallet_db2.is_unlocked().unwrap());

        // Trying to unlock with a bad password should fail
        let mut wrong_password_hash = [1u8; 32];
        rng.fill_bytes(&mut wrong_password_hash);

        match wallet_db2.unlock(&wrong_password_hash) {
            Err(WalletDbError::PasswordFailed) => {}
            Ok(_) => panic!("Should not be able to unlock DB with bad password"),
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        match wallet_db2.unlock(&password_hash) {
            Ok(_) => {}
            _ => panic!("Correct password should unlock DB"),
        }

        assert!(wallet_db2.is_unlocked().unwrap());

        // Now we'll change the password
        let mut new_password_hash = [1u8; 32];
        rng.fill_bytes(&mut new_password_hash);

        wallet_db2
            .change_password(
                &password_hash,
                &new_password_hash,
                &wallet_db2.get_conn().unwrap(),
            )
            .unwrap();

        // Should still be unlocked
        assert!(wallet_db2.is_unlocked().unwrap());

        // Open a new db instance
        let wallet_db3 = WalletDb::new_from_url(
            &format!("{}/{}", db_test_context.base_url, db_test_context.db_name),
            logger,
        )
        .unwrap();

        // Old password should fail
        match wallet_db3.unlock(&password_hash) {
            Err(WalletDbError::PasswordFailed) => {}
            Ok(_) => panic!("Should not be able to unlock DB with bad password"),
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        match wallet_db3.unlock(&new_password_hash) {
            Ok(_) => {}
            _ => panic!("Correct password should unlock DB"),
        }

        // Unlocking twice should no-op
        match wallet_db3.unlock(&new_password_hash) {
            Ok(_) => {}
            _ => panic!("Unlocking an unlocked DB should succeed."),
        }
    }
}
