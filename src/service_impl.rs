// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods.

use crate::db::WalletDb;
use crate::error::WalletServiceError;
use mc_account_keys::{AccountKey, RootIdentity, DEFAULT_SUBADDRESS_INDEX};
use mc_common::logger::{log, Logger};
use mc_util_from_random::FromRandom;

pub const DEFAULT_CHANGE_SUBADDRESS_INDEX: u64 = 1;
pub const DEFAULT_NEXT_SUBADDRESS_INDEX: u64 = 2;
pub const DEFAULT_FIRST_BLOCK: u64 = 0;

/// Service for interacting with the wallet
pub struct WalletService {
    walletdb: WalletDb,
    logger: Logger,
}

impl WalletService {
    pub fn new(walletdb: WalletDb, logger: Logger) -> WalletService {
        WalletService { walletdb, logger }
    }
    /// Creates a new account with defaults
    pub fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<(String, String, String), WalletServiceError> {
        log::info!(
            self.logger,
            "Creating account {:?} with first_block: {:?}",
            name,
            first_block,
        );
        // Generate entropy for the account
        let mut rng = rand::thread_rng();
        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id.clone());
        let entropy_str = hex::encode(root_id.root_entropy);

        let (account_id, public_address_b58) = self.walletdb.create_account(
            &account_key,
            DEFAULT_SUBADDRESS_INDEX,
            DEFAULT_CHANGE_SUBADDRESS_INDEX,
            DEFAULT_NEXT_SUBADDRESS_INDEX,
            first_block.unwrap_or(DEFAULT_FIRST_BLOCK),
            DEFAULT_FIRST_BLOCK + 1,
            &name.unwrap_or("".to_string()),
        )?;

        Ok((entropy_str.to_string(), public_address_b58, account_id))
    }

    pub fn list_accounts(&self) -> Result<Vec<String>, WalletServiceError> {
        Ok(self
            .walletdb
            .list_accounts()?
            .iter()
            .map(|a| a.account_id_hex.clone())
            .collect())
    }

    pub fn get_account(&self, account_id_hex: &str) -> Result<String, WalletServiceError> {
        let account = self.walletdb.get_account(account_id_hex)?;
        Ok(account.name)
    }

    pub fn update_account_name(
        &self,
        account_id_hex: &str,
        name: String,
    ) -> Result<(), WalletServiceError> {
        self.walletdb.update_account_name(account_id_hex, name)?;
        Ok(())
    }

    pub fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletServiceError> {
        self.walletdb.delete_account(account_id_hex)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};

    fn setup_service(logger: Logger) -> WalletService {
        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance();

        WalletService::new(walletdb, logger)
    }

    #[test_with_logger]
    fn test_create_account(logger: Logger) {
        let service = setup_service(logger);
        let _account_details = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // FIXME: TODO - assert other things that should be true with the service state
        //        after an account has been created, such as the balance, etc
    }
}
