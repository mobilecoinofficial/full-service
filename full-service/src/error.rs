// Copyright (c) 2020-2021 MobileCoin Inc.

//! Errors for the wallet service.

use crate::{
    db::WalletDbError,
    service::{
        account::AccountServiceError, balance::BalanceServiceError,
        confirmation_number::ConfirmationServiceError, gift_code::GiftCodeServiceError,
        ledger::LedgerServiceError, payment_request::PaymentRequestServiceError,
        transaction::TransactionServiceError, transaction_log::TransactionLogServiceError,
        txo::TxoServiceError,
    },
    util::b58::B58Error,
};
use displaydoc::Display;
use hex::FromHexError;

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WalletServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Error parsing u64
    U64Parse,

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error serializing json: {0}
    SerdeJson(serde_json::Error),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with the transaction service: {0}
    TransactionService(TransactionServiceError),

    /// Error with the balance service: {0}
    BalanceService(BalanceServiceError),

    /// Error with the ledger service: {0}
    LedgerService(LedgerServiceError),

    /// Error with the Txo service: {0}
    TxoService(TxoServiceError),

    /// Error with the Proof service: {0}
    ConfirmationService(ConfirmationServiceError),

    /// Error with the TransactionLog service: {0}
    TransactionLogService(TransactionLogServiceError),

    /// Error with the GiftCode service: {0}
    GiftCodeService(GiftCodeServiceError),

    /// Error with the Account service: {0}
    AccountService(AccountServiceError),

    /// Error with the Payment service: {0}
    PaymentRequestService(PaymentRequestServiceError),
}

impl From<WalletDbError> for WalletServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<hex::FromHexError> for WalletServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<TransactionServiceError> for WalletServiceError {
    fn from(src: TransactionServiceError) -> Self {
        Self::TransactionService(src)
    }
}

impl From<BalanceServiceError> for WalletServiceError {
    fn from(src: BalanceServiceError) -> Self {
        Self::BalanceService(src)
    }
}

impl From<LedgerServiceError> for WalletServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

impl From<TxoServiceError> for WalletServiceError {
    fn from(src: TxoServiceError) -> Self {
        Self::TxoService(src)
    }
}

impl From<ConfirmationServiceError> for WalletServiceError {
    fn from(src: ConfirmationServiceError) -> Self {
        Self::ConfirmationService(src)
    }
}

impl From<TransactionLogServiceError> for WalletServiceError {
    fn from(src: TransactionLogServiceError) -> Self {
        Self::TransactionLogService(src)
    }
}

impl From<GiftCodeServiceError> for WalletServiceError {
    fn from(src: GiftCodeServiceError) -> Self {
        Self::GiftCodeService(src)
    }
}

impl From<AccountServiceError> for WalletServiceError {
    fn from(src: AccountServiceError) -> Self {
        Self::AccountService(src)
    }
}

impl From<PaymentRequestServiceError> for WalletServiceError {
    fn from(src: PaymentRequestServiceError) -> Self {
        Self::PaymentRequestService(src)
    }
}

impl From<std::num::ParseIntError> for WalletServiceError {
    fn from(_src: std::num::ParseIntError) -> Self {
        Self::U64Parse
    }
}

impl From<mc_ledger_db::Error> for WalletServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_util_serial::DecodeError> for WalletServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<serde_json::Error> for WalletServiceError {
    fn from(src: serde_json::Error) -> Self {
        Self::SerdeJson(src)
    }
}

impl From<diesel::result::Error> for WalletServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

#[derive(Display, Debug)]
pub enum T3SyncError {
    /// WalletDb: {0}
    WalletDb(WalletDbError),

    /// FromHex: {0}
    FromHex(FromHexError),

    /// T3Connection: {0}
    T3Connection(t3_connection::Error),

    /// mc util serial decode: {0}
    McUtilSerialDecode(mc_util_serial::DecodeError),
}

impl From<WalletDbError> for T3SyncError {
    fn from(src: WalletDbError) -> Self {
        Self::WalletDb(src)
    }
}

impl From<FromHexError> for T3SyncError {
    fn from(src: FromHexError) -> Self {
        Self::FromHex(src)
    }
}

impl From<t3_connection::Error> for T3SyncError {
    fn from(src: t3_connection::Error) -> Self {
        Self::T3Connection(src)
    }
}

impl From<mc_util_serial::DecodeError> for T3SyncError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::McUtilSerialDecode(src)
    }
}

#[derive(Display, Debug)]
pub enum SyncError {
    /// Could not find account
    AccountNotFound,

    /// Error with WalletDb: {0}
    Database(WalletDbError),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error with Keys: {0}
    CryptoKey(mc_crypto_keys::KeyError),

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error with the Amount: {0}
    Amount(mc_transaction_core::AmountError),

