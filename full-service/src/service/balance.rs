// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing balances.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{
            Account, Txo, TXO_STATUS_ORPHANED, TXO_STATUS_PENDING, TXO_STATUS_SECRETED,
            TXO_STATUS_SPENT, TXO_STATUS_UNSPENT,
        },
        txo::TxoModel,
    },
    error::WalletServiceError,
    service::WalletService,
};
use mc_common::HashMap;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    Connection,
};

/// The balance object returned by balance services.
///
/// This must be a service object because there is no "Balance" table in our
/// data model.
pub struct Balance {
    pub unspent: u64,
    pub pending: u64,
    pub spent: u64,
    pub secreted: u64,
    pub orphaned: u64,
    pub network_block_index: u64,
    pub local_block_index: u64,
    pub synced_blocks: u64,
}

/// The Wallet Status object returned by balance services.
///
/// This must be a service object because there is no "WalletStatus" table in
/// our data model.
///
/// It shares several fields with balance, but also returns details about the
/// accounts in the wallet.
pub struct WalletStatus {
    pub unspent: u64,
    pub pending: u64,
    pub spent: u64,
    pub secreted: u64,
    pub orphaned: u64,
    pub network_block_index: u64,
    pub local_block_index: u64,
    pub min_synced_block_index: u64,
    pub account_ids: Vec<AccountID>,
    pub account_map: HashMap<AccountID, Account>,
}

/// Trait defining the ways in which the wallet can interact with and manage
/// balances.
pub trait BalanceService {
    /// Gets the balance for a given account.
    ///
    /// Balance consists of the sums of the various txo states in our wallet
    fn get_balance_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<Balance, WalletServiceError>;

    /*
    fn get_balance_for_address(
        &self,
        account_id: &AccountID,
        b58_address: String,
    ) -> Result<Balance, WalletServiceError>;

     */

    fn get_wallet_status(&self) -> Result<WalletStatus, WalletServiceError>;
}

impl<T, FPR> BalanceService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_balance_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<Balance, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let account_id_hex = &account_id.to_string();

        let (unspent, pending, spent, secreted, orphaned) =
            Self::get_balance_inner(account_id_hex, &conn)?;

        let network_block_index = self.get_network_block_index()? + 1;
        let local_block_index = self.ledger_db.num_blocks()?;
        let account = Account::get(account_id, &conn)?;

        Ok(Balance {
            unspent,
            pending,
            spent,
            secreted,
            orphaned,
            network_block_index,
            local_block_index,
            synced_blocks: account.next_block as u64,
        })
    }

    /*
    fn get_balance_for_address(
        &self,
        account_id: &AccountID,
        b58_address: String,
    ) -> Result<Balance, WalletServiceError> {

    }*/

    // Wallet Status is an overview of the wallet's status
    fn get_wallet_status(&self) -> Result<WalletStatus, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let network_block_index = self.get_network_block_index()?;

        Ok(conn.transaction::<WalletStatus, WalletServiceError, _>(|| {
            let accounts = Account::list_all(&conn)?;
            let mut account_map = HashMap::default();

            let mut unspent = 0;
            let mut pending = 0;
            let mut spent = 0;
            let mut secreted = 0;
            let mut orphaned = 0;

            let mut min_synced_block_index = network_block_index;
            let mut account_ids = Vec::new();
            for account in accounts {
                let account_id = AccountID(account.account_id_hex.clone());
                let balance = Self::get_balance_inner(&account_id.to_string(), &conn)?;
                account_map.insert(account_id.clone(), account.clone());
                unspent += balance.0;
                pending += balance.1;
                spent += balance.2;
                secreted += balance.3;
                orphaned += balance.4;

                // account.next_block is an index in range [0..ledger_db.num_blocks()]
                min_synced_block_index = std::cmp::min(
                    min_synced_block_index,
                    (account.next_block as u64).saturating_sub(1),
                );
                account_ids.push(account_id);
            }

            Ok(WalletStatus {
                unspent: unspent as u64,
                pending: pending as u64,
                spent: spent as u64,
                secreted: secreted as u64,
                orphaned: orphaned as u64,
                network_block_index: network_block_index + 1,
                local_block_index: self.ledger_db.num_blocks()?,
                min_synced_block_index: min_synced_block_index as u64,
                account_ids,
                account_map,
            })
        })?)
    }
}

impl<T, FPR> WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_balance_inner(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(u64, u64, u64, u64, u64), WalletServiceError> {
        let unspent = Txo::list_by_status(account_id_hex, TXO_STATUS_UNSPENT, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let spent = Txo::list_by_status(account_id_hex, TXO_STATUS_SPENT, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let secreted = Txo::list_by_status(account_id_hex, TXO_STATUS_SECRETED, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let orphaned = Txo::list_by_status(account_id_hex, TXO_STATUS_ORPHANED, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let pending = Txo::list_by_status(account_id_hex, TXO_STATUS_PENDING, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();

        let result = (
            unspent as u64,
            pending as u64,
            spent as u64,
            secreted as u64,
            orphaned as u64,
        );

        Ok(result)
    }
}
