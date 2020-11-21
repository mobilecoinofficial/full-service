// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::{AccountKey, PublicAddress};
use mc_crypto_digestible::{Digestible, MerlinTranscript};

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;

use crate::error::WalletDBError;
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

    pub fn new_from_url(database_url: &str) -> Result<Self, WalletDBError> {
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
    ) -> Result<String, WalletDBError> {
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
    pub fn list_accounts(&self) -> Result<Vec<Account>, WalletDBError> {
        let conn = self.pool.get()?;

        let results: Vec<Account> = schema_accounts::table
            .select(schema_accounts::all_columns)
            .load::<Account>(&conn)?;
        Ok(results)
    }
}

/* Example UPDATE
pub fn publish_post(conn: &SqliteConnection, key: String) -> usize {
    diesel::update(dsl_accounts.find(key))
        .set(published.eq(true))
        .execute(conn)
        .expect("Unable to find post")
}
 */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_account_crud(_logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance();

        // FIXME: eventually mobilecoind test_utils to get the test_environment

        let account_key = AccountKey::from(&RootIdentity::from_random(&mut rng));
        walletdb
            .create_account(&account_key, 0, 1, 2, 0, 1, Some("Alice's Main Account"))
            .unwrap();

        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

        let const_data = ConstAccountData {
            address: account_key.subaddress(0),
            main_subaddress: 0,
            first_block: 0,
        };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"monitor_data");
        let account_id_hex = hex::encode(temp);

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
        assert_eq!(expected_account, res[0]);

        // Add another account with no name, scanning from later
        let account_key_secondary = AccountKey::from(&RootIdentity::from_random(&mut rng));
        walletdb
            .create_account(&account_key_secondary, 0, 1, 2, 50, 51, None)
            .unwrap();
        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 2);

        let const_data = ConstAccountData {
            address: account_key_secondary.subaddress(0),
            main_subaddress: 0,
            first_block: 50,
        };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"monitor_data");
        let account_id_hex_secondary = hex::encode(temp);

        let expected_account_secondary = Account {
            account_id_hex: account_id_hex_secondary,
            encrypted_account_key: mc_util_serial::encode(&account_key_secondary),
            main_subaddress_index: "0".to_string(),
            change_subaddress_index: "1".to_string(),
            next_subaddress_index: "2".to_string(),
            first_block: "50".to_string(),
            next_block: "51".to_string(),
            name: None,
        };
        assert_eq!(expected_account_secondary, res[1]);
    }
}
