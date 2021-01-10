// Copyright (c) 2020 MobileCoin Inc.

//! Service for managing passwords.

use crate::{
    db::{
        account::AccountModel,
        encryption_indicator::{EncryptionModel, EncryptionState},
        models::{Account, EncryptionIndicator},
        WalletDb,
    },
    error::WalletDbError,
    service::WalletService,
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_connection::FogPubkeyResolver;

use blake2::{Blake2b, Digest};
use displaydoc::Display;

const SALT_DOMAIN_TAG: &str = "full-service-salt";

#[derive(Display, Debug)]
pub enum PasswordServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Cannot set password on encrypted database. Must change_password.
    DatabaseEncrypted,

    /// Cannot perform this action without a set password or while database is locked. Please set_password or unlock first.
    DatabaseLocked,

    /// Must provide either password or password hash, not both.
    CannotDisambiguatePassword,

    /// Argon2 Error: {0}
    Argon2(argon2::Error),
}

impl From<WalletDbError> for PasswordServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<hex::FromHexError> for PasswordServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<argon2::Error> for PasswordServiceError {
    fn from(src: argon2::Error) -> Self {
        Self::Argon2(src)
    }
}

pub trait PasswordService {
    // Helper method to expand password to password hash using argon2.
    fn get_password_hash(
        &self,
        password: Option<String>,
        password_hash: Option<String>,
    ) -> Result<Vec<u8>, PasswordServiceError>;

    /// The initial call to set the password for the DB.
    fn set_password(
        &self,
        password: Option<String>,
        password_hash: Option<String>,
    ) -> Result<bool, PasswordServiceError>;

    /// Unlock the DB.
    fn unlock(
        &self,
        password: Option<String>,
        password_hash: Option<String>,
    ) -> Result<bool, PasswordServiceError>;

    /// Change the password for the DB.
    fn change_password(
        &self,
        old_password: Option<String>,
        old_password_hash: Option<String>,
        new_password: Option<String>,
        new_password_hash: Option<String>,
    ) -> Result<bool, PasswordServiceError>;

    /// Whether the database is locked.
    ///
    /// Returns:
    /// * Some(true) if database is locked
    /// * Some(false) if database is unlocked
    /// * None if database has not yet had a password set up.
    fn is_locked(&self) -> Result<Option<bool>, PasswordServiceError>;

    /// Utility method to call on service methods which require the database to be unlocked
    fn verify_unlocked(&self) -> Result<(), PasswordServiceError>;
}

impl<T, FPR> PasswordService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_password_hash(
        &self,
        password: Option<String>,
        password_hash: Option<String>,
    ) -> Result<Vec<u8>, PasswordServiceError> {
        if (password.is_some() && password_hash.is_some())
            || (password.is_none() && password_hash.is_none())
        {
            return Err(PasswordServiceError::CannotDisambiguatePassword.into());
        }
        Ok(if let Some(pw) = password {
            // Get the salt from password.
            // Note: We use a deterministic salt so that we can derive the same hash from the same text
            //       string. This is ok for our use case, see discussion on precomputation attacks:
            //       https://crypto.stackexchange.com/questions/77549/is-it-safe-to-use-a-deterministic-salt-as-an-input-to-kdf-argon2
            let mut hasher = Blake2b::new();
            hasher.update(&SALT_DOMAIN_TAG);
            hasher.update(&pw);
            let salt = hasher.finalize();
            let config = argon2::Config::default();
            argon2::hash_raw(pw.as_bytes(), &salt, &config)?
        } else {
            hex::decode(password_hash.unwrap())?
        })
    }

    fn set_password(
        &self,
        password: Option<String>,
        password_hash: Option<String>,
    ) -> Result<bool, PasswordServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let password_hash = self.get_password_hash(password, password_hash)?;

        // FIXME: put in db transaction
        match EncryptionIndicator::get_encryption_state(&conn)? {
            EncryptionState::Empty => {
                log::info!(
                    self.logger,
                    "Database has never been locked and has no accounts. Setting password for future accounts."
                );
                self.wallet_db.set_password_hash(&password_hash, &conn)?;
            }
            EncryptionState::Encrypted => {
                return Err(PasswordServiceError::DatabaseEncrypted.into());
            }
            EncryptionState::Unencrypted => {
                log::info!(
                    self.logger,
                    "Database is unencrypted. Setting password with new password, and encrypting all accounts."
                );
                self.wallet_db.set_password_hash(&password_hash, &conn)?;
                for account in Account::list_all(&conn)? {
                    let encrypted_account_key = WalletDb::encrypt(
                        &account.account_key,
                        &self.wallet_db.get_password_hash()?,
                    )?;
                    account.update_encrypted_account_key(&encrypted_account_key, &conn)?;
                }
            }
        }
        Ok(true)
    }

    fn unlock(
        &self,
        password: Option<String>,
        password_hash: Option<String>,
    ) -> Result<bool, PasswordServiceError> {
        let password_hash = self.get_password_hash(password, password_hash)?;

        self.wallet_db.unlock(&password_hash)?;
        Ok(true)
    }

    fn change_password(
        &self,
        old_password: Option<String>,
        old_password_hash: Option<String>,
        new_password: Option<String>,
        new_password_hash: Option<String>,
    ) -> Result<bool, PasswordServiceError> {
        let old_password_hash = self.get_password_hash(old_password, old_password_hash)?;
        let new_password_hash = self.get_password_hash(new_password, new_password_hash)?;

        // FIXME: logic to convert password to password hash
        self.wallet_db
            .change_password(&old_password_hash, &new_password_hash)?;
        // Re-encrypt all of our accounts with the new password hash
        // FIXME: put in db transaction
        let conn = self.wallet_db.get_conn()?;
        for account in Account::list_all(&conn)? {
            let decrypted_account_key =
                account.get_decrypted_account_key(&old_password_hash, &conn)?;

            let encrypted_account_key = WalletDb::encrypt(
                &mc_util_serial::encode(&decrypted_account_key),
                &self.wallet_db.get_password_hash()?,
            )?;
            account.update_encrypted_account_key(&encrypted_account_key, &conn)?;
        }

        Ok(true)
    }

    fn is_locked(&self) -> Result<Option<bool>, PasswordServiceError> {
        Ok(
            match EncryptionIndicator::get_encryption_state(&self.wallet_db.get_conn()?)? {
                EncryptionState::Empty => None,
                EncryptionState::Encrypted => Some(!self.wallet_db.is_unlocked()?),
                EncryptionState::Unencrypted => Some(false),
            },
        )
    }

    fn verify_unlocked(&self) -> Result<(), PasswordServiceError> {
        if !self.wallet_db.is_unlocked()? {
            return Err(PasswordServiceError::DatabaseLocked.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_test_ledger, setup_service};
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_db_set_password_hash(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db.clone(), logger);

        // Should unlock while DB is empty, and stored password_hash will be empty
        let mut password_hash = [0u8; 32];
        rng.fill_bytes(&mut password_hash);
        let res = service
            .set_password(None, Some(hex::encode(&password_hash)))
            .unwrap();
        assert!(res);
        assert_eq!(
            service.wallet_db.get_password_hash().unwrap(),
            password_hash.to_vec()
        );
    }
}
