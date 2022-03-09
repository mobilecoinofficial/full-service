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
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use std::{fmt, str};

pub const DEFAULT_FIRST_BLOCK_INDEX: u64 = 0;
pub const ROOT_ENTROPY_KEY_DERIVATION_VERSION: u8 = 1;
pub const MNEMONIC_KEY_DERIVATION_VERSION: u8 = 2;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ViewOnlyAccountID(pub String);

impl From<&RistrettoPrivate> for ViewOnlyAccountID {
    fn from(src: &RistrettoPrivate) -> ViewOnlyAccountID {
        let view_public_key = RistrettoPublic::from(src);
        let temp: [u8; 32] = view_public_key.digest32::<MerlinTranscript>(b"view_account_data");
        Self(hex::encode(temp))
    }
}

impl fmt::Display for ViewOnlyAccountID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait ViewOnlyAccountModel {
    // insert new view-only-account in the db
    fn create(
        account_id_hex: &str,
        view_private_key: Vec<u8>,
        first_block_index: i64,
        import_block_index: i64,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError>;

    /// Get a specific account.
    /// Returns:
    /// * ViewOnlyAccount
    fn get(
        account_id: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError>;

    // TODO(cc) should this be paginated instead?
    /// List all view-only-accounts.
    /// Returns:
    /// * Vector of all View Only Accounts in the DB
    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    /// Update an view-only-account name.
    /// The only updatable field is the name. Any other desired update requires
    /// adding a new account, and deleting the existing if desired.
    fn update_name(
        &self,
        new_name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Update the next block index this account will need to sync.
    fn update_next_block_index(
        &self,
        next_block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Delete a view-only-account.
    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    // /// Get the accounts associated with the given Txo.
    // fn get_by_txo_id(
    //     txo_id_hex: &str,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    // /// Update the next block index this account will need to sync.
    // fn update_next_block_index(
    //     &self,
    //     next_block_index: i64,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;
}

impl ViewOnlyAccountModel for ViewOnlyAccount {
    fn create(
        account_id_hex: &str,
        view_private_key: Vec<u8>,
        first_block_index: i64,
        import_block_index: i64,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError> {
        use crate::db::schema::view_only_accounts;

        let new_view_only_account = NewViewOnlyAccount {
            account_id_hex: &account_id_hex,
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

        ViewOnlyAccount::get(&account_id_hex, conn.clone())
    }

    fn get(
        account_id: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyAccount, WalletDbError> {
        use crate::db::schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, view_only_accounts,
        };

        match view_only_accounts
            .filter((dsl_account_id).eq(account_id))
            .get_result::<ViewOnlyAccount>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::AccountNotFound(
                // TODO(cc) improve this error handling
                // str::from_utf8(view_private_key)?,
                "account not found".to_string(),
            )),
            Err(e) => Err(e.into()),
        }
    }

    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ViewOnlyAccount>, WalletDbError> {
        use crate::db::schema::view_only_accounts;

        Ok(view_only_accounts::table
            .select(view_only_accounts::all_columns)
            .load::<ViewOnlyAccount>(conn)?)
    }

    fn update_name(
        &self,
        new_name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, name as dsl_name, view_only_accounts,
        };

        diesel::update(view_only_accounts.filter(dsl_account_id.eq(&self.account_id_hex)))
            .set(dsl_name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }

    fn update_next_block_index(
        &self,
        next_block_index: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, next_block_index as dsl_next_block,
            view_only_accounts,
        };
        diesel::update(view_only_accounts.filter(dsl_account_id.eq(&self.account_id_hex)))
            .set(dsl_next_block.eq(next_block_index))
            .execute(conn)?;
        Ok(())
    }

    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, view_only_accounts,
        };

        diesel::delete(view_only_accounts.filter(dsl_account_id.eq(&self.account_id_hex)))
            .execute(conn)?;

        Ok(())
    }

    // /// Get the accounts associated with the given Txo.
    // fn get_by_txo_id(
    //     txo_id_hex: &str,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    // /// Update the next block index this account will need to sync.
    // fn update_next_block_index(
    //     &self,
    //     next_block_index: i64,
    //     conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    // ) -> Result<(), WalletDbError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;

    use mc_common::logger::{test_with_logger, Logger};

    #[test_with_logger]
    fn test_view_only_account_crud(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

        // test account creation

        let name = "Coins for cats";
        let view_private_key: Vec<u8> = [1, 2, 3].to_vec();
        let first_block_index: i64 = 25;
        let import_block_index: i64 = 26;
        let account_id_hex = "abcd";

        let expected_account = ViewOnlyAccount {
            id: 1,
            account_id_hex: account_id_hex.to_string(),
            view_private_key: view_private_key.clone(),
            first_block_index,
            next_block_index: first_block_index,
            import_block_index,
            name: name.to_string(),
        };

        let created = ViewOnlyAccount::create(
            account_id_hex,
            view_private_key.clone(),
            first_block_index,
            import_block_index,
            &name,
            &conn,
        )
        .unwrap();
        assert_eq!(expected_account, created);

        // test account name update

        let new_name = "coins for dogs";

        created.update_name(&new_name, &conn).unwrap();

        // test updating next block index

        let new_next_block = 100;

        created
            .update_next_block_index(new_next_block, &conn)
            .unwrap();

        // test getting an account

        let updated: ViewOnlyAccount = ViewOnlyAccount::get(&account_id_hex, &conn).unwrap();

        assert_eq!(&updated.name, &new_name);
        assert_eq!(updated.next_block_index, new_next_block);

        // test getting all accounts

        ViewOnlyAccount::create(
            "some_account_id",
            view_private_key.clone(),
            first_block_index,
            import_block_index,
            "catcoin_name",
            &conn,
        )
        .unwrap();

        let all_accounts = ViewOnlyAccount::list_all(&conn).unwrap();

        assert_eq!(all_accounts.len(), 2);

        // test deleting the account

        created.delete(&conn).unwrap();

        let not_found = ViewOnlyAccount::get(&account_id_hex, &conn);
        assert!(not_found.is_err());
    }
}
