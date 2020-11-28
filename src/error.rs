// Copyright (c) 2020 MobileCoin Inc.

//! Errors for the wallet service

use displaydoc::Display;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::response::Response;
use rocket::Request;
use std::io::Cursor;

#[derive(Display, Debug)]
pub enum WalletServiceError {
    /// Error interacting with the DB {0}
    DBError(WalletDbError),

    /// Error decoding from hex {0}
    HexDecode(hex::FromHexError),
}

impl From<WalletDbError> for WalletServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::DBError(src)
    }
}

impl From<hex::FromHexError> for WalletServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

#[derive(Display, Debug)]
pub enum WalletDbError {
    /// Diesel Error: {0}
    DieselError(diesel::result::Error),

    /// Error with rocket databases: {0}
    RocketDB(rocket_contrib::databases::r2d2::Error),

    /// Duplicate entries with the same ID {0}
    DuplicateEntries(String),

    /// Entry not found
    NotFound(String),

    /// Error encoding b58 {0}
    B58EncodeError(mc_api::display::Error),
}

impl From<diesel::result::Error> for WalletDbError {
    fn from(src: diesel::result::Error) -> Self {
        Self::DieselError(src)
    }
}

impl From<rocket_contrib::databases::r2d2::Error> for WalletDbError {
    fn from(src: rocket_contrib::databases::r2d2::Error) -> Self {
        Self::RocketDB(src)
    }
}

impl From<mc_api::display::Error> for WalletDbError {
    fn from(src: mc_api::display::Error) -> Self {
        Self::B58EncodeError(src)
    }
}

#[derive(Display, Debug)]
pub enum WalletAPIError {
    /// Error Parsing u64
    U64ParseError,

    /// Error interacting with Wallet Service {0}
    WalletService(WalletServiceError),
}

impl Responder<'static> for WalletAPIError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        Response::build()
            .sized_body(Cursor::new(format!("{:?}", self)))
            .ok()
    }
}

impl From<WalletServiceError> for WalletAPIError {
    fn from(src: WalletServiceError) -> Self {
        Self::WalletService(src)
    }
}

impl From<std::num::ParseIntError> for WalletAPIError {
    fn from(_src: std::num::ParseIntError) -> Self {
        Self::U64ParseError
    }
}

#[derive(Display, Debug)]
pub enum SyncError {
    /// Error downloading blocks
    DownloadError,

    /// Could not find account
    AccountNotFound,

    /// Error with WalletDb {0}
    WalletDb(WalletDbError),

    /// Error with LedgerDB {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error with Keys {0}
    CryptoKey(mc_crypto_keys::KeyError),

    /// Error decoding prost {0}
    ProstDecode(prost::DecodeError),
}

impl From<WalletDbError> for SyncError {
    fn from(src: WalletDbError) -> Self {
        Self::WalletDb(src)
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
