// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing confirmation numbers.

use std::ops::DerefMut;

use crate::{
    db::{
        account::AccountID,
        models::Txo,
        txo::{TxoID, TxoModel},
        WalletDbError,
    },
    service::{
        transaction_log::{TransactionLogService, TransactionLogServiceError},
        txo::{TxoService, TxoServiceError},
    },
    WalletService,
};
use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_extra::TxOutConfirmationNumber;

/// Errors for the Txo Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
pub enum ConfirmationServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Minted Txo should contain confirmation: {0}
    MissingConfirmation(String),

    /// Error with the TxoService: {0}
    TxoService(TxoServiceError),

    /// Error with the TxoService: {0}
    TransactionLogService(TransactionLogServiceError),
}

impl From<WalletDbError> for ConfirmationServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for ConfirmationServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_ledger_db::Error> for ConfirmationServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_util_serial::DecodeError> for ConfirmationServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<hex::FromHexError> for ConfirmationServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<TxoServiceError> for ConfirmationServiceError {
    fn from(src: TxoServiceError) -> Self {
        Self::TxoService(src)
    }
}

impl From<TransactionLogServiceError> for ConfirmationServiceError {
    fn from(src: TransactionLogServiceError) -> Self {
        Self::TransactionLogService(src)
    }
}

#[derive(Debug)]
pub struct Confirmation {
    pub txo_id: TxoID,
    pub txo_index: u64,
    pub confirmation: TxOutConfirmationNumber,
}

/// Trait defining the ways in which the wallet can interact with and manage
/// confirmation numbers.
#[rustfmt::skip]
#[allow(clippy::result_large_err)]
pub trait ConfirmationService {
    /// Get the confirmations from the outputs in a transaction log.
    ///
    /// # Arguments
    ///
    ///| Name                 | Purpose                                                       | Notes                                         |
    ///|----------------------|---------------------------------------------------------------|-----------------------------------------------|
    ///| `transaction_log_id` | The transaction log ID for which to get confirmation numbers. | The transaction log must exist in the wallet. |
    ///
    fn get_confirmations(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<Confirmation>, ConfirmationServiceError>;

    /// Validate the confirmation number with a given Txo.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                          | Notes                                                                             |
    ///|--------------------|------------------------------------------------------------------|-----------------------------------------------------------------------------------|
    ///| `account_id`       | The account on which to perform this action.                     | Account must exist in the wallet.                                                 |
    ///| `txo_id`           | The ID of the TXO for which to validate the confirmation number. | TXO must be a received TXO.                                                       |
    ///| `confirmation_hex` | The confirmation number to validate.                             | The confirmation number should be delivered by the sender of the Txo in question. |
    ///
    fn validate_confirmation(
        &self,
        account_id: &AccountID,
        txo_id: &TxoID,
        confirmation_hex: &str,
    ) -> Result<bool, ConfirmationServiceError>;
}

impl<T, FPR> ConfirmationService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_confirmations(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<Confirmation>, ConfirmationServiceError> {
        let (_transaction_log, associated_txos, _value_map) =
            self.get_transaction_log(transaction_log_id)?;

        let mut results = Vec::new();
        for (associated_txo, _) in associated_txos.outputs {
            let txo_info = self.get_txo(&TxoID(associated_txo.id.clone()))?;
            if let Some(confirmation) = txo_info.txo.confirmation {
                let confirmation: TxOutConfirmationNumber = mc_util_serial::decode(&confirmation)?;
                let pubkey: CompressedRistrettoPublic =
                    mc_util_serial::decode(&txo_info.txo.public_key)?;
                let txo_index = self.ledger_db.get_tx_out_index_by_public_key(&pubkey)?;
                results.push(Confirmation {
                    txo_id: TxoID(txo_info.txo.id),
                    txo_index,
                    confirmation,
                });
            } else {
                return Err(ConfirmationServiceError::MissingConfirmation(
                    associated_txo.id,
                ));
            }
        }
        Ok(results)
    }

    fn validate_confirmation(
        &self,
        account_id: &AccountID,
        txo_id: &TxoID,
        confirmation_hex: &str,
    ) -> Result<bool, ConfirmationServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let confirmation: TxOutConfirmationNumber =
            mc_util_serial::decode(&hex::decode(confirmation_hex)?)?;
        Ok(Txo::validate_confirmation(
            &AccountID(account_id.to_string()),
            &txo_id.to_string(),
            &confirmation,
            conn,
        )?)
    }
}
