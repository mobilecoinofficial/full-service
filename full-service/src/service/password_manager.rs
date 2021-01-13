// Copyright (c) 2020 MobileCoin Inc.

//! Service for managing passwords.

use crate::{
    db::{
        account::AccountModel,
        encryption_indicator::{EncryptionModel, EncryptionState},
        encryption_provider::EncryptionProvider,
        models::{Account, EncryptionIndicator},
    },
    error::WalletDbError,
    service::WalletService,
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_connection::FogPubkeyResolver;

use blake2::{Blake2b, Digest};
use diesel::Connection;
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

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),
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

impl From<diesel::result::Error> for PasswordServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
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
            return Err(PasswordServiceError::CannotDisambiguatePassword);
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
        let password_hash = self.get_password_hash(password, password_hash)?;

        let conn_manager = self.wallet_db.get_conn_manager()?;
        conn_manager.conn.transaction::<(), PasswordServiceError, _>(|| {
            match EncryptionIndicator::get_encryption_state(&conn_manager.conn)? {
                EncryptionState::Empty => {
                    log::info!(
                        self.logger,
                        "Database has never been locked and has no accounts. Setting password for future accounts."
                    );
                    self.wallet_db.set_password_hash(&password_hash, &conn_manager.conn)?;
                }
                EncryptionState::Encrypted => {
                    return Err(PasswordServiceError::DatabaseEncrypted);
                }
                EncryptionState::Unencrypted => {
                    log::info!(
                        self.logger,
                        "Database is unencrypted. Setting password with new password, and encrypting all accounts."
                    );
                    self.wallet_db.set_password_hash(&password_hash, &conn_manager.conn)?;
                    for account in Account::list_all(&conn_manager.conn)? {
                        let encrypted_account_key = conn_manager.encryption_provider.encrypt(
                            &account.account_key,
                        )?;
                        account.update_encrypted_account_key(&encrypted_account_key, &conn_manager.conn)?;
                    }
                }
            }
            Ok(())
        })?;
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

        // Re-encrypt all of our accounts with the new password hash
        let conn_manager = self.wallet_db.get_conn_manager()?;
        conn_manager
            .conn
            .transaction::<(), PasswordServiceError, _>(|| {
                for account in Account::list_all(&conn_manager.conn)? {
                    // Get decrypted with current password
                    let decrypted_account_key = account.get_decrypted_account_key(&conn_manager)?;

                    // Encrypt for the new password
                    let encrypted_account_key = EncryptionProvider::encrypt_with_password(
                        &mc_util_serial::encode(&decrypted_account_key),
                        &new_password_hash,
                    )?;
                    account
                        .update_encrypted_account_key(&encrypted_account_key, &conn_manager.conn)?;
                }
                // Set the new password for the whole DB
                self.wallet_db.change_password(
                    &old_password_hash,
                    &new_password_hash,
                    &conn_manager.conn,
                )?;
                Ok(())
            })?;

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
            return Err(PasswordServiceError::DatabaseLocked);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::account::AccountID,
        test_utils::{get_test_ledger, setup_service},
    };
    use mc_account_keys::{AccountKey, PublicAddress, RootEntropy, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_set_password_hash(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db.clone(), logger);

        // Service should be "never yet locked" on startup
        assert_eq!(service.is_locked().unwrap(), None);
        assert!(service.verify_unlocked().is_err());

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
        assert_eq!(service.is_locked().unwrap(), Some(false));
        assert!(service.verify_unlocked().is_ok());
    }

    #[test_with_logger]
    fn test_change_password(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db.clone(), logger);

        let mut password_hash = [0u8; 32];
        rng.fill_bytes(&mut password_hash);
        service
            .set_password(None, Some(hex::encode(&password_hash)))
            .unwrap();

        // Create Account
        let alice_account_resp = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Account key should be encrypted and retrieved with get_decrypted
        let mut entropy_bytes = [0u8; 32];
        hex::decode_to_slice(alice_account_resp.entropy, &mut entropy_bytes).unwrap();
        let alice_account_key =
            AccountKey::from(&RootIdentity::from(&RootEntropy::from(&entropy_bytes)));
        let alice_account = Account::get(
            &AccountID(alice_account_resp.account.account_id.clone()),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let alice_decrypted = alice_account
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(alice_decrypted, alice_account_key);

        // Change password
        let mut new_password_hash = [0u8; 32];
        rng.fill_bytes(&mut new_password_hash);
        service
            .change_password(
                None,
                Some(hex::encode(password_hash)),
                None,
                Some(hex::encode(new_password_hash)),
            )
            .unwrap();
        assert_eq!(
            service.wallet_db.get_password_hash().unwrap(),
            new_password_hash
        );

        // Verify that the newly encrypted account key also decrypts to the same value
        let alice_account_changed = Account::get(
            &AccountID(alice_account_resp.account.account_id.clone()),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let alice_decrypted_changed = alice_account_changed
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(alice_decrypted_changed, alice_account_key);

        // Add a few more accounts, to make sure that we update all accounts in the DB on password change
        let bob_account_resp = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();
        let carol_account_resp = service
            .create_account(Some("Carol's Main Account".to_string()), None)
            .unwrap();

        // Account key should be encrypted and retrieved with get_decrypted
        let mut entropy_bytes = [0u8; 32];
        hex::decode_to_slice(bob_account_resp.entropy, &mut entropy_bytes).unwrap();
        let bob_account_key =
            AccountKey::from(&RootIdentity::from(&RootEntropy::from(&entropy_bytes)));
        let bob_account = Account::get(
            &AccountID(bob_account_resp.account.account_id.clone()),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let bob_decrypted = bob_account
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(bob_decrypted, bob_account_key);

        let mut entropy_bytes = [0u8; 32];
        hex::decode_to_slice(carol_account_resp.entropy, &mut entropy_bytes).unwrap();
        let carol_account_key =
            AccountKey::from(&RootIdentity::from(&RootEntropy::from(&entropy_bytes)));
        let carol_account = Account::get(
            &AccountID(carol_account_resp.account.account_id.clone()),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let carol_decrypted = carol_account
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(carol_decrypted, carol_account_key);

        // Change password
        let mut final_password_hash = [0u8; 32];
        rng.fill_bytes(&mut final_password_hash);
        service
            .change_password(
                None,
                Some(hex::encode(new_password_hash)),
                None,
                Some(hex::encode(final_password_hash)),
            )
            .unwrap();
        assert_eq!(
            service.wallet_db.get_password_hash().unwrap(),
            final_password_hash
        );

        // Verify that all the newly encrypted account keys also decrypt to the same value
        let alice_account_final = Account::get(
            &AccountID(alice_account_resp.account.account_id),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let alice_decrypted_final = alice_account_final
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(alice_decrypted_final, alice_account_key);

        let bob_account_final = Account::get(
            &AccountID(bob_account_resp.account.account_id),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let bob_decrypted_final = bob_account_final
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(bob_decrypted_final, bob_account_key);

        let carol_account_final = Account::get(
            &AccountID(carol_account_resp.account.account_id),
            &service.wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let carol_decrypted_final = carol_account_final
            .get_decrypted_account_key(&service.wallet_db.get_conn_manager().unwrap())
            .unwrap();
        assert_eq!(carol_decrypted_final, carol_account_key);
    }
}
