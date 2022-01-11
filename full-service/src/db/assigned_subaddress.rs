// Copyright (c) 2020-2021 MobileCoin Inc.

//! A subaddress assigned to a particular contact for the purpose of tracking
//! funds received from that contact.

use crate::db::{
    account::{AccountID, AccountModel},
    models::{Account, AssignedSubaddress, NewAssignedSubaddress, Txo},
    txo::TxoModel,
};

use crate::util::b58::b58_encode_public_address;

use mc_transaction_core::{
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
};

use mc_account_keys::AccountKey;
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_ledger_db::{Ledger, LedgerDB};

use crate::db::WalletDbError;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};

pub trait AssignedSubaddressModel {
    /// Assign a subaddress to a contact.
    ///
    /// Inserts an AssignedSubaddress to the DB.
    ///
    /// # Arguments
    /// * `account_key` - An account's private keys.
    /// * `address_book_entry` -
    /// * `subaddress_index` -
    /// * `comment` -
    /// * `conn` -
    ///
    /// # Returns
    /// * assigned_subaddress_b58
    fn create(
        account_key: &AccountKey,
        address_book_entry: Option<i64>,
        subaddress_index: u64,
        comment: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;

    /// Create the next subaddress for a given account.
    ///
    /// Returns:
    /// * (assigned_subaddress_b58, subaddress_index)
    fn create_next_for_account(
        account_id_hex: &str,
        comment: &str,
        ledger_db: &LedgerDB,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(String, i64), WalletDbError>;

    /// Get the AssignedSubaddress for a given assigned_subaddress_b58
    fn get(
        public_address_b58: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssignedSubaddress, WalletDbError>;

    /// Get the Assigned Subaddress for a given index in an account, if it
    /// exists
    fn get_for_account_by_index(
        account_id_hex: &str,
        index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssignedSubaddress, WalletDbError>;

    /// Find an AssignedSubaddress by the subaddress spend public key
    ///
    /// Returns:
    /// * (subaddress_index, assigned_subaddress_b58)
    fn find_by_subaddress_spend_public_key(
        subaddress_spend_public_key: &RistrettoPublic,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(i64, String), WalletDbError>;

    /// List all AssignedSubaddresses for a given account.
    fn list_all(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<AssignedSubaddress>, WalletDbError>;

    /// Delete all AssignedSubaddresses for a given account.
    fn delete_all(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl AssignedSubaddressModel for AssignedSubaddress {
    fn create(
        account_key: &AccountKey,
        address_book_entry: Option<i64>,
        subaddress_index: u64,
        comment: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        use crate::db::schema::assigned_subaddresses;

        let account_id = AccountID::from(account_key);

        let subaddress = account_key.subaddress(subaddress_index);
        let subaddress_b58 = b58_encode_public_address(&subaddress)?;

        let subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &subaddress_b58,
            account_id_hex: &account_id.to_string(),
            address_book_entry,
            public_address: &mc_util_serial::encode(&subaddress),
            subaddress_index: subaddress_index as i64,
            comment,
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(assigned_subaddresses::table)
            .values(&subaddress_entry)
            .execute(conn)?;
        Ok(subaddress_b58)
    }

    fn create_next_for_account(
        account_id_hex: &str,
        comment: &str,
        ledger_db: &LedgerDB,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(String, i64), WalletDbError> {
        use crate::db::schema::{
            accounts::dsl::{account_id_hex as dsl_account_id_hex, accounts},
            assigned_subaddresses,
            transaction_logs::dsl::{
                account_id_hex as tx_log_account_id_hex,
                transaction_id_hex as tx_log_transaction_id_hex, transaction_logs,
            },
        };

        let account = Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        let account_view_key = account_key.view_key();
        let subaddress_index = account.next_subaddress_index;
        let subaddress = account_key.subaddress(subaddress_index as u64);

        let subaddress_b58 = b58_encode_public_address(&subaddress)?;
        let subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &subaddress_b58,
            account_id_hex,
            address_book_entry: None, /* FIXME: WS-8 - Address Book Entry if details
                                       * provided, or None always for main? */
            public_address: &mc_util_serial::encode(&subaddress),
            subaddress_index: subaddress_index as i64,
            comment,
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(assigned_subaddresses::table)
            .values(&subaddress_entry)
            .execute(conn)?;

        // Update the next subaddress index for the account
        diesel::update(accounts.filter(dsl_account_id_hex.eq(account_id_hex)))
            .set((crate::db::schema::accounts::next_subaddress_index.eq(subaddress_index + 1),))
            .execute(conn)?;

        // Find and repair orphaned txos at this subaddress.
        let orphaned_txos = Txo::list_orphaned(account_id_hex, conn)?;

        for orphaned_txo in orphaned_txos.iter() {
            let tx_out_target_key: RistrettoPublic =
                mc_util_serial::decode(&orphaned_txo.target_key).unwrap();
            let tx_public_key: RistrettoPublic =
                mc_util_serial::decode(&orphaned_txo.public_key).unwrap();
            let txo_public_key = CompressedRistrettoPublic::from(tx_public_key);

            let txo_subaddress_spk: RistrettoPublic = recover_public_subaddress_spend_key(
                &account_view_key.view_private_key,
                &tx_out_target_key,
                &tx_public_key,
            );

            if txo_subaddress_spk == *subaddress.spend_public_key() {
                let onetime_private_key = recover_onetime_private_key(
                    &tx_public_key,
                    account_key.view_private_key(),
                    &account_key.subaddress_spend_private(subaddress_index as u64),
                );

                let key_image = KeyImage::from(&onetime_private_key);

                if ledger_db.contains_key_image(&key_image)? {
                    let txo_index = ledger_db.get_tx_out_index_by_public_key(&txo_public_key)?;
                    let block_index = ledger_db.get_block_index_by_tx_out_index(txo_index)?;
                    diesel::update(orphaned_txo)
                        .set(
                            crate::db::schema::txos::spent_block_index.eq(Some(block_index as i64)),
                        )
                        .execute(conn)?;
                }

                let key_image_bytes = mc_util_serial::encode(&key_image);

                // Update the account status mapping.
                diesel::update(orphaned_txo)
                    .set((
                        crate::db::schema::txos::subaddress_index.eq(subaddress_index),
                        crate::db::schema::txos::key_image.eq(key_image_bytes),
                    ))
                    .execute(conn)?;

                diesel::update(
                    transaction_logs
                        .filter(tx_log_transaction_id_hex.eq(&orphaned_txo.txo_id_hex))
                        .filter(tx_log_account_id_hex.eq(account_id_hex)),
                )
                .set(
                    (crate::db::schema::transaction_logs::assigned_subaddress_b58
                        .eq(&subaddress_b58),),
                )
                .execute(conn)?;
            }
        }

        Ok((subaddress_b58, subaddress_index))
    }

    fn get(
        public_address_b58: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssignedSubaddress, WalletDbError> {
        use crate::db::schema::assigned_subaddresses::dsl::{
            assigned_subaddress_b58, assigned_subaddresses,
        };

        let assigned_subaddress: AssignedSubaddress = match assigned_subaddresses
            .filter(assigned_subaddress_b58.eq(&public_address_b58))
            .get_result::<AssignedSubaddress>(conn)
        {
            Ok(t) => t,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::AssignedSubaddressNotFound(
                    public_address_b58.to_string(),
                ));
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        Ok(assigned_subaddress)
    }

    fn get_for_account_by_index(
        account_id_hex: &str,
        index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssignedSubaddress, WalletDbError> {
        let account = Account::get(&AccountID(account_id_hex.to_string()), conn)?;

        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
        let subaddress = account_key.subaddress(index as u64);

        let subaddress_b58 = b58_encode_public_address(&subaddress)?;
        Self::get(&subaddress_b58, conn)
    }

    fn find_by_subaddress_spend_public_key(
        subaddress_spend_public_key: &RistrettoPublic,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(i64, String), WalletDbError> {
        use crate::db::schema::assigned_subaddresses::{
            account_id_hex, dsl::assigned_subaddresses, subaddress_index, subaddress_spend_key,
        };

        let matches = assigned_subaddresses
            .select((subaddress_index, account_id_hex))
            .filter(subaddress_spend_key.eq(mc_util_serial::encode(subaddress_spend_public_key)))
            .load::<(i64, String)>(conn)?;

        if matches.is_empty() {
            Err(WalletDbError::AssignedSubaddressNotFound(format!(
                "{:?}",
                subaddress_spend_public_key
            )))
        } else if matches.len() > 1 {
            Err(WalletDbError::DuplicateEntries(format!(
                "{:?}",
                subaddress_spend_public_key
            )))
        } else {
            Ok(matches[0].clone())
        }
    }

    fn list_all(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<AssignedSubaddress>, WalletDbError> {
        use crate::db::schema::assigned_subaddresses::{
            account_id_hex as schema_account_id_hex, all_columns, dsl::assigned_subaddresses,
        };

        let addresses_query = assigned_subaddresses
            .select(all_columns)
            .filter(schema_account_id_hex.eq(account_id_hex));

        let addresses: Vec<AssignedSubaddress> = if let (Some(o), Some(l)) = (offset, limit) {
            addresses_query.offset(o).limit(l).load(conn)?
        } else {
            addresses_query.load(conn)?
        };

        Ok(addresses)
    }

    fn delete_all(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::assigned_subaddresses::dsl::{
            account_id_hex as schema_account_id_hex, assigned_subaddresses,
        };

        diesel::delete(assigned_subaddresses.filter(schema_account_id_hex.eq(account_id_hex)))
            .execute(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_subaddress_by_spk(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);

        let subaddress_b58 =
            AssignedSubaddress::create(&account_key, None, 0, "", &wallet_db.get_conn().unwrap())
                .unwrap();

        let expected_subaddress = account_key.subaddress(0);
        let expected_subaddress_spk = expected_subaddress.spend_public_key();
        let matching = AssignedSubaddress::find_by_subaddress_spend_public_key(
            expected_subaddress_spk,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(matching.0, 0);
        assert_eq!(matching.1, AccountID::from(&account_key).to_string());
        assert_eq!(
            subaddress_b58,
            b58_encode_public_address(&expected_subaddress).unwrap()
        );
    }
}
