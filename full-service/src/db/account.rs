// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Account model.

use crate::{
    db::{
        assigned_subaddress::AssignedSubaddressModel,
        b58_encode,
        models::{
            Account, AccountTxoStatus, AssignedSubaddress, NewAccount, TransactionLog, Txo,
            TXO_STATUS_PENDING, TXO_STATUS_SPENT, TXO_STATUS_UNSPENT,
        },
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDbError,
    },
    json_rpc::api_v1::decorated_types::JsonAccount,
};

use mc_account_keys::{AccountKey, DEFAULT_SUBADDRESS_INDEX};
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::ring_signature::KeyImage;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use std::fmt;

pub const DEFAULT_CHANGE_SUBADDRESS_INDEX: u64 = 1;
pub const DEFAULT_NEXT_SUBADDRESS_INDEX: u64 = 2;
pub const DEFAULT_FIRST_BLOCK: u64 = 0;

#[derive(Debug, Clone)]
pub struct AccountID(pub String);

impl From<&AccountKey> for AccountID {
    fn from(src: &AccountKey) -> AccountID {
        let main_subaddress = src.subaddress(DEFAULT_SUBADDRESS_INDEX);
        let temp: [u8; 32] = main_subaddress.digest32::<MerlinTranscript>(b"account_data");
        Self(hex::encode(temp))
    }
}

