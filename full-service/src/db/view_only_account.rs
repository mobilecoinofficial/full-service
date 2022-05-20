// Copyright (c) 2020-2022 MobileCoin Inc.

//! DB impl for the View Only Account model.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{Account, NewViewOnlyAccount, ViewOnlyAccount, ViewOnlySubaddress, ViewOnlyTxo},
        schema,
        view_only_subaddress::ViewOnlySubaddressModel,
        view_only_txo::ViewOnlyTxoModel,
        Conn, WalletDbError,
    },
    util::{b58::b58_decode_public_address, encoding_helpers::ristretto_to_vec},
};
use diesel::prelude::*;
use mc_account_keys::PublicAddress;
use mc_crypto_keys::RistrettoPrivate;
use std::str;

pub trait ViewOnlyAccountModel {
    // insert new view-only-account in the db\
    #[allow(clippy::too_many_arguments)]
    fn create(
        account_id_hex: &str,
        view_private_key: &RistrettoPrivate,
        first_block_index: u64,
        import_block_index: u64,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        name: &str,
        conn: &Conn,
    ) -> Result<ViewOnlyAccount, WalletDbError>;

    /// Get a specific account.
    /// Returns:
    /// * ViewOnlyAccount
    fn get(account_id: &str, conn: &Conn) -> Result<ViewOnlyAccount, WalletDbError>;

    /// List all view-only-accounts.
    /// Returns:
    /// * Vector of all View Only Accounts in the DB
    fn list_all(conn: &Conn) -> Result<Vec<ViewOnlyAccount>, WalletDbError>;

    /// Update an view-only-account name.
    /// The only updatable field is the name. Any other desired update requires
    /// adding a new account, and deleting the existing if desired.
    fn update_name(&self, new_name: &str, conn: &Conn) -> Result<(), WalletDbError>;

    /// Update the next block index this account will need to sync.
    fn update_next_block_index(
        &self,
        next_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn update_next_subaddress_index(
        &self,
        next_subaddress_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    fn change_public_address(&self, conn: &Conn) -> Result<PublicAddress, WalletDbError>;

    /// Delete a view-only-account.
    fn delete(self, conn: &Conn) -> Result<(), WalletDbError>;
}

impl ViewOnlyAccountModel for ViewOnlyAccount {
    fn create(
        account_id_hex: &str,
        view_private_key: &RistrettoPrivate,
        first_block_index: u64,
        import_block_index: u64,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        name: &str,
        conn: &Conn,
    ) -> Result<ViewOnlyAccount, WalletDbError> {
        use schema::view_only_accounts;

        if Account::get(&AccountID(account_id_hex.to_string()), conn).is_ok() {
            return Err(WalletDbError::ViewOnlyAccountAlreadyExists(
                account_id_hex.to_string(),
            ));
        }

        let encoded_key = ristretto_to_vec(view_private_key);

        let new_view_only_account = NewViewOnlyAccount {
            account_id_hex,
            view_private_key: &encoded_key,
            first_block_index: first_block_index as i64,
            next_block_index: first_block_index as i64,
            import_block_index: import_block_index as i64,
            name,
            next_subaddress_index: next_subaddress_index as i64,
            main_subaddress_index: main_subaddress_index as i64,
            change_subaddress_index: change_subaddress_index as i64,
        };

        diesel::insert_into(view_only_accounts::table)
            .values(&new_view_only_account)
            .execute(conn)?;

        ViewOnlyAccount::get(account_id_hex, conn)
    }

    fn get(account_id: &str, conn: &Conn) -> Result<ViewOnlyAccount, WalletDbError> {
        use schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, view_only_accounts,
        };

        match view_only_accounts
            .filter((dsl_account_id).eq(account_id.to_string()))
            .get_result::<ViewOnlyAccount>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::AccountNotFound(account_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn list_all(conn: &Conn) -> Result<Vec<ViewOnlyAccount>, WalletDbError> {
        use schema::view_only_accounts;

        Ok(view_only_accounts::table
            .select(view_only_accounts::all_columns)
            .load::<ViewOnlyAccount>(conn)?)
    }

    fn update_name(&self, new_name: &str, conn: &Conn) -> Result<(), WalletDbError> {
        use schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, name as dsl_name, view_only_accounts,
        };

        diesel::update(view_only_accounts.filter(dsl_account_id.eq(&self.account_id_hex)))
            .set(dsl_name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }

    fn update_next_block_index(
        &self,
        next_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, next_block_index as dsl_next_block,
            view_only_accounts,
        };
        diesel::update(view_only_accounts.filter(dsl_account_id.eq(&self.account_id_hex)))
            .set(dsl_next_block.eq(next_block_index as i64))
            .execute(conn)?;
        Ok(())
    }

    fn update_next_subaddress_index(
        &self,
        next_subaddress_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::view_only_accounts;

        diesel::update(
            view_only_accounts::table
                .filter(view_only_accounts::account_id_hex.eq(&self.account_id_hex)),
        )
        .set(view_only_accounts::next_subaddress_index.eq(next_subaddress_index as i64))
        .execute(conn)?;

        Ok(())
    }

    fn change_public_address(&self, conn: &Conn) -> Result<PublicAddress, WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        let change_subaddress = view_only_subaddresses::table
            .filter(view_only_subaddresses::view_only_account_id_hex.eq(&self.account_id_hex))
            .filter(view_only_subaddresses::subaddress_index.eq(self.change_subaddress_index))
            .first::<ViewOnlySubaddress>(conn)?;

        let change_public_address =
            b58_decode_public_address(&change_subaddress.public_address_b58)?;

        Ok(change_public_address)
    }

    fn delete(self, conn: &Conn) -> Result<(), WalletDbError> {
        use schema::view_only_accounts::dsl::{
            account_id_hex as dsl_account_id, view_only_accounts,
        };

        // delete associated view-only-txos
        ViewOnlyTxo::delete_all_for_account(&self.account_id_hex, conn)?;
        ViewOnlySubaddress::delete_all_for_account(&self.account_id_hex, conn)?;
        diesel::delete(view_only_accounts.filter(dsl_account_id.eq(&self.account_id_hex)))
            .execute(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::{CHANGE_SUBADDRESS_INDEX, DEFAULT_SUBADDRESS_INDEX};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPrivate;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_view_only_account_crud(logger: Logger) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        // test account creation

        let name = "Coins for cats";
        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let first_block_index: u64 = 25;
        let import_block_index: u64 = 26;
        let account_id_hex = "abcd";

        let expected_account = ViewOnlyAccount {
            id: 1,
            account_id_hex: account_id_hex.to_string(),
            view_private_key: ristretto_to_vec(&view_private_key),
            first_block_index: first_block_index as i64,
            next_block_index: first_block_index as i64,
            import_block_index: import_block_index as i64,
            name: name.to_string(),
            main_subaddress_index: DEFAULT_SUBADDRESS_INDEX as i64,
            change_subaddress_index: CHANGE_SUBADDRESS_INDEX as i64,
            next_subaddress_index: 2,
        };

        let created = ViewOnlyAccount::create(
            account_id_hex,
            &view_private_key,
            first_block_index,
            import_block_index,
            DEFAULT_SUBADDRESS_INDEX,
            CHANGE_SUBADDRESS_INDEX,
            2,
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
        assert_eq!(updated.next_block_index as u64, new_next_block);

        // test getting all accounts

        ViewOnlyAccount::create(
            "some_account_id",
            &view_private_key,
            first_block_index,
            import_block_index,
            DEFAULT_SUBADDRESS_INDEX,
            CHANGE_SUBADDRESS_INDEX,
            2,
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
