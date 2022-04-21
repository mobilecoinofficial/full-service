// Copyright (c) 2020-2022 MobileCoin Inc.

//! Service for managing view-only Txos.

use crate::{
    db::{models::ViewOnlyTxo, transaction, view_only_txo::ViewOnlyTxoModel},
    service::txo::TxoServiceError,
    WalletService,
};
use diesel::prelude::*;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

/// Trait defining the ways in which the wallet can interact with and manage
/// view only Txos.
pub trait ViewOnlyTxoService {
    /// List the Txos for a given account in the wallet.
    fn list_view_only_txos(
        &self,
        account_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ViewOnlyTxo>, TxoServiceError>;

    /// set a group of txos as spent.
    fn set_view_only_txos_spent(&self, txo_ids: Vec<String>) -> Result<bool, TxoServiceError>;
}

impl<T, FPR> ViewOnlyTxoService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_view_only_txos(
        &self,
        account_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ViewOnlyTxo>, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(ViewOnlyTxo::list_for_account(
            account_id, limit, offset, &conn,
        )?)
    }

    fn set_view_only_txos_spent(&self, txo_ids: Vec<String>) -> Result<bool, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;
        transaction(&conn, || {
            ViewOnlyTxo::set_spent(txo_ids, &conn)?;
            Ok(true)
        })
    }
}
