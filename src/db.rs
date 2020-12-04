// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::logger::{log, Logger};
use mc_transaction_core::ring_signature::KeyImage;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    RunQueryDsl,
};

use crate::db_models::transaction_log::TransactionLogModel;
use crate::error::WalletDbError;
use crate::models::{Account, AssignedSubaddress, NewAssignedSubaddress, TransactionLog, Txo};
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

    pub fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: &str,
    ) -> Result<(String, i64), WalletDbError> {
        let conn = self.pool.get()?;

        let account: Account = match dsl_accounts
            .find(account_id_hex)
            .get_result::<Account>(&conn)
        {
            Ok(a) => a,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::NotFound(account_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let account_key: AccountKey = mc_util_serial::decode(&account.encrypted_account_key)?;
        let subaddress_index = account.next_subaddress_index;
        let subaddress = account_key.subaddress(subaddress_index as u64);

        let subaddress_b58 = b58_encode(&subaddress)?;
        let subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &subaddress_b58,
            account_id_hex,
            address_book_entry: None, // FIXME: Address Book Entry if details provided, or None always for main?
            public_address: &mc_util_serial::encode(&subaddress),
            subaddress_index: subaddress_index as i64,
            comment,
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&subaddress_entry)
            .execute(&conn)?;
        // Update the next subaddress index for the account
        // Note: we also update the first_block back to 0 to scan from the beginning of the
        //       ledger for this new subaddress.
        // FIXME: pass in a "sync from" block rather than 0
        let sync_from = 0;
        diesel::update(dsl_accounts.find(account_id_hex))
            .set((
                schema_accounts::next_subaddress_index.eq(subaddress_index + 1),
                schema_accounts::next_block.eq(sync_from),
            ))
            .execute(&conn)?;

        // Update the next subaddress index for the account
        diesel::update(dsl_accounts.find(account_id_hex))
            .set((schema_accounts::next_subaddress_index.eq(subaddress_index + 1),))
            .execute(&conn)?;

        // FIXME: Currently getting 2020-11-30 04:15:15.758107 UTC ERRO error syncing monitor 873b295eb48dccd3612b64ec69d5dc26c0923aa3545a783c6423354bea5de3e8:
        // WalletDb(DieselError(DatabaseError(UniqueViolation, "UNIQUE constraint failed: transaction_logs.transaction_id_hex"))),
        // mc.app: wallet-service, mc.module: mc_wallet_service::sync, mc.src: src/sync.rs:282
        //        when creating a new subaddress to try to pick up orphaned transactions

        // FIXME: When adding multiple addresses for syncing, possibly due to error above, the
        //        syncing stops, and the account is stuck at 0
        Ok((subaddress_b58, subaddress_index))
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
