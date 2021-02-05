use crate::db::WalletDbError;
use displaydoc::Display;

#[derive(Display, Debug)]
pub enum SyncError {
    /// Error with WalletDb {0}
    Database(WalletDbError),

    /// Error with LedgerDB {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error with Keys {0}
    CryptoKey(mc_crypto_keys::KeyError),

    /// Error decoding prost {0}
    ProstDecode(prost::DecodeError),

    /// Error with the Amount {0}
    Amount(mc_transaction_core::AmountError),

    /// Error executing diesel transaction {0}
    Diesel(diesel::result::Error),
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

impl From<prost::DecodeError> for SyncError {
    fn from(src: prost::DecodeError) -> Self {
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
