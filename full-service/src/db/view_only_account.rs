// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the View Only Account model.

use crate::db::{
    models::{NewViewOnlyAccount, ViewOnlyAccount},
    WalletDbError,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};

use std::str;

pub const DEFAULT_FIRST_BLOCK_INDEX: u64 = 0;
pub const ROOT_ENTROPY_KEY_DERIVATION_VERSION: u8 = 1;
pub const MNEMONIC_KEY_DERIVATION_VERSION: u8 = 2;

pub trait ViewOnlyAccountModel {
    // insert new view-only-account in the db
    fn create(
        view_private_key: Vec<u8>,
        first_block_index: i64,
        import_block_index: i64,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError>;

    // /// List all view-only-accounts.
    // ///
    // /// Returns:
    // /// * Vector of all Accounts in the DB
    // fn list_all(
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    /// Get a specific account.
    ///
    /// Returns:
    /// * Account
    fn get(
        view_private_key: Vec<u8>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError>;

    // /// Get the accounts associated with the given Txo.
    // fn get_by_txo_id(
    //     txo_id_hex: &str,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    // /// Update an account.
    // /// The only updatable field is the name. Any other desired update requires
    // /// adding a new account, and deleting the existing if desired.
    // fn update_name(
    //     &self,
    //     new_name: String,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;

    // /// Update the next block index this account will need to sync.
    // fn update_next_block_index(
    //     &self,
    //     next_block_index: i64,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;

    // /// Delete an account.
    // fn delete(
    //     self,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;
}

impl ViewOnlyAccountModel for ViewOnlyAccount {
    fn create(
        view_private_key: Vec<u8>,
        first_block_index: i64,
        import_block_index: i64,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError> {
        use crate::db::schema::view_only_accounts;

        let new_view_only_account = NewViewOnlyAccount {
            view_private_key: &view_private_key,
            first_block_index,
            // next block index will always be the same as
            // first block index when importing an account
            next_block_index: first_block_index,
            import_block_index,
            name,
        };

        diesel::insert_into(view_only_accounts::table)
            .values(&new_view_only_account)
            .execute(conn.clone())?;

        ViewOnlyAccount::get(
            new_view_only_account.view_private_key.to_vec(),
            conn.clone(),
        )
    }

    fn get(
        view_private_key: Vec<u8>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError> {
        use crate::db::schema::view_only_accounts::dsl::{
            view_only_accounts, view_private_key as dsl_view_key,
        };

        match view_only_accounts
            .filter((dsl_view_key).eq(view_private_key))
            .get_result::<ViewOnlyAccount>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::AccountNotFound(
                // str::from_utf8(view_private_key)?,
                "account not found".to_string(),
            )),
            Err(e) => Err(e.into()),
        }
    }

    // /// Get the accounts associated with the given Txo.
    // fn get_by_txo_id(
    //     txo_id_hex: &str,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    // /// Update an account.
    // /// The only updatable field is the name. Any other desired update requires
    // /// adding a new account, and deleting the existing if desired.
    // fn update_name(
    //     &self,
    //     new_name: String,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;

    // /// Update the next block index this account will need to sync.
    // fn update_next_block_index(
    //     &self,
    //     next_block_index: i64,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;

    // /// Delete an account.
    // fn delete(
    //     self,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;

    use mc_common::logger::{test_with_logger, Logger};

    #[test_with_logger]
    fn test_view_only_account_create_and_get_model(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

        let name = "Coins for cats";
        let view_private_key: Vec<u8> = [1, 2, 3].to_vec();
        let first_block_index: i64 = 25;
        let import_block_index: i64 = 26;

        let expected_account = ViewOnlyAccount {
            id: 1,
            view_private_key: view_private_key.clone(),
            first_block_index,
            next_block_index: first_block_index,
            import_block_index,
            name: name.to_string(),
        };

        let created = ViewOnlyAccount::create(
            view_private_key,
            first_block_index,
            import_block_index,
            &name,
            &conn,
        )
        .unwrap();
        assert_eq!(expected_account, created);
    }
}
