// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing balances.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{Account, Txo, TXO_ORPHANED, TXO_PENDING, TXO_SECRETED, TXO_SPENT, TXO_UNSPENT},
        txo::TxoModel,
    },
    error::WalletServiceError,
    service::WalletService,
};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_ledger_sync::NetworkState;

/*
use displaydoc::Display;
/// Errors for the Balance Service.
#[derive(Display, Debug)]
pub enum BalanceServiceError {}
*/

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
    pub network_block_count: u64,
    pub local_block_count: u64,
    pub synced_blocks: u64,
}

/// Trait defining the ways in which the wallet can interact with and manage
/// balances.
pub trait BalanceService {
    /// Gets the balance for a given account.
    ///
    /// Balance consists of the sums of the various txo states in our wallet
    fn get_balance(&self, account_id_hex: &str) -> Result<Balance, WalletServiceError>;
}

impl<T, FPR> BalanceService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_balance(&self, account_id_hex: &str) -> Result<Balance, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;

        let unspent = Txo::list_by_status(account_id_hex, TXO_UNSPENT, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let spent = Txo::list_by_status(account_id_hex, TXO_SPENT, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let secreted = Txo::list_by_status(account_id_hex, TXO_SECRETED, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let orphaned = Txo::list_by_status(account_id_hex, TXO_ORPHANED, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();
        let pending = Txo::list_by_status(account_id_hex, TXO_PENDING, &conn)?
            .iter()
            .map(|t| t.value as u128)
            .sum::<u128>();

        let network_state = self.network_state.read().expect("lock poisoned");
        // network_height = network_block_index + 1
        let network_height = network_state
            .highest_block_index_on_network()
            .map(|v| v + 1)
            .unwrap_or(0);

        let local_block_count = self.ledger_db.num_blocks()?;
        let account = Account::get(&AccountID(account_id_hex.to_string()), &conn)?;

        Ok(Balance {
            unspent: unspent as u64,
            pending: pending as u64,
            spent: spent as u64,
            secreted: secreted as u64,
            orphaned: orphaned as u64,
            network_block_count: network_height,
            local_block_count,
            synced_blocks: account.next_block as u64,
        })
    }
}
