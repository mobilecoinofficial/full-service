// Copyright (c) 2020-2021 MobileCoin Inc.

//! Full Service Wallet.

#![feature(proc_macro_hygiene, decl_macro)]
#![feature(assert_matches)]

pub mod check_host;
pub mod config;
pub mod db;
mod error;
pub mod json_rpc;
pub mod service;
pub mod util;
mod validator_ledger_sync;

pub use db::WalletDb;
pub use json_rpc::wallet;
pub use service::WalletService;
pub use validator_ledger_sync::ValidatorLedgerSyncThread;

extern crate alloc;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[allow(unused_imports)] // Needed for embedded_migrations!
#[macro_use]
extern crate diesel_migrations;
#[allow(unused_imports)] // Needed for json!
#[macro_use]
extern crate rocket;

#[cfg(any(test, feature = "test_utils"))]
mod test_utils;
