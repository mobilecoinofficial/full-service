// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing accounts.

use crate::{
    db::{account::AccountModel, models::Account},
    error::WalletServiceError,
    service::WalletService,
};
use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_util_from_random::FromRandom;

use displaydoc::Display;

/// Errors for the Account Service.
#[derive(Display, Debug)]
pub enum AccountServiceError {}

/// Trait defining the ways in which the wallet can interact with and manage
/// accounts.
pub trait AccountService {
    /// Creates a new account with default values.
    fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<Account, WalletServiceError>;
}

impl<T, FPR> AccountService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<Account, WalletServiceError> {
        log::info!(
            self.logger,
            "Creating account {:?} with first_block: {:?}",
            name,
            first_block,
        );

        // Generate entropy for the account
        let mut rng = rand::thread_rng();
        let entropy = RootEntropy::from_random(&mut rng);

        let conn = self.wallet_db.get_conn()?;
        let (account_id, _public_address_b58) = Account::create(
            &entropy,
            first_block,
            None,
            &name.unwrap_or_else(|| "".to_string()),
            &conn,
        )?;

        let account = Account::get(&account_id, &conn)?;
        Ok(account)
    }
}
