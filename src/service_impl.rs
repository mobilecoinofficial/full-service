// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods.

use crate::db::WalletDb;
use crate::error::WalletServiceError;
use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, DEFAULT_SUBADDRESS_INDEX};
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

pub struct WalletService {
    walletdb: WalletDb,
}

impl WalletService {
    pub fn new(walletdb: WalletDb) -> WalletService {
        WalletService { walletdb }
    }
    /// Creates a new account with defaults
    pub fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<(String, String, String), WalletServiceError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_common::logger::{log, test_with_logger, Logger};

    #[test_with_logger]
    fn test_create_account(_logger: Logger) {
        assert!(true);
        //create_account(conn, "Alice's Main Account".to_string(), None).unwrap();
    }
}
