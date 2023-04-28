// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing accounts.

use std::ops::DerefMut;

use crate::{
    db::{assigned_subaddress::AssignedSubaddressModel, models::AssignedSubaddress, WalletDbError},
    service::WalletService,
    util::b58::{b58_decode_public_address, b58_encode_payment_request, B58Error},
};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_transaction_core::Amount;

use crate::service::ledger::LedgerServiceError;
use displaydoc::Display;

#[derive(Display, Debug)]
pub enum PaymentRequestServiceError {
    /// Error interacting with the B58 Util: {0}
    B58(B58Error),

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

    /// Unknown key version version: {0}
    UnknownKeyDerivation(u8),

    /// Invalid BIP39 english mnemonic: {0}
    InvalidMnemonic(String),
}

impl From<WalletDbError> for PaymentRequestServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<B58Error> for PaymentRequestServiceError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
    }
}

impl From<mc_ledger_db::Error> for PaymentRequestServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<hex::FromHexError> for PaymentRequestServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<diesel::result::Error> for PaymentRequestServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<LedgerServiceError> for PaymentRequestServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

#[rustfmt::skip]
pub trait PaymentRequestService {
    /// Create a payment request b58 code to give to someone else.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                          | Notes                             |
    ///|--------------------|------------------------------------------------------------------|-----------------------------------|
    ///| `account_id`       | The account on which to perform this action.                     | Account must exist in the wallet. |
    ///| `subaddress_index` | The subaddress index on the account to generate the request with |                                   |
    ///| `amount`           | The Amount to send in this transaction                           | 64-bit signed integer             |
    ///| `memo`             | Memo for the payment request                                     |                                   |
    ///
    fn create_payment_request(
        &self,
        account_id: String,
        subaddress_index: Option<i64>,
        amount: Amount,
        memo: Option<String>,
    ) -> Result<String, PaymentRequestServiceError>;
}

impl<T, FPR> PaymentRequestService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn create_payment_request(
        &self,
        account_id: String,
        subaddress_index: Option<i64>,
        amount: Amount,
        memo: Option<String>,
    ) -> Result<String, PaymentRequestServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();

        let assigned_subaddress = AssignedSubaddress::get_for_account_by_index(
            &account_id,
            subaddress_index.unwrap_or_default(),
            conn,
        )?;

        let public_address = b58_decode_public_address(&assigned_subaddress.public_address_b58)?;

        let payment_request_b58 =
            b58_encode_payment_request(&public_address, &amount, memo.unwrap_or_default())?;

        Ok(payment_request_b58)
    }
}
