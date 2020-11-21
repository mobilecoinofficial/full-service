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
    /// Error encoding b58 {0}
    B58EncodeError(mc_api::display::Error),

    /// Error interacting with the DB {0}
    DBError(WalletDbError),
}

impl From<WalletDbError> for WalletServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::DBError(src)
    }
}

impl From<mc_api::display::Error> for WalletServiceError {
    fn from(src: mc_api::display::Error) -> Self {
        Self::B58EncodeError(src)
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
