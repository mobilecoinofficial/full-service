// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::PublicAddress;
use mc_common::logger::{log, Logger};
use mc_transaction_core::ring_signature::KeyImage;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    RunQueryDsl,
};

use crate::db_models::transaction_log::TransactionLogModel;
use crate::error::WalletDbError;
use crate::models::{AssignedSubaddress, TransactionLog, Txo};
// Schema Tables
use crate::schema::account_txo_statuses as schema_account_txo_statuses;
use crate::schema::accounts as schema_accounts;
use crate::schema::assigned_subaddresses as schema_assigned_subaddresses;
use crate::schema::txos as schema_txos;

// Query Objects
use crate::schema::account_txo_statuses::dsl::account_txo_statuses as dsl_account_txo_statuses;
use crate::schema::accounts::dsl::accounts as dsl_accounts;
use crate::schema::txos::dsl::txos as dsl_txos;

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

    /// List all subaddresses for a given account.
    pub fn list_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<AssignedSubaddress>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<AssignedSubaddress> = schema_accounts::table
            .inner_join(
                schema_assigned_subaddresses::table.on(schema_accounts::account_id_hex
                    .eq(schema_assigned_subaddresses::account_id_hex)
                    .and(schema_accounts::account_id_hex.eq(account_id_hex))),
            )
            .select(schema_assigned_subaddresses::all_columns)
            .load(&conn)?;

        Ok(results)
    }

    // FIXME: goes on Account
    pub fn update_spent_and_increment_next_block(
        &self,
        account_id_hex: &str,
        spent_block_height: i64,
        key_images: Vec<KeyImage>,
    ) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        for key_image in key_images {
            // Get the txo by key_image
            let matches = schema_txos::table
                .select(schema_txos::all_columns)
                .filter(schema_txos::key_image.eq(mc_util_serial::encode(&key_image)))
                .load::<Txo>(&conn)?;

            if matches.len() == 0 {
                // Not Found is ok - this means it's a key_image not associated with any of our txos
                continue;
            } else if matches.len() > 1 {
                return Err(WalletDbError::DuplicateEntries(format!(
                    "Key Image: {:?}",
                    key_image
                )));
            } else {
                // Update the TXO
                log::trace!(
                    self.logger,
                    "Updating spent for account {:?} at block height {:?} with key_image {:?}",
                    account_id_hex,
                    spent_block_height,
                    key_image
                );
                diesel::update(dsl_txos.find(&matches[0].txo_id_hex))
                    .set(schema_txos::spent_block_height.eq(Some(spent_block_height)))
                    .execute(&conn)?;

                // Update the AccountTxoStatus
                diesel::update(
                    dsl_account_txo_statuses.find((account_id_hex, &matches[0].txo_id_hex)),
                )
                .set(schema_account_txo_statuses::txo_status.eq("spent".to_string()))
                .execute(&conn)?;

                // FIXME: make sure the path for all txo_statuses and txo_types exist and are tested
                // Update the transaction status if the txos are all spent
                TransactionLog::update_transactions_associated_to_txo(
                    &matches[0].txo_id_hex,
                    spent_block_height,
                    &conn,
                )?;
            }
        }
        diesel::update(dsl_accounts.find(account_id_hex))
            .set(schema_accounts::next_block.eq(spent_block_height + 1))
            .execute(&conn)?;
        Ok(())
    }

    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        Ok(self.pool.get()?)
    }
}
