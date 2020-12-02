// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the AssignedSubaddress model

use crate::db::b58_encode;
use crate::error::WalletDbError;
use crate::models::{AssignedSubaddress, NewAssignedSubaddress};

use crate::schema::assigned_subaddresses as schema_assigned_subaddresses;

use diesel::prelude::*;
use mc_account_keys::{AccountKey, PublicAddress};

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub trait AssignedSubaddressModel {
    fn create(
        public_address: &AccountKey,
        account_id_hex: &str,
        address_book_entry: Option<i64>,
        subaddress_index: u64,
        comment: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;
}

impl AssignedSubaddressModel for AssignedSubaddress {
    fn create(
        account_key: &AccountKey,
        account_id_hex: &str,
        address_book_entry: Option<i64>,
        subaddress_index: u64,
        comment: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        let subaddress = account_key.subaddress(subaddress_index);
        let subaddress_b58 = b58_encode(&subaddress)?;
        let subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &subaddress_b58,
            account_id_hex: &account_id_hex,
            address_book_entry,
            public_address: &mc_util_serial::encode(&subaddress),
            subaddress_index: subaddress_index as i64,
            comment,
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values((&subaddress_entry))
            .execute(conn)?;

        Ok(subaddress_b58)
    }
}
