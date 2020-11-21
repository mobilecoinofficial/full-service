// Copyright (c) 2020 MobileCoin Inc.

#![feature(proc_macro_hygiene, decl_macro)]

mod db;
mod error;
pub mod models;
pub mod schema;
pub mod service;
mod service_impl;

pub use db::WalletDb;
pub use service_impl::WalletService;

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
