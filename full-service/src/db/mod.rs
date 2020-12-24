// Copyright (c) 2020-2021 MobileCoin Inc.

//! Implementations of DB and DB models.

pub mod account;
pub mod account_txo_status;
pub mod assigned_subaddress;
pub mod locked_indicator;
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
use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
};

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode(b58_public_address: &str) -> Result<PublicAddress, WalletDbError> {
    let wrapper =
        mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(b58_public_address.to_string())
            .unwrap();
    let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
        let payment_request = wrapper.get_payment_request();
        payment_request.get_public_address()
    } else if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        return Err(WalletDbError::B58Decode);
    };
    Ok(PublicAddress::try_from(pubaddr_proto).unwrap())
}

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
    password_hash: Arc<Mutex<Vec<u8>>>,
    logger: Logger,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>, logger: Logger) -> Self {
        Self {
            pool,
            password_hash: Arc::new(Mutex::new(vec![])),
            logger,
        }
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

    pub fn set_password_hash(&self, password_hash: &[u8]) -> Result<(), WalletDbError> {
        let mut cur_password_hash = self.password_hash.lock()?;
        *cur_password_hash = password_hash.to_vec();
        Ok(())
    }

    pub fn get_password_hash(&self) -> Result<Vec<u8>, WalletDbError> {
        let cur_password_hash = self.password_hash.lock()?;
        Ok(cur_password_hash.clone())
    }
}
