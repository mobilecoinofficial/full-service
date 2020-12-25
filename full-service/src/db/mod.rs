// Copyright (c) 2020-2021 MobileCoin Inc.

//! Implementations of DB and DB models.

pub mod account;
pub mod account_txo_status;
pub mod assigned_subaddress;
pub mod encryption_indicator;
pub mod models;
pub mod schema;
pub mod transaction_log;
pub mod txo;

use self::{
    encryption_indicator::{EncryptionModel, EncryptionState},
    models::EncryptionIndicator,
};
use crate::error::WalletDbError;

use mc_account_keys::PublicAddress;
use mc_common::logger::{log, Logger};

use aes_gcm::{
    aead::{
        generic_array::{sequence::Split, GenericArray},
        Aead, AeadInPlace, NewAead,
    },
    Aes256Gcm,
};
use blake2::{Blake2b, Digest};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
};

/// Domain tag for database-wide encryption.
pub const ENCRYPTION_KEY_DOMAIN_TAG: &str = "mc_full_service";
const ENCRYPTION_VERIFICATION_VAL: &[u8] = b"verify";

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

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

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
    password_hash: Arc<Mutex<Vec<u8>>>,
    logger: Logger,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>, logger: Logger) -> Self {
        Self {
            pool,
            password_hash: Arc::new(Mutex::new(vec![])),
            logger,
        }
    }

    pub fn new_from_url(database_url: &str, logger: Logger) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool, logger))
    }

    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        Ok(self.pool.get()?)
    }

    /// Set the password hash for the DB.
    pub fn set_password_hash(
        &self,
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        {
            let mut cur_password_hash = self.password_hash.lock()?;
            *cur_password_hash = password_hash.to_vec();
        }
        // Encrypt the verification value and set in the DB
        let verification_value = Self::encrypt(ENCRYPTION_VERIFICATION_VAL, password_hash)?;
        EncryptionIndicator::set_verification_value(&verification_value, conn)?;

        Ok(())
    }

    pub fn get_password_hash(&self) -> Result<Vec<u8>, WalletDbError> {
        let cur_password_hash = self.password_hash.lock()?;
        Ok(cur_password_hash.clone())
    }

    pub fn is_unlocked(&self) -> Result<bool, WalletDbError> {
        Ok(!self.password_hash.lock()?.is_empty())
    }

    pub fn unlock(&self, password_hash: &[u8]) -> Result<(), WalletDbError> {
        // No need to check db state if we're already unlocked
        if self.is_unlocked()? {
            // Sanity check that password is correct - FIXME: should we just return Ok since it's unlocked already?
            if self.get_password_hash()? != password_hash {
                return Err(WalletDbError::PasswordFailed);
            }
            return Ok(());
        }

        let conn = self.get_conn()?;
        // Check whether encrypted, and if so, then attempt to unlock
        match EncryptionIndicator::get_encryption_state(&conn)? {
            EncryptionState::Empty => {
                // FIXME: move these log messages to service and return errors here
                log::info!(
                    self.logger,
                    "DB has never been locked. Please call set_password to enable encryption."
                );
            }
            EncryptionState::Encrypted => {
                log::debug!(self.logger, "DB is locked. Verifying password.");
                // Attempt to decrypt the test value to confirm if password is correct
                let expected_val = Self::encrypt(ENCRYPTION_VERIFICATION_VAL, password_hash)?;
                if EncryptionIndicator::verify_password(&expected_val, &conn)? {
                    // Store password hash in memory
                    let mut cur_password_hash = self.password_hash.lock()?;
                    *cur_password_hash = password_hash.to_vec();
                } else {
                    return Err(WalletDbError::PasswordFailed);
                }
            }
            EncryptionState::Unencrypted => {
                log::info!(
                    self.logger,
                    "DB is unencrypted. Please call set_password to enable encryption."
                );
            }
        }
        Ok(())
    }

    pub fn change_password(
        &self,
        old_password_hash: &[u8],
        new_password_hash: &[u8],
    ) -> Result<(), WalletDbError> {
        let conn = self.get_conn()?;
        // Check whether encrypted, and if so, then verify old password. For any other state, set_password to new.
        match EncryptionIndicator::get_encryption_state(&conn)? {
            EncryptionState::Empty => {
                log::info!(
                    self.logger,
                    "Database has never been locked. Setting password with new password."
                );
                self.set_password_hash(new_password_hash, &conn)?;
            }
            EncryptionState::Encrypted => {
                log::debug!(self.logger, "Database is locked. Verifying old password.");
                println!("\x1b[1;36m db locked verifying old password\x1b[0m");
                // Attempt to decrypt the test value to confirm if password is correct
                let expected_val = Self::encrypt(ENCRYPTION_VERIFICATION_VAL, old_password_hash)?;
                if EncryptionIndicator::verify_password(&expected_val, &conn)? {
                    // Set password to new password
                    self.set_password_hash(new_password_hash, &conn)?;
                } else {
                    return Err(WalletDbError::PasswordFailed);
                }
            }
            EncryptionState::Unencrypted => {
                log::info!(
                    self.logger,
                    "Database is unencrypted. Setting password with new password."
                );
                self.set_password_hash(new_password_hash, &conn)?;
            }
        }
        Ok(())
    }

    /// Encrypt data.
    pub fn encrypt(plaintext_bytes: &[u8], password_hash: &[u8]) -> Result<Vec<u8>, WalletDbError> {
        let (key, nonce) = Self::expand_password(&password_hash)?;
        let cipher = Aes256Gcm::new(&key);
        Ok(cipher.encrypt(&nonce, &plaintext_bytes[..])?)
    }

    /// Decrypt data.
    pub fn decrypt(ciphertext: &[u8], password_hash: &[u8]) -> Result<Vec<u8>, WalletDbError> {
        let (key, nonce) = Self::expand_password(&password_hash)?;
        let cipher = Aes256Gcm::new(&key);
        Ok(cipher.decrypt(&nonce, ciphertext)?)
    }

    /// Expands the password into an encryption key and a nonce.
    fn expand_password(
        password: &[u8],
    ) -> Result<
        (
            GenericArray<u8, <Aes256Gcm as NewAead>::KeySize>,
            GenericArray<u8, <Aes256Gcm as AeadInPlace>::NonceSize>,
        ),
        WalletDbError,
    > {
        // Hash the password hash with Blake2b to get 64 bytes, first 32 for aeskey, second 32 for nonce
        let mut hasher = Blake2b::new();
        hasher.update(&ENCRYPTION_KEY_DOMAIN_TAG);
        hasher.update(&password);
        let result = hasher.finalize();

        let (key, remainder) = Split::<u8, <Aes256Gcm as NewAead>::KeySize>::split(result);
        let (nonce, _remainder) =
            Split::<u8, <Aes256Gcm as AeadInPlace>::NonceSize>::split(remainder);

        Ok((key, nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    // Encrypting and decrypting with a set password should succeed.
    #[test]
    fn test_basic_encrypt_decrypt() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let mut password_hash = [1u8; 32];
        rng.fill_bytes(&mut password_hash);

        let plaintext = b"test_plaintext";
        let ciphertext = WalletDb::encrypt(plaintext, &password_hash).unwrap();
        let decrypted = WalletDb::decrypt(&ciphertext, &password_hash).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

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
            .change_password(&password_hash, &new_password_hash)
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

    // FIXME: test for upgrade path
}
