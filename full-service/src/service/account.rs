// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing accounts.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::Account,
        WalletDbError,
    },
    service::{ledger::LedgerService, WalletService},
};
use mc_account_keys::RootEntropy;
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_util_from_random::FromRandom;

use crate::service::ledger::LedgerServiceError;
use diesel::Connection;
use displaydoc::Display;

#[derive(Display, Debug)]
pub enum AccountServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Diesel error: {0}
    Diesel(diesel::result::Error),

    /// Error with the Ledger Service: {0}
    LedgerService(LedgerServiceError),
}

impl From<WalletDbError> for AccountServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<mc_ledger_db::Error> for AccountServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<hex::FromHexError> for AccountServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<diesel::result::Error> for AccountServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<LedgerServiceError> for AccountServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// accounts.
pub trait AccountService {
    /// Creates a new account with default values.
    fn create_account(
        &self,
        name: Option<String>,
        first_block_index: Option<u64>,
    ) -> Result<Account, AccountServiceError>;

    /// Import an existing account to the wallet using the entropy.
    fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
    ) -> Result<Account, AccountServiceError>;

    /// List accounts in the wallet.
    fn list_accounts(&self) -> Result<Vec<Account>, AccountServiceError>;

    /// Get an account in the wallet.
    fn get_account(&self, account_id: &AccountID) -> Result<Account, AccountServiceError>;

    /// Update the name for an account.
    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, AccountServiceError>;

    /// Delete an account from the wallet.
    fn delete_account(&self, account_id: &AccountID) -> Result<bool, AccountServiceError>;
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
    ) -> Result<Account, AccountServiceError> {
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
            None,
            None,
            None,
            &conn,
        )?;

        let account = Account::get(&account_id, &conn)?;
        Ok(account)
    }

    fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block_index: Option<u64>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
    ) -> Result<Account, AccountServiceError> {
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
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
            &conn,
        )?)
    }

    fn list_accounts(&self) -> Result<Vec<Account>, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(Account::list_all(&conn)?)
    }

    fn get_account(&self, account_id: &AccountID) -> Result<Account, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(Account::get(&account_id, &conn)?)
    }

    fn update_account_name(
        &self,
        account_id: &AccountID,
        name: String,
    ) -> Result<Account, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(conn.transaction::<Account, AccountServiceError, _>(|| {
            Account::get(&account_id, &conn)?.update_name(name, &conn)?;
            Ok(Account::get(&account_id, &conn)?)
        })?)
    }

    fn delete_account(&self, account_id: &AccountID) -> Result<bool, AccountServiceError> {
        log::info!(self.logger, "Deleting account {}", account_id,);

        let conn = self.wallet_db.get_conn()?;
        Account::get(account_id, &conn)?.delete(&conn)?;
        Ok(true)
    }
}