impl fmt::Display for AccountID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait AccountModel {
    /// Create an account.
    ///
    /// Returns:
    /// * (account_id, main_subaddress_b58)
    fn create(
        account_key: &AccountKey,
        first_block: Option<u64>,
        import_block: Option<u64>,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Import account.
    fn import(
        entropy: &AccountKey,
        name: Option<String>,
        first_block: Option<u64>,
        local_height: u64,
        network_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<JsonAccount, WalletDbError>;

    /// List all accounts.
    ///
    /// Returns:
    /// * Vector of all Accounts in the DB
    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError>;

    /// Get a specific account.
    ///
    /// Returns:
    /// * Account
    fn get(
        account_id_hex: &AccountID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError>;

    /// Get the API-decorated Account object
    fn get_decorated(
        account_id_hex: &AccountID,
        local_height: u64,
        network_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<JsonAccount, WalletDbError>;

    fn get_by_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError>;

    /// Update an account.
    /// The only updatable field is the name. Any other desired update requires
    /// adding a new account, and deleting the existing if desired.
    fn update_name(
        &self,
        new_name: String,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Update key-image-matching txos associated with this account to spent for
    /// a given block height.
    fn update_spent_and_increment_next_block(
        &self,
        spent_block_count: i64,
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
        first_block: Option<u64>,
        import_block: Option<u64>,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError> {
        use crate::db::schema::accounts;

        let account_id = AccountID::from(account_key);
        let fb = first_block.unwrap_or(DEFAULT_FIRST_BLOCK);

        Ok(
            conn.transaction::<(AccountID, String), WalletDbError, _>(|| {
                let new_account = NewAccount {
                    account_id_hex: &account_id.to_string(),
                    account_key: &mc_util_serial::encode(account_key), /* FIXME: WS-6 - add
                                                                        * encryption */
                    main_subaddress_index: DEFAULT_SUBADDRESS_INDEX as i64,
                    change_subaddress_index: DEFAULT_CHANGE_SUBADDRESS_INDEX as i64,
                    next_subaddress_index: DEFAULT_NEXT_SUBADDRESS_INDEX as i64,
                    first_block: fb as i64,
                    next_block: fb as i64,
                    import_block: import_block.map(|i| i as i64),
                    name,
                };

                diesel::insert_into(accounts::table)
                    .values(&new_account)
                    .execute(conn)?;

                let main_subaddress_b58 = AssignedSubaddress::create(
                    account_key,
                    None, /* FIXME: WS-8 - Address Book Entry if details provided, or None
                           * always for main? */
                    DEFAULT_SUBADDRESS_INDEX,
                    "Main",
                    &conn,
                )?;

                let _change_subaddress_b58 = AssignedSubaddress::create(
                    account_key,
                    None, /* FIXME: WS-8 - Address Book Entry if details provided, or None
                           * always for main? */
                    DEFAULT_CHANGE_SUBADDRESS_INDEX,
                    "Change",
                    &conn,
                )?;
                Ok((account_id, main_subaddress_b58))
            })?,
        )
    }

    fn import(
        account_key: &AccountKey,
        name: Option<String>,
        first_block: Option<u64>,
        local_height: u64,
        network_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<JsonAccount, WalletDbError> {
        Ok(conn.transaction::<JsonAccount, WalletDbError, _>(|| {
            let (account_id, _public_address_b58) = Account::create(
                &account_key,
                first_block,
                Some(local_height),
                &name.unwrap_or_else(|| "".to_string()),
                conn,
            )?;
            Ok(Account::get_decorated(
                &account_id,
                local_height,
                network_height,
                &conn,
            )?)
        })?)
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
        account_id_hex: &AccountID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts::dsl::{account_id_hex as dsl_account_id_hex, accounts};

        match accounts
            .filter(dsl_account_id_hex.eq(account_id_hex.to_string()))
            .get_result::<Account>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::AccountNotFound(account_id_hex.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_decorated(
        account_id_hex: &AccountID,
        local_height: u64,
        network_height: u64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<JsonAccount, WalletDbError> {
        Ok(conn.transaction::<JsonAccount, WalletDbError, _>(|| {
            let account = Account::get(account_id_hex, conn)?;

            let unspent =
                Txo::list_by_status(&account_id_hex.to_string(), TXO_STATUS_UNSPENT, conn)?
                    .iter()
                    .map(|t| t.value as u128)
                    .sum::<u128>();
            let pending =
                Txo::list_by_status(&account_id_hex.to_string(), TXO_STATUS_PENDING, conn)?
                    .iter()
                    .map(|t| t.value as u128)
                    .sum::<u128>();

            let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
            let main_subaddress_b58 =
                b58_encode(&account_key.subaddress(DEFAULT_SUBADDRESS_INDEX))?;
            Ok(JsonAccount {
                object: "account".to_string(),
                account_id: account.account_id_hex,
                name: account.name,
                network_height: network_height.to_string(),
                local_height: local_height.to_string(),
                account_height: account.next_block.to_string(),
                is_synced: account.next_block == network_height as i64,
                available_pmob: unspent.to_string(),
                pending_pmob: pending.to_string(),
                main_address: main_subaddress_b58,
                next_subaddress_index: account.next_subaddress_index.to_string(),
                recovery_mode: false, // FIXME: WS-24 - Recovery mode for account
            })
        })?)
    }

    fn get_by_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError> {
        use crate::db::schema::account_txo_statuses::dsl::account_txo_statuses;

        match account_txo_statuses
            .select(crate::db::schema::account_txo_statuses::all_columns)
            .filter(crate::db::schema::account_txo_statuses::txo_id_hex.eq(txo_id_hex))
            .load::<AccountTxoStatus>(conn)
        {
            Ok(accounts) => Ok(accounts
                .iter()
                .map(|a| Account::get(&AccountID(a.account_id_hex.to_string()), conn))
                .collect::<Result<Vec<Account>, WalletDbError>>()?),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::TxoNotFound(txo_id_hex.to_string()))
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
        spent_block_count: i64,
        key_images: Vec<KeyImage>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{
            account_txo_statuses::dsl::account_txo_statuses,
            accounts::dsl::{account_id_hex, accounts},
            txos::dsl::{txo_id_hex, txos},
        };

        Ok(conn.transaction::<(), WalletDbError, _>(|| {
            for key_image in key_images {
                // Get the txo by key_image
                let matches = crate::db::schema::txos::table
                    .select(crate::db::schema::txos::all_columns)
                    .filter(
                        crate::db::schema::txos::key_image.eq(mc_util_serial::encode(&key_image)),
                    )
                    .load::<Txo>(conn)?;

                if matches.is_empty() {
                    // Not Found is ok - this means it's a key_image not associated with any of our
                    // txos
                    continue;
                } else if matches.len() > 1 {
                    return Err(WalletDbError::DuplicateEntries(format!(
                        "Key Image: {:?}",
                        key_image
                    )));
                } else {
                    // Update the TXO
                    diesel::update(txos.filter(txo_id_hex.eq(&matches[0].txo_id_hex)))
                        .set(crate::db::schema::txos::spent_block_count.eq(Some(spent_block_count)))
                        .execute(conn)?;

                    // Update the AccountTxoStatus
                    diesel::update(
                        account_txo_statuses.find((&self.account_id_hex, &matches[0].txo_id_hex)),
                    )
                    .set(
                        crate::db::schema::account_txo_statuses::txo_status
                            .eq(TXO_STATUS_SPENT.to_string()),
                    )
                    .execute(conn)?;

                    // FIXME: WS-13 - make sure the path for all txo_statuses and txo_types exist
                    // and are tested Update the transaction status if the txos
                    // are all spent
                    TransactionLog::update_transactions_associated_to_txo(
                        &matches[0].txo_id_hex,
                        spent_block_count,
                        conn,
                    )?;
                }
            }
            diesel::update(accounts.filter(account_id_hex.eq(&self.account_id_hex)))
                .set(crate::db::schema::accounts::next_block.eq(spent_block_count + 1))
                .execute(conn)?;
            Ok(())
        })?)
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
    use std::{collections::HashSet, iter::FromIterator};

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);
        let account_id_hex = {
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) =
                Account::create(&account_key, Some(0), None, "Alice's Main Account", &conn)
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
            account_id_hex: account_id_hex.to_string(),
            account_key: mc_util_serial::encode(&account_key),
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block: 0,
            next_block: 0,
            import_block: None,
            name: "Alice's Main Account".to_string(),
        };
        assert_eq!(expected_account, acc);

        // Verify that the subaddress table entries were updated for main and change
        let subaddresses = AssignedSubaddress::list_all(
            &account_id_hex.to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
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
        assert_eq!(retrieved_acocunt_id_hex, account_id_hex.to_string());

        // Add another account with no name, scanning from later
        let account_key_secondary = AccountKey::from(&RootIdentity::from_random(&mut rng));
        let (account_id_hex_secondary, _public_address_b58_secondary) = Account::create(
            &account_key_secondary,
            Some(51),
            Some(50),
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
            account_id_hex: account_id_hex_secondary.to_string(),
            account_key: mc_util_serial::encode(&account_key_secondary),
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block: 51,
            next_block: 51,
            import_block: Some(50),
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
            Err(WalletDbError::AccountNotFound(s)) => {
                assert_eq!(s, account_id_hex_secondary.to_string())
            }
            Err(_) => panic!("Should error with NotFound but got {:?}", res),
        }
    }
}
