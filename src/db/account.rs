// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Account model.

use crate::{
    db::{
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AccountTxoStatus, AssignedSubaddress, NewAccount, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
    },
    error::WalletDbError,
};

use mc_account_keys::{AccountKey, PublicAddress, DEFAULT_SUBADDRESS_INDEX};
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::ring_signature::KeyImage;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};

#[derive(Debug)]
pub struct AccountID(String);

impl From<&AccountKey> for AccountID {
    fn from(src: &AccountKey) -> AccountID {
        let main_subaddress = src.subaddress(DEFAULT_SUBADDRESS_INDEX);
        /// The account ID is derived from the contents of the account key
        #[derive(Digestible)]
        struct ConstAccountData {
            /// The public address of the main subaddress for this account
            pub address: PublicAddress,
        }
        let const_data = ConstAccountData {
            address: main_subaddress.clone(),
        };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"account_data");
        Self(hex::encode(temp))
    }
}

impl AccountID {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub trait AccountModel {
    /// Create an account.
    fn create(
        account_key: &AccountKey,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        first_block: u64,
        next_block: u64,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(String, String), WalletDbError>;

    /// List all accounts.
    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError>;

    /// Get a specific account.
    fn get(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError>;

    fn get_by_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError>;

    /// Update an account.
    /// The only updatable field is the name. Any other desired update requires adding
    /// a new account, and deleting the existing if desired.
    fn update_name(
        &self,
        new_name: String,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    fn update_spent_and_increment_next_block(
        &self,
        spent_block_height: i64,
        key_images: Vec<KeyImage>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Delete an account.
    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl AccountModel for Account {
    fn create(
        account_key: &AccountKey,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        first_block: u64,
        next_block: u64,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(String, String), WalletDbError> {
        use crate::db::schema::accounts;

        let account_id = AccountID::from(account_key);

        let new_account = NewAccount {
            account_id_hex: &account_id.to_string(),
            encrypted_account_key: &mc_util_serial::encode(account_key), // FIXME: add encryption
            main_subaddress_index: main_subaddress_index as i64,
            change_subaddress_index: change_subaddress_index as i64,
            next_subaddress_index: next_subaddress_index as i64,
            first_block: first_block as i64,
            next_block: next_block as i64,
            name,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .execute(conn)?;

        let main_subaddress_b58 = AssignedSubaddress::create(
            account_key,
            None, // FIXME: Address Book Entry if details provided, or None always for main?
            main_subaddress_index,
            "Main",
            &conn,
        )?;

        let _change_subaddress_b58 = AssignedSubaddress::create(
            account_key,
            None, // FIXME: Address Book Entry if details provided, or None always for main?
            change_subaddress_index,
            "Change",
            &conn,
        )?;

        Ok((account_id.to_string(), main_subaddress_b58))
    }

    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError> {
        use crate::db::schema::accounts;

        Ok(accounts::table
            .select(accounts::all_columns)
            .load::<Account>(conn)?)
    }

    fn get(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts::dsl::{account_id_hex as dsl_account_id_hex, accounts};

        match accounts
            .filter(dsl_account_id_hex.eq(account_id_hex))
            .get_result::<Account>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::NotFound(account_id_hex.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_by_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::account_txo_statuses::dsl::account_txo_statuses;

        match account_txo_statuses
            .select(crate::db::schema::account_txo_statuses::all_columns)
            .filter(crate::db::schema::account_txo_statuses::txo_id_hex.eq(txo_id_hex))
            .load::<AccountTxoStatus>(conn)
        {
            Ok(a) => {
                if a.len() > 1 {
                    return Err(WalletDbError::MultipleStatusesForTxo);
                }
                Ok(Account::get(&a[0].account_id_hex, conn)?)
            }
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::NotFound(txo_id_hex.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn update_name(
        &self,
        new_name: String,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts::dsl::{account_id_hex, accounts};

        diesel::update(accounts.filter(account_id_hex.eq(&self.account_id_hex)))
            .set(crate::db::schema::accounts::name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }

    fn update_spent_and_increment_next_block(
        &self,
        spent_block_height: i64,
        key_images: Vec<KeyImage>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::account_txo_statuses::dsl::account_txo_statuses;
        use crate::db::schema::accounts::dsl::{account_id_hex, accounts};
        use crate::db::schema::txos::dsl::{txo_id_hex, txos};

        for key_image in key_images {
            // Get the txo by key_image
            let matches = crate::db::schema::txos::table
                .select(crate::db::schema::txos::all_columns)
                .filter(crate::db::schema::txos::key_image.eq(mc_util_serial::encode(&key_image)))
                .load::<Txo>(conn)?;

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
                diesel::update(txos.filter(txo_id_hex.eq(&matches[0].txo_id_hex)))
                    .set(crate::db::schema::txos::spent_block_height.eq(Some(spent_block_height)))
                    .execute(conn)?;

                // Update the AccountTxoStatus
                diesel::update(
                    account_txo_statuses.find((&self.account_id_hex, &matches[0].txo_id_hex)),
                )
                .set(crate::db::schema::account_txo_statuses::txo_status.eq("spent".to_string()))
                .execute(conn)?;

                // FIXME: make sure the path for all txo_statuses and txo_types exist and are tested
                // Update the transaction status if the txos are all spent
                TransactionLog::update_transactions_associated_to_txo(
                    &matches[0].txo_id_hex,
                    spent_block_height,
                    conn,
                )?;
            }
        }
        diesel::update(accounts.filter(account_id_hex.eq(&self.account_id_hex)))
            .set(crate::db::schema::accounts::next_block.eq(spent_block_height + 1))
            .execute(conn)?;
        Ok(())
    }

    /// Delete an account.
    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts::dsl::{account_id_hex, accounts};

        diesel::delete(accounts.filter(account_id_hex.eq(self.account_id_hex))).execute(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);
        let account_id_hex = {
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) =
                Account::create(&account_key, 0, 1, 2, 0, 1, "Alice's Main Account", &conn)
                    .unwrap();
            account_id_hex
        };

        {
            let conn = wallet_db.get_conn().unwrap();
            let res = Account::list_all(&conn).unwrap();
            assert_eq!(res.len(), 1);
        }

        let acc = Account::get(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        let expected_account = Account {
            id: 1,
            account_id_hex: account_id_hex.clone(),
            encrypted_account_key: mc_util_serial::encode(&account_key),
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block: 0,
            next_block: 1,
            name: "Alice's Main Account".to_string(),
        };
        assert_eq!(expected_account, acc);

        // Verify that the subaddress table entries were updated for main and change
        let subaddresses =
            AssignedSubaddress::list_all(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(subaddresses.len(), 2);
        let subaddress_indices: HashSet<i64> =
            HashSet::from_iter(subaddresses.iter().map(|s| s.subaddress_index));
        assert!(subaddress_indices.get(&0).is_some());
        assert!(subaddress_indices.get(&1).is_some());

        // Verify that we can get the correct subaddress index from the spend public key
        let main_subaddress = account_key.subaddress(0);
        let (retrieved_index, retrieved_acocunt_id_hex) =
            AssignedSubaddress::find_by_subaddress_spend_public_key(
                main_subaddress.spend_public_key(),
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();
        assert_eq!(retrieved_index, 0);
        assert_eq!(retrieved_acocunt_id_hex, account_id_hex);

        // Add another account with no name, scanning from later
        let account_key_secondary = AccountKey::from(&RootIdentity::from_random(&mut rng));
        let (account_id_hex_secondary, _public_address_b58_secondary) = Account::create(
            &account_key_secondary,
            0,
            1,
            2,
            50,
            51,
            "",
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let res = Account::list_all(&wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(res.len(), 2);

        let acc_secondary =
            Account::get(&account_id_hex_secondary, &wallet_db.get_conn().unwrap()).unwrap();
        let mut expected_account_secondary = Account {
            id: 2,
            account_id_hex: account_id_hex_secondary.clone(),
            encrypted_account_key: mc_util_serial::encode(&account_key_secondary),
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block: 50,
            next_block: 51,
            name: "".to_string(),
        };
        assert_eq!(expected_account_secondary, acc_secondary);

        // Update the name for the secondary account
        acc_secondary
            .update_name(
                "Alice's Secondary Account".to_string(),
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();
        let acc_secondary2 =
            Account::get(&account_id_hex_secondary, &wallet_db.get_conn().unwrap()).unwrap();
        expected_account_secondary.name = "Alice's Secondary Account".to_string();
        assert_eq!(expected_account_secondary, acc_secondary2);

        // Delete the secondary account
        acc_secondary
            .delete(&wallet_db.get_conn().unwrap())
            .unwrap();

        let res = Account::list_all(&wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(res.len(), 1);

        // Attempt to get the deleted account
        let res = Account::get(&account_id_hex_secondary, &wallet_db.get_conn().unwrap());
        match res {
            Ok(_) => panic!("Should have deleted account"),
            Err(WalletDbError::NotFound(s)) => assert_eq!(s, account_id_hex_secondary.to_string()),
            Err(_) => panic!("Should error with NotFound but got {:?}", res),
        }
    }
}
