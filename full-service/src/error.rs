// Copyright (c) 2020-2021 MobileCoin Inc.

//! Errors for the wallet service.

use crate::db::WalletDbError;
use displaydoc::Display;

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WalletServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Error building transaction: {0}
    TransactionBuilder(WalletTransactionBuilderError),

    /// Error parsing u64
    U64Parse,

    /// No peers configured
    NoPeersConfigured,

    /// Node not found
    NodeNotFound,

    /// Error converting json: {0}
    JsonConversion(String),

    /// Connection Error
    Connection(retry::Error<mc_connection::Error>),

    /// Error converting to/from API protos: {0}
    ProtoConversion(mc_api::ConversionError),

    /// Error Converting Proto but throws convert::Infallible
    ProtoConversionInfallible,

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// No transaction object associated with this transaction. Note, received
    /// transactions do not have transaction objects.
    NoTxInTransaction,

    /// Error decoding prost: {0}
    ProstDecode(prost::DecodeError),

    /// Error serializing json: {0}
    SerdeJson(serde_json::Error),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Txo should contain proof: {0}
    MissingProof(String),

    /// Cannot complete this action in offline mode.
    Offline,
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

impl From<WalletTransactionBuilderError> for WalletServiceError {
    fn from(src: WalletTransactionBuilderError) -> Self {
        Self::TransactionBuilder(src)
    }
}

impl From<std::num::ParseIntError> for WalletServiceError {
    fn from(_src: std::num::ParseIntError) -> Self {
        Self::U64Parse
    }
}

impl From<retry::Error<mc_connection::Error>> for WalletServiceError {
    fn from(e: retry::Error<mc_connection::Error>) -> Self {
        Self::Connection(e)
    }
}

impl From<mc_api::ConversionError> for WalletServiceError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<mc_ledger_db::Error> for WalletServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<prost::DecodeError> for WalletServiceError {
    fn from(src: prost::DecodeError) -> Self {
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
    TxBuilder(mc_transaction_std::TxBuilderError),

    /// Invalid Argument: {0}
    InvalidArgument(String),

    /// Prost decode failed: {0}
    ProstDecode(prost::DecodeError),

    /// Wallet DB Error: {0}
    WalletDb(WalletDbError),

    /// Error interacting with fog: {0}
    FogError(String),

    /// Attempting to build a transaction from a TXO without a subaddress: {0}
    NullSubaddress(String),

    /// Full-service transactions can only have one recipient.
    MultipleRecipientsInTransaction,

    /// Error executing diesel transaction: {0}
    Diesel(diesel::result::Error),

    /// No inputs selected. Must set or select inputs before building.
    NoInputs,

    /// Outbound value + fee exceeds u64::MAX
    OutboundValueTooLarge,

    /// Must set tombstone before building. Setting to 0 picks reasonable
    /// default.
    TombstoneNotSet,

    /// Fee must be at least MINIMUM_FEE: {0}
    InsufficientFee(String),

    /// The wallet service only supports transactions with one recipient at this
    /// time.
    MultipleOutgoingRecipients,

    /// Error parsing URI {0}
    UriParse(mc_util_uri::UriParseError),

    /// Error generating FogPubkeyResolver {0}
    FogPubkeyResolver(String),
}

impl From<mc_ledger_db::Error> for WalletTransactionBuilderError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_transaction_std::TxBuilderError> for WalletTransactionBuilderError {
    fn from(src: mc_transaction_std::TxBuilderError) -> Self {
        Self::TxBuilder(src)
    }
}

impl From<prost::DecodeError> for WalletTransactionBuilderError {
    fn from(src: prost::DecodeError) -> Self {
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
