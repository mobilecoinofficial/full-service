use crate::db::WalletDbError;
use crate::service::transaction_builder_error::WalletTransactionBuilderError;
use displaydoc::Display;

#[derive(Display, Debug)]
pub enum WalletServiceError {
    /// Error interacting with the DB {0}
    Database(WalletDbError),

    /// Error decoding from hex {0}
    HexDecode(hex::FromHexError),

    /// Error building transaction {0}
    TransactionBuilder(WalletTransactionBuilderError),

    /// Error parsing u64
    U64Parse,

    /// No peers configured
    NoPeersConfigured,

    /// Node not found
    NodeNotFound,

    /// Error converting json {0}
    JsonConversion(String),

    /// Connection Error
    Connection(retry::Error<mc_connection::Error>),

    /// Error converting to/from API protos {0}
    ProtoConversion(mc_api::ConversionError),

    /// Error Converting Proto but throws convert::Infallible
    ProtoConversionInfallible,

    /// Error with LedgerDB {0}
    LedgerDB(mc_ledger_db::Error),

    /// No transaction object associated with this transaction. Note, received transactions do not have transaction objects.
    NoTxInTransaction,

    /// Error decoding prost {0}
    ProstDecode(prost::DecodeError),

    /// Error serializing json {0}
    SerdeJson(serde_json::Error),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Txo should contain proof: {0}
    MissingProof(String),
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
