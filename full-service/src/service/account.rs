// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing accounts.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::Account,
    },
    error::WalletServiceError,
    service::{ledger::LedgerService, WalletService},
};
use mc_account_keys::RootEntropy;
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_util_from_random::FromRandom;

use diesel::Connection;

/// Trait defining the ways in which the wallet can interact with and manage
/// accounts.
pub trait AccountService {
    /// Creates a new account with default values.
    fn create_account(
        &self,
        name: Option<String>,
        first_block_index: Option<u64>,
    ) -> Result<Account, WalletServiceError>;

    /// Import an existing account to the wallet using the entropy.
    fn import_account_entropy(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
    ) -> Result<Account, WalletServiceError>;

    /// List accounts in the wallet.
    fn list_accounts(&self) -> Result<Vec<Account>, WalletServiceError>;

    /// Get an account in the wallet.
    fn get_account(&self, account_id: &AccountID) -> Result<Account, WalletServiceError>;

    /// Update the name for an account.
    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, WalletServiceError>;

    /// Delete an account from the wallet.
    fn delete_account(&self, account_id: &AccountID) -> Result<bool, WalletServiceError>;
}

impl<T, FPR> AccountService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn create_account(
        &self,
        name: Option<String>,
        first_block_index: Option<u64>,
    ) -> Result<Account, WalletServiceError> {
        log::info!(
            self.logger,
            "Creating account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

        // Generate entropy for the account
        let mut rng = rand::thread_rng();
        let entropy = RootEntropy::from_random(&mut rng);

        // Since we are creating the account from randomness, it is highly unlikely that
        // it would have collided with another account that already received funds. For
        // this reason, start scanning at the current network block index.
        let first_block_index = first_block_index.unwrap_or(self.get_network_block_index()?);

        // The earliest we could start scanning is the current highest block index of
        // the local ledger.
        let import_block_index = self.ledger_db.num_blocks()? - 1;

        let conn = self.wallet_db.get_conn()?;
        let (account_id, _public_address_b58) = Account::create(
            &entropy,
            Some(first_block_index),
            Some(import_block_index),
            &name.unwrap_or_else(|| "".to_string()),
            &conn,
        )?;

        let account = Account::get(&account_id, &conn)?;
        Ok(account)
    }

    fn import_account_entropy(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
    ) -> Result<Account, WalletServiceError> {
        log::info!(
            self.logger,
            "Importing account {:?} with first block: {:?}",
            name,
            first_block_index,
        );
        // Get account key from entropy
        let mut entropy_bytes = [0u8; 32];
        hex::decode_to_slice(entropy, &mut entropy_bytes)?;

        // We record the local highest block index because that is the earliest we could
        // start scanning.
        let import_block = self.ledger_db.num_blocks()? - 1;

        let conn = self.wallet_db.get_conn()?;
        Ok(Account::import(
            &RootEntropy::from(&entropy_bytes),
            name,
            import_block,
            first_block_index,
            &conn,
        )?)
    }

    fn list_accounts(&self) -> Result<Vec<Account>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(Account::list_all(&conn)?)
    }

    fn get_account(&self, account_id: &AccountID) -> Result<Account, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(Account::get(&account_id, &conn)?)
    }

    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(conn.transaction::<Account, WalletServiceError, _>(|| {
            Account::get(&account_id, &conn)?.update_name(name, &conn)?;
            Ok(Account::get(&account_id, &conn)?)
        })?)
    }

    fn delete_account(&self, account_id: &AccountID) -> Result<bool, WalletServiceError> {
        log::info!(self.logger, "Deleting account {}", account_id,);

        let conn = self.wallet_db.get_conn()?;
        Account::get(account_id, &conn)?.delete(&conn)?;
        Ok(true)
    }
}
