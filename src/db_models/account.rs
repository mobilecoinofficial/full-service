// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Account model

use crate::db_models::assigned_subaddress::AssignedSubaddressModel;
use crate::error::WalletDbError;
use crate::models::{Account, AssignedSubaddress, NewAccount};

use mc_account_keys::{AccountKey, PublicAddress, DEFAULT_SUBADDRESS_INDEX};
use mc_crypto_digestible::{Digestible, MerlinTranscript};

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::RunQueryDsl;

// Schema Tables
use crate::schema::accounts as schema_accounts;

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
    fn list_accounts(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError>;
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

        diesel::insert_into(schema_accounts::table)
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

        println!(
            "\x1b[1;32m got main subadddress {:?} and change subaddress {:?}\x1b[0m",
            main_subaddress_b58, _change_subaddress_b58
        );

        Ok((account_id.to_string(), main_subaddress_b58))
    }

    /// List all accounts.
    fn list_accounts(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError> {
        Ok(schema_accounts::table
            .select(schema_accounts::all_columns)
            .load::<Account>(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

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
            let res = Account::list_accounts(&conn).unwrap();
            assert_eq!(res.len(), 1);
        }

        let acc = wallet_db.get_account(&account_id_hex).unwrap();
        let expected_account = Account {
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
    }
}
