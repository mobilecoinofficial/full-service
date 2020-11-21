// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods.

use crate::db::WalletDb;
use crate::error::WalletServiceError;
use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, DEFAULT_SUBADDRESS_INDEX};
use mc_common::logger::{log, Logger};
use mc_util_from_random::FromRandom;

pub const DEFAULT_CHANGE_SUBADDRESS_INDEX: u64 = 1;
pub const DEFAULT_NEXT_SUBADDRESS_INDEX: u64 = 2;
pub const DEFAULT_FIRST_BLOCK: u64 = 0;

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletServiceError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

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
        let public_address = account_key.subaddress(DEFAULT_SUBADDRESS_INDEX);
        // FIXME: Also add public address to assigned_subaddresses table

        let account_id = self.walletdb.create_account(
            &account_key,
            DEFAULT_SUBADDRESS_INDEX,
            DEFAULT_CHANGE_SUBADDRESS_INDEX,
            DEFAULT_NEXT_SUBADDRESS_INDEX,
            first_block.unwrap_or(DEFAULT_FIRST_BLOCK),
            DEFAULT_FIRST_BLOCK + 1,
            name.as_deref(),
        )?;

        Ok((
            entropy_str.to_string(),
            b58_encode(&public_address)?,
            account_id,
        ))
    }

    pub fn list_accounts(&self) -> Result<Vec<String>, WalletServiceError> {
        Ok(self
            .walletdb
            .list_accounts()?
            .iter()
            .map(|a| a.account_id_hex.clone())
            .collect())
    }

    pub fn get_account(&self, account_id_hex: &str) -> Result<Option<String>, WalletServiceError> {
        let account = self.walletdb.get_account(account_id_hex)?;
        Ok(account.name)
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
