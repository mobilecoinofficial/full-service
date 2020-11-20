// Copyright (c) 2020 MobileCoin Inc.

#![feature(proc_macro_hygiene, decl_macro)]

mod db;
pub mod models;
pub mod schema;
pub mod service;
mod service_impl;

extern crate alloc;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate rocket_contrib;
