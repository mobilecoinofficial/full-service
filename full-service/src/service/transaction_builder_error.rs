use displaydoc::Display;
use crate::db::WalletDbError;

#[derive(Display, Debug)]
pub enum WalletTransactionBuilderError {
    /// Insufficient Funds {0}
    InsufficientFunds(String),

    /// Insufficient Funds in inputs to construct transaction {0}
    InsufficientInputFunds(String),

    /// Insufficient TxOuts to construct transaction
    InsufficientTxOuts,

    /// Ring size does not match number of inputs
    RingSizeMismatch,

    /// No recipient was specified
    NoRecipient,

    /// Not enough Rings and Proofs
    RingsAndProofsEmpty,

    /// Error with LedgerDB {0}
    LedgerDB(mc_ledger_db::Error),

    /// Tx Builder Error {0}
    TxBuilder(mc_transaction_std::TxBuilderError),

    /// Invalid Argument {0}
    InvalidArgument(String),

    /// Prost decode failed {0}
    ProstDecode(prost::DecodeError),

    /// Wallet DB Error {0}
    WalletDb(WalletDbError),

    /// Error interacting with fog {0}
    FogError(String),

    /// Attempting to build a transaction from a TXO without a subaddress {0}
    NullSubaddress(String),

    /// Full-service transactions can only have one recipient.
    MultipleRecipientsInTransaction,

    /// Error executing diesel transaction {0}
    Diesel(diesel::result::Error),

    /// No inputs selected. Must set or select inputs before building.
    NoInputs,

    /// Outbound value + fee exceeds u64::MAX
    OutboundValueTooLarge,

    /// Must set tombstone before building. Setting to 0 picks reasonable default.
    TombstoneNotSet,

    /// Fee must be at least MINIMUM_FEE {0}
    InsufficientFee(String),

    /// The wallet service only supports transactions with one recipient at this time.
    MultipleOutgoingRecipients,
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
