// Copyright (c) 2020-2021 MobileCoin Inc.

//! Full Service Wallet.

#![feature(proc_macro_hygiene, decl_macro)]

pub mod config;
mod db;
mod error;
mod json_rpc;
mod service;
mod util;

pub use db::WalletDb;
pub use json_rpc::wallet;
pub use service::WalletService;

extern crate alloc;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[allow(unused_imports)] // Needed for json!
#[macro_use]
extern crate rocket_contrib;
#[allow(unused_imports)] // Needed for embedded_migrations!
#[macro_use]
extern crate diesel_migrations;

#[cfg(any(test, feature = "test_utils"))]
mod test_utils;
