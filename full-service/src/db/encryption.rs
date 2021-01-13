// Copyright (c) 2020 MobileCoin Inc.

//! Encryption for database fields can be managed with a top level password, through the
//! EncryptionProvider.

use crate::{
    db::{
        encryption_indicator::{EncryptionModel, EncryptionState},
        models::EncryptionIndicator,
    },
    error::WalletDbError,
};

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
    r2d2::{ConnectionManager, PooledConnection},
};

use std::sync::{Arc, Mutex};

/// Domain tag for database-wide encryption.
pub const ENCRYPTION_KEY_DOMAIN_TAG: &str = "mc_full_service";
const ENCRYPTION_VERIFICATION_VAL: &[u8] = b"verify";

type ExpandedPassword = (
    GenericArray<u8, <Aes256Gcm as NewAead>::KeySize>,
    GenericArray<u8, <Aes256Gcm as AeadInPlace>::NonceSize>,
);

/// Provides global storage for the password hash, as well as convenience encryption methods.
#[derive(Clone)]
pub struct EncryptionProvider {
    password_hash: Arc<Mutex<Vec<u8>>>,
    logger: Logger,
}

impl EncryptionProvider {
    pub fn new(logger: Logger) -> Self {
        Self {
            password_hash: Arc::new(Mutex::new(vec![])),
            logger,
        }
    }

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

    pub fn unlock(
        &self,
        password_hash: &[u8],
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        // No need to check db state if we're already unlocked
        if self.is_unlocked()? {
            return Ok(());
        }

        // Check whether encrypted, and if so, then attempt to unlock
        match EncryptionIndicator::get_encryption_state(&conn)? {
            EncryptionState::Empty => {
                log::info!(
                    self.logger,
                    "DB has never been locked. Please call set_password to enable encryption."
                );
                return Err(WalletDbError::SetPassword);
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
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
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
    fn expand_password(password: &[u8]) -> Result<ExpandedPassword, WalletDbError> {
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
    use mc_crypto_rand::rand_core::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    // Encrypting and decrypting with a set password should succeed.
    #[test]
    fn test_basic_encrypt_decrypt() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let mut password_hash = [1u8; 32];
        rng.fill_bytes(&mut password_hash);

        let plaintext = b"test_plaintext";
        let ciphertext = EncryptionProvider::encrypt(plaintext, &password_hash).unwrap();
        let decrypted = EncryptionProvider::decrypt(&ciphertext, &password_hash).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