    /// Error executing diesel transaction: {0}
    Diesel(diesel::result::Error),

    /// Error with the B58 Util: {0}
    B58(B58Error),
}

impl From<WalletDbError> for SyncError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<mc_ledger_db::Error> for SyncError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_crypto_keys::KeyError> for SyncError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::CryptoKey(src)
    }
}

impl From<mc_util_serial::DecodeError> for SyncError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<mc_transaction_core::AmountError> for SyncError {
    fn from(src: mc_transaction_core::AmountError) -> Self {
        Self::Amount(src)
    }
}

impl From<diesel::result::Error> for SyncError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<B58Error> for SyncError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
    }
}

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WalletTransactionBuilderError {
    /// Insufficient Funds: {0}
    InsufficientFunds(String),

    /// Insufficient Funds in inputs to construct transaction: {0}
    InsufficientInputFunds(String),

    /// Insufficient TxOuts to construct transaction
    InsufficientTxOuts,

    /// Ring size does not match number of inputs
    RingSizeMismatch,

    /// No recipient was specified
    NoRecipient,

    /// Not enough Rings and Proofs
    RingsAndProofsEmpty,

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Tx Builder Error: {0}
    TxBuilder(mc_transaction_builder::TxBuilderError),

    /// Invalid Argument: {0}
    InvalidArgument(String),

    /// Prost decode failed: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Wallet DB Error: {0}
    WalletDb(WalletDbError),

    /// Error interacting with fog: {0}
    FogError(String),

    /// Attempting to build a transaction from a TXO without a subaddress: {0}
    NullSubaddress(String),

    /// Error executing diesel transaction: {0}
    Diesel(diesel::result::Error),

    /// No inputs selected. Must set or select inputs before building.
    NoInputs,

    /// Outbound value + fee exceeds u64::MAX
    OutboundValueTooLarge,

    /**
     * Must set tombstone before building. Setting to 0 picks reasonable
     * default.
     */
    TombstoneNotSet,

    /// Fee must be at least MINIMUM_FEE: {0}
    InsufficientFee(String),

    /// Error parsing URI {0}
    UriParse(mc_util_uri::UriParseError),

    /// Error generating FogPubkeyResolver {0}
    FogPubkeyResolver(String),

    /// Error with the b58 util: {0}
    B58(B58Error),

    /// Error passed up from AmountError: {0}
    AmountError(mc_transaction_core::AmountError),

    /// Error passed up from KeyError: {0}
    KeyError(mc_crypto_keys::KeyError),

    /// Transaction is missing inputs for outputs with token id {0}
    MissingInputsForTokenId(String),

    /// Error decoding the hex string: {0}
    FromHexError(hex::FromHexError),

    /// Burn Redemption Memo must be exactly 128 characters (64 bytes) long: {0}
    InvalidBurnRedemptionMemo(String),

    /// Error converting a TxOut: {0}
    TxOutConversion(mc_transaction_core::TxOutConversionError),

    /// RTH is currently unavailable for view only accounts.
    RTHUnavailableForViewOnlyAccounts,

    /// Cannot use orphaned txo as an input: {0}
    CannotUseOrphanedTxoAsInput(String),

    /**
     * Change amount must be <= u64::MAX, but total change value is: {0}
     */
    ChangeLargerThanMaxValue(u128),
}

impl From<mc_transaction_core::AmountError> for WalletTransactionBuilderError {
    fn from(src: mc_transaction_core::AmountError) -> Self {
        Self::AmountError(src)
    }
}

impl From<mc_crypto_keys::KeyError> for WalletTransactionBuilderError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::KeyError(src)
    }
}

impl From<mc_ledger_db::Error> for WalletTransactionBuilderError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_transaction_builder::TxBuilderError> for WalletTransactionBuilderError {
    fn from(src: mc_transaction_builder::TxBuilderError) -> Self {
        Self::TxBuilder(src)
    }
}

impl From<mc_util_serial::DecodeError> for WalletTransactionBuilderError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<WalletDbError> for WalletTransactionBuilderError {
    fn from(src: WalletDbError) -> Self {
        Self::WalletDb(src)
    }
}

impl From<diesel::result::Error> for WalletTransactionBuilderError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_util_uri::UriParseError> for WalletTransactionBuilderError {
    fn from(src: mc_util_uri::UriParseError) -> Self {
        Self::UriParse(src)
    }
}

impl From<B58Error> for WalletTransactionBuilderError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
    }
}

impl From<hex::FromHexError> for WalletTransactionBuilderError {
    fn from(src: hex::FromHexError) -> Self {
        Self::FromHexError(src)
    }
}

impl From<mc_transaction_core::TxOutConversionError> for WalletTransactionBuilderError {
    fn from(src: mc_transaction_core::TxOutConversionError) -> Self {
        Self::TxOutConversion(src)
    }
}
