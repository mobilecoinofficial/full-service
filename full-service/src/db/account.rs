// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Account model.

use crate::db::{
    assigned_subaddress::AssignedSubaddressModel,
    models::{Account, AssignedSubaddress, NewAccount, TransactionLog, Txo},
    transaction_log::TransactionLogModel,
    txo::TxoModel,
    WalletDbError,
};

use mc_account_keys::{AccountKey, RootEntropy, RootIdentity, DEFAULT_SUBADDRESS_INDEX};
use mc_account_keys_slip10::Slip10Key;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::ring_signature::KeyImage;

use bip39::Mnemonic;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use std::fmt;

pub const DEFAULT_CHANGE_SUBADDRESS_INDEX: u64 = 1;
pub const DEFAULT_NEXT_SUBADDRESS_INDEX: u64 = 2;
pub const DEFAULT_FIRST_BLOCK_INDEX: u64 = 0;

pub const ROOT_ENTROPY_KEY_DERIVATION_VERSION: u8 = 1;
pub const MNEMONIC_KEY_DERIVATION_VERSION: u8 = 2;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
    #[allow(clippy::too_many_arguments)]
    fn create_from_mnemonic(
        mnemonic: &Mnemonic,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Create an account.
    ///
    /// Returns:
    /// * (account_id, main_subaddress_b58)
    #[allow(clippy::too_many_arguments)]
    fn create_from_root_entropy(
        entropy: &RootEntropy,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Create an account.
    ///
    /// Returns:
    /// * (account_id, main_subaddress_b58)
    #[allow(clippy::too_many_arguments)]
    fn create(
        entropy: &[u8],
        key_derivation_version: u8,
        account_key: &AccountKey,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Import account.
    #[allow(clippy::too_many_arguments)]
    fn import(
        mnemonic: &Mnemonic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError>;

    /// Import account.
    #[allow(clippy::too_many_arguments)]
    fn import_legacy(
        entropy: &RootEntropy,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError>;

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
        account_id: &AccountID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError>;

    /// Get the accounts associated with the given Txo.
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
        spent_block_index: i64,
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
    fn create_from_mnemonic(
        mnemonic: &Mnemonic,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError> {
        let account_key = Slip10Key::from(mnemonic.clone())
            .try_into_account_key(
                &fog_report_url.unwrap_or_else(|| "".to_string()),
                &fog_report_id.unwrap_or_else(|| "".to_string()),
                &hex::decode(fog_authority_spki.unwrap_or_else(|| "".to_string()))
                    .expect("invalid spki"),
            )
            .unwrap();

        Account::create(
            mnemonic.entropy(),
            MNEMONIC_KEY_DERIVATION_VERSION,
            &account_key,
            first_block_index,
            import_block_index,
            next_subaddress_index,
            name,
            conn,
        )
    }

    fn create_from_root_entropy(
        entropy: &RootEntropy,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError> {
        let root_id = RootIdentity {
            root_entropy: entropy.clone(),
            fog_report_url: fog_report_url.unwrap_or_else(|| "".to_string()),
            fog_report_id: fog_report_id.unwrap_or_else(|| "".to_string()),
            fog_authority_spki: hex::decode(fog_authority_spki.unwrap_or_else(|| "".to_string()))
                .expect("invalid spki"),
        };
        let account_key = AccountKey::from(&root_id);

        Account::create(
            &entropy.bytes,
            ROOT_ENTROPY_KEY_DERIVATION_VERSION,
            &account_key,
            first_block_index,
            import_block_index,
            next_subaddress_index,
            name,
            conn,
        )
    }

    fn create(
        entropy: &[u8],
        key_derivation_version: u8,
        account_key: &AccountKey,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(AccountID, String), WalletDbError> {
        use crate::db::schema::accounts;

        let account_id = AccountID::from(account_key);
        let fb = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX);

        let new_account = NewAccount {
            account_id_hex: &account_id.to_string(),
            account_key: &mc_util_serial::encode(account_key), /* FIXME: WS-6 - add
                                                                * encryption */
            entropy,
            key_derivation_version: key_derivation_version as i32,
            main_subaddress_index: DEFAULT_SUBADDRESS_INDEX as i64,
            change_subaddress_index: DEFAULT_CHANGE_SUBADDRESS_INDEX as i64,
            next_subaddress_index: next_subaddress_index.unwrap_or(DEFAULT_NEXT_SUBADDRESS_INDEX)
                as i64,
            first_block_index: fb as i64,
            next_block_index: fb as i64,
            import_block_index: import_block_index.map(|i| i as i64),
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
            conn,
        )?;

        let _change_subaddress_b58 = AssignedSubaddress::create(
            account_key,
            None, /* FIXME: WS-8 - Address Book Entry if details provided, or None
                   * always for main? */
            DEFAULT_CHANGE_SUBADDRESS_INDEX,
            "Change",
            conn,
        )?;

        for subaddress_index in 2..next_subaddress_index.unwrap_or(DEFAULT_NEXT_SUBADDRESS_INDEX) {
            AssignedSubaddress::create(account_key, None, subaddress_index, "", conn)?;
        }

        Ok((account_id, main_subaddress_b58))
    }

    fn import(
        mnemonic: &Mnemonic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError> {
        let (account_id, _public_address_b58) = Account::create_from_mnemonic(
            mnemonic,
            first_block_index,
            Some(import_block_index),
            next_subaddress_index,
            &name.unwrap_or_else(|| "".to_string()),
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
            conn,
        )?;
        Account::get(&account_id, conn)
    }

    fn import_legacy(
        root_entropy: &RootEntropy,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: Option<String>,
        fog_report_id: Option<String>,
        fog_authority_spki: Option<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError> {
        let (account_id, _public_address_b58) = Account::create_from_root_entropy(
            root_entropy,
            first_block_index,
            Some(import_block_index),
            next_subaddress_index,
            &name.unwrap_or_else(|| "".to_string()),
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
            conn,
        )?;
        Account::get(&account_id, conn)
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
        account_id: &AccountID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts::dsl::{account_id_hex as dsl_account_id_hex, accounts};

        match accounts
            .filter(dsl_account_id_hex.eq(account_id.to_string()))
            .get_result::<Account>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::AccountNotFound(account_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_by_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Account>, WalletDbError> {
        let txo = Txo::get(txo_id_hex, conn)?;

        let mut accounts: Vec<Account> = Vec::<Account>::new();

        if let Some(received_account_id_hex) = txo.received_account_id_hex {
            let account = Account::get(&AccountID(received_account_id_hex), conn)?;
            accounts.push(account);
        }

        if let Some(minted_account_id_hex) = txo.minted_account_id_hex {
            let account = Account::get(&AccountID(minted_account_id_hex), conn)?;
            accounts.push(account);
        }

        Ok(accounts)
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
        spent_block_index: i64,
        key_images: Vec<KeyImage>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::{
            accounts::dsl::{account_id_hex, accounts},
            txos::dsl::{txo_id_hex, txos},
        };

        for key_image in key_images {
            // Get the txo by key_image
            let matches = crate::db::schema::txos::table
                .select(crate::db::schema::txos::all_columns)
                .filter(crate::db::schema::txos::key_image.eq(mc_util_serial::encode(&key_image)))
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
                    .set((
                        crate::db::schema::txos::spent_block_index.eq(Some(spent_block_index)),
                        crate::db::schema::txos::pending_tombstone_block_index
                            .eq::<Option<i64>>(None),
                    ))
                    .execute(conn)?;

                // FIXME: WS-13 - make sure the path for all txo_statuses and txo_types exist
                // and are tested Update the transaction status if the txos
                // are all spent
                TransactionLog::update_transactions_associated_to_txo(
                    &matches[0].txo_id_hex,
                    spent_block_index,
                    conn,
                )?;
            }
        }
        diesel::update(accounts.filter(account_id_hex.eq(&self.account_id_hex)))
            .set(crate::db::schema::accounts::next_block_index.eq(spent_block_index + 1))
            .execute(conn)?;
        Ok(())
    }

    /// Delete an account.
    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts::dsl::{account_id_hex, accounts};

        diesel::delete(accounts.filter(account_id_hex.eq(&self.account_id_hex))).execute(conn)?;

        // Also delete transaction logs associated with this account
        TransactionLog::delete_all_for_account(&self.account_id_hex, conn)?;

        // Also delete the associated assigned subaddresses
        AssignedSubaddress::delete_all(&self.account_id_hex, conn)?;

        Txo::scrub_account(&self.account_id_hex, conn)?;

        // Delete Txos with no references.
        Txo::delete_unreferenced(conn)?;

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
    use std::{collections::HashSet, convert::TryFrom, iter::FromIterator};

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let account_id_hex = {
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's Main Account",
                None,
                None,
                None,
                &conn,
            )
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
            entropy: root_id.root_entropy.bytes.to_vec(),
            key_derivation_version: 1,
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block_index: 0,
            next_block_index: 0,
            import_block_index: None,
            name: "Alice's Main Account".to_string(),
        };
        assert_eq!(expected_account, acc);

        // Verify that the subaddress table entries were updated for main and change
        let subaddresses = AssignedSubaddress::list_all(
            &account_id_hex.to_string(),
            None,
            None,
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
        let root_id_secondary = RootIdentity::from_random(&mut rng);
        let account_key_secondary = AccountKey::from(&root_id_secondary);
        let (account_id_hex_secondary, _public_address_b58_secondary) =
            Account::create_from_root_entropy(
                &root_id_secondary.root_entropy,
                Some(51),
                Some(50),
                None,
                "",
                None,
                None,
                None,
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
            entropy: root_id_secondary.root_entropy.bytes.to_vec(),
            key_derivation_version: 1,
            main_subaddress_index: 0,
            change_subaddress_index: 1,
            next_subaddress_index: 2,
            first_block_index: 51,
            next_block_index: 51,
            import_block_index: Some(50),
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

    // Providing entropy should succeed and derive account key.
    #[test_with_logger]
    fn test_create_account_from_entropy(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        // Test providing entropy.
        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let account_id = {
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's Main Account",
                None,
                None,
                None,
                &conn,
            )
            .unwrap();
            account_id_hex
        };
        let account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();
        let decoded_entropy = RootEntropy::try_from(account.entropy.as_slice()).unwrap();
        assert_eq!(decoded_entropy, root_id.root_entropy);
        let decoded_account_key: AccountKey = mc_util_serial::decode(&account.account_key).unwrap();
        assert_eq!(decoded_account_key, account_key);
    }
}
