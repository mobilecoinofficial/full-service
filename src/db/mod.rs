// Copyright (c) 2020 MobileCoin Inc.

//! Implementations of DB and DB models.

pub mod account;
pub mod account_txo_status;
pub mod assigned_subaddress;
pub mod models;
pub mod schema;
pub mod transaction_log;
pub mod txo;

use mc_account_keys::PublicAddress;
use mc_common::logger::Logger;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

use crate::error::WalletDbError;

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
    logger: Logger,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>, logger: Logger) -> Self {
        Self { pool, logger }
    }

    pub fn new_from_url(database_url: &str, logger: Logger) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool, logger))
    }

    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        Ok(self.pool.get()?)
    }
}
