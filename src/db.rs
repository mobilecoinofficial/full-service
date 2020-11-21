// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::{AccountKey, PublicAddress};
use mc_crypto_digestible::{Digestible, MerlinTranscript};

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;

use crate::error::WalletDbError;
use crate::models::{Account, NewAccount};
use crate::schema::accounts as schema_accounts;
use crate::schema::accounts::dsl::accounts as dsl_accounts;

// The account ID is derived from the contents of the account key
#[derive(Digestible)]
struct ConstAccountData {
    pub address: PublicAddress,
    pub main_subaddress: u64,
    pub first_block: u64,
}

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { pool }
    }

    pub fn new_from_url(database_url: &str) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool))
    }

    /// Create a new account.
    pub fn create_account(
        &self,
        account_key: &AccountKey,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        first_block: u64,
        next_block: u64,
        name: Option<&str>,
    ) -> Result<String, WalletDbError> {
        let conn = self.pool.get()?;

        let const_data = ConstAccountData {
            address: account_key.subaddress(main_subaddress_index),
            main_subaddress: main_subaddress_index,
            first_block: first_block,
        };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"monitor_data");
        let account_id_hex = hex::encode(temp);

        // FIXME: how do we want to do optional/defaults and overrides?
        let new_account = NewAccount {
            account_id_hex: &account_id_hex,
            encrypted_account_key: &mc_util_serial::encode(account_key), // FIXME: add encryption
            main_subaddress_index: &main_subaddress_index.to_string(),
            change_subaddress_index: &change_subaddress_index.to_string(),
            next_subaddress_index: &next_subaddress_index.to_string(),
            first_block: &first_block.to_string(),
            next_block: &next_block.to_string(),
            name,
        };

        diesel::insert_into(schema_accounts::table)
            .values(&new_account)
            .execute(&conn)?;

        Ok(account_id_hex)
    }

    /// List all accounts.
    pub fn list_accounts(&self) -> Result<Vec<Account>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<Account> = schema_accounts::table
            .select(schema_accounts::all_columns)
            .load::<Account>(&conn)?;
        Ok(results)
    }

    /// Get a specific account
    pub fn get_account(&self, account_id_hex: &str) -> Result<Account, WalletDbError> {
        let conn = self.pool.get()?;

        let matches = schema_accounts::table
            .select(schema_accounts::all_columns)
            .filter(schema_accounts::account_id_hex.eq(account_id_hex))
            .load::<Account>(&conn)?;

        if matches.len() == 0 {
            Err(WalletDbError::NotFound(account_id_hex.to_string()))
        } else if matches.len() > 1 {
            Err(WalletDbError::DuplicateEntries(account_id_hex.to_string()))
        } else {
            Ok(matches[0].clone())
        }
    }

    /// Update an account.
    /// The only updatable field is the name. Any other desired update requires adding
    /// a new account, and deleting the existing if desired.
    pub fn update_account(
        &self,
        account_id_hex: &str,
        new_name: Option<String>,
    ) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        diesel::update(dsl_accounts.find(account_id_hex))
            .set(schema_accounts::name.eq(new_name))
            .execute(&conn)?;
        Ok(())
    }

    /// Delete an account.
    pub fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletDbError> {
        let conn = self.pool.get()?;

        diesel::delete(dsl_accounts.find(account_id_hex)).execute(&conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_account_crud() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance();

        let account_key = AccountKey::from(&RootIdentity::from_random(&mut rng));
        let account_id_hex = walletdb
            .create_account(&account_key, 0, 1, 2, 0, 1, Some("Alice's Main Account"))
            .unwrap();

        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

        let acc = walletdb.get_account(&account_id_hex).unwrap();
        let expected_account = Account {
            account_id_hex,
            encrypted_account_key: mc_util_serial::encode(&account_key),
            main_subaddress_index: "0".to_string(),
            change_subaddress_index: "1".to_string(),
            next_subaddress_index: "2".to_string(),
            first_block: "0".to_string(),
            next_block: "1".to_string(),
            name: Some("Alice's Main Account".to_string()),
        };
        assert_eq!(expected_account, acc);

        // Add another account with no name, scanning from later
        let account_key_secondary = AccountKey::from(&RootIdentity::from_random(&mut rng));
        let account_id_hex_secondary = walletdb
            .create_account(&account_key_secondary, 0, 1, 2, 50, 51, None)
            .unwrap();
        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 2);

        let acc_secondary = walletdb.get_account(&account_id_hex_secondary).unwrap();
        let mut expected_account_secondary = Account {
            account_id_hex: account_id_hex_secondary.clone(),
            encrypted_account_key: mc_util_serial::encode(&account_key_secondary),
            main_subaddress_index: "0".to_string(),
            change_subaddress_index: "1".to_string(),
            next_subaddress_index: "2".to_string(),
            first_block: "50".to_string(),
            next_block: "51".to_string(),
            name: None,
        };
        assert_eq!(expected_account_secondary, acc_secondary);

        // Update the name for the secondary account
        walletdb
            .update_account(
                &account_id_hex_secondary,
                Some("Alice's Secondary Account".to_string()),
            )
            .unwrap();
        let acc_secondary2 = walletdb.get_account(&account_id_hex_secondary).unwrap();
        expected_account_secondary.name = Some("Alice's Secondary Account".to_string());
        assert_eq!(expected_account_secondary, acc_secondary2);

        // Delete the secondary account
        walletdb.delete_account(&account_id_hex_secondary).unwrap();

        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

        // Attempt to get the deleted account
        let res = walletdb.get_account(&account_id_hex_secondary);
        match res {
            Ok(_) => panic!("Should have deleted account"),
            Err(WalletDbError::NotFound(s)) => assert_eq!(s, account_id_hex_secondary.to_string()),
            Err(_) => panic!("Should error with NotFound"),
        }
    }
}
