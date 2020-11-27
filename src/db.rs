// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::{AccountKey, PublicAddress};
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::tx::TxOut;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;

use crate::error::WalletDbError;
use crate::models::{
    Account, AssignedSubaddress, NewAccount, NewAccountTxoStatus, NewAssignedSubaddress, NewTxo,
    Txo,
};
use crate::schema::account_txo_statuses as schema_account_txo_statuses;
use crate::schema::accounts as schema_accounts;
use crate::schema::accounts::dsl::accounts as dsl_accounts;
use crate::schema::assigned_subaddresses as schema_assigned_subaddresses;
use crate::schema::txos as schema_txos;

// Helper method to use our PrintableWrapper to b58 encode the PublicAddress
pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

/// The account ID is derived from the contents of the account key
#[derive(Digestible)]
struct ConstAccountData {
    /// The public address of the main subaddress for this account
    pub address: PublicAddress,
}

/// The txo ID is derived from the contents of the txo
#[derive(Digestible)]
struct ConstTxoData {
    /// The public address of the main subaddress for this account
    pub txo: TxOut,
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
        name: &str,
    ) -> Result<(String, String), WalletDbError> {
        let conn = self.pool.get()?;

        let main_subaddress = account_key.subaddress(main_subaddress_index);
        let const_data = ConstAccountData {
            address: main_subaddress.clone(),
        };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"account_data");
        let account_id_hex = hex::encode(temp);

        // FIXME: It's concerning to lose a bit of precision in casting to i64
        let new_account = NewAccount {
            account_id_hex: &account_id_hex,
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
            .execute(&conn)?;

        // Insert the assigned subaddresses for main and change
        let main_subaddress_b58 = b58_encode(&main_subaddress)?;
        let main_subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &main_subaddress_b58,
            account_id_hex: &account_id_hex,
            address_book_entry: None, // FIXME: Address Book Entry if details provided, or None always for main?
            public_address: &mc_util_serial::encode(&main_subaddress),
            subaddress_index: main_subaddress_index as i64,
            comment: "Main",
            expected_value: None,
            subaddress_spend_key: &mc_util_serial::encode(main_subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&main_subaddress_entry)
            .execute(&conn)?;

        let change_subaddress = account_key.subaddress(change_subaddress_index);
        let change_subaddress_b58 = b58_encode(&change_subaddress)?;
        let change_subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &change_subaddress_b58,
            account_id_hex: &account_id_hex,
            address_book_entry: None, // FIXME: Address Book Entry if details provided, or None always for main?
            public_address: &mc_util_serial::encode(&change_subaddress),
            subaddress_index: change_subaddress_index as i64,
            comment: "Change",
            expected_value: None,
            subaddress_spend_key: &mc_util_serial::encode(change_subaddress.spend_public_key()),
        };

        diesel::insert_into(schema_assigned_subaddresses::table)
            .values(&change_subaddress_entry)
            .execute(&conn)?;

        Ok((account_id_hex, main_subaddress_b58))
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

        match dsl_accounts
            .find(account_id_hex)
            .get_result::<Account>(&conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::NotFound(account_id_hex.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Update an account.
    /// The only updatable field is the name. Any other desired update requires adding
    /// a new account, and deleting the existing if desired.
    pub fn update_account_name(
        &self,
        account_id_hex: &str,
        new_name: String,
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

    /// Create a TXO entry
    pub fn create_received_txo(
        &self,
        txo: TxOut,
        subaddress_index: u64,
        key_image: Vec<u8>,
        value: u64,
        received_block_height: u64,
        account_id_hex: &str,
    ) -> Result<String, WalletDbError> {
        let conn = self.pool.get()?;

        let const_data = ConstTxoData { txo: txo.clone() };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"txo_data");
        let txo_id_hex = hex::encode(temp);

        let new_txo = NewTxo {
            txo_id_hex: &txo_id_hex,
            value: value as i64,
            target_key: &txo.target_key.as_bytes().to_vec(),
            public_key: &txo.public_key.as_bytes().to_vec(),
            e_fog_hint: &txo.e_fog_hint.to_bytes().to_vec(),
            subaddress_index: subaddress_index as i64,
            key_image: Some(&key_image),
            received_block_height: Some(received_block_height as i64),
            spent_tombstone_block_height: None,
            spent_block_height: None,
            proof: None,
        };

        diesel::insert_into(schema_txos::table)
            .values(&new_txo)
            .execute(&conn)?;

        let new_account_txo_status = NewAccountTxoStatus {
            account_id_hex: &account_id_hex,
            txo_id_hex: &txo_id_hex,
            txo_status: "unspent",
            txo_type: "received",
        };

        diesel::insert_into(schema_account_txo_statuses::table)
            .values(&new_account_txo_status)
            .execute(&conn)?;

        Ok(txo_id_hex)
    }

    /// List all txos for a given account.
    pub fn list_txos(&self, account_id_hex: &str) -> Result<Vec<Txo>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<Txo> = schema_txos::table
            .inner_join(
                schema_account_txo_statuses::table.on(schema_txos::txo_id_hex
                    .eq(schema_account_txo_statuses::txo_id_hex)
                    .and(schema_account_txo_statuses::account_id_hex.eq(account_id_hex))),
            )
            .select(schema_txos::all_columns)
            .load(&conn)?;

        Ok(results)
    }

    /// List all subaddresses for a given account.
    pub fn list_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<AssignedSubaddress>, WalletDbError> {
        let conn = self.pool.get()?;

        let results: Vec<AssignedSubaddress> = schema_accounts::table
            .inner_join(
                schema_assigned_subaddresses::table.on(schema_accounts::account_id_hex
                    .eq(schema_assigned_subaddresses::account_id_hex)
                    .and(schema_accounts::account_id_hex.eq(account_id_hex))),
            )
            .select(schema_assigned_subaddresses::all_columns)
            .load(&conn)?;

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_transaction_core::encrypted_fog_hint::EncryptedFogHint;
    use mc_transaction_core::onetime_keys::recover_public_subaddress_spend_key;
    use mc_transaction_core::ring_signature::KeyImage;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashSet;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    #[test]
    fn test_account_crud() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance();

        let account_key = AccountKey::random(&mut rng);
        let (account_id_hex, _public_address_b58) = walletdb
            .create_account(&account_key, 0, 1, 2, 0, 1, "Alice's Main Account")
            .unwrap();

        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

        let acc = walletdb.get_account(&account_id_hex).unwrap();
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

        // FIXME Verify that the subaddress table entries were updated for main and change
        let subaddresses = walletdb.list_subaddresses(&account_id_hex).unwrap();
        assert_eq!(subaddresses.len(), 2);
        let subaddress_indices: HashSet<i64> =
            HashSet::from_iter(subaddresses.iter().map(|s| s.subaddress_index));
        assert!(subaddress_indices.get(&0).is_some());
        assert!(subaddress_indices.get(&1).is_some());

        // Add another account with no name, scanning from later
        let account_key_secondary = AccountKey::from(&RootIdentity::from_random(&mut rng));
        let (account_id_hex_secondary, _public_address_b58_secondary) = walletdb
            .create_account(&account_key_secondary, 0, 1, 2, 50, 51, "")
            .unwrap();
        let res = walletdb.list_accounts().unwrap();
        assert_eq!(res.len(), 2);

        let acc_secondary = walletdb.get_account(&account_id_hex_secondary).unwrap();
        let mut expected_account_secondary = Account {
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
        walletdb
            .update_account_name(
                &account_id_hex_secondary,
                "Alice's Secondary Account".to_string(),
            )
            .unwrap();
        let acc_secondary2 = walletdb.get_account(&account_id_hex_secondary).unwrap();
        expected_account_secondary.name = "Alice's Secondary Account".to_string();
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
            Err(_) => panic!("Should error with NotFound but got {:?}", res),
        }
    }

    #[test]
    fn test_received_tx_crud() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance();

        let account_key = AccountKey::random(&mut rng);
        let (account_id_hex, _public_address_b58) = walletdb
            .create_account(&account_key, 0, 1, 2, 0, 1, "Alice's Main Account")
            .unwrap();

        // FIXME: get recipient via the assigned subaddresses table, not directly
        let recipient = account_key.subaddress(0);

        // Create TXO for the account
        let tx_private_key = RistrettoPrivate::from_random(&mut rng);
        let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
        let value = 10;
        let txo = TxOut::new(value, &recipient, &tx_private_key, hint).unwrap();

        // Get KeyImage from the onetime private key
        let key_image = KeyImage::from(&tx_private_key);

        // Sanity check: Ensure that we can recover the subaddress
        // FIXME: Assert that the public address and the subaddress spend key was added to the
        //        assigned_subaddresses table
        let _subaddress_index = recover_public_subaddress_spend_key(
            account_key.view_private_key(),
            &RistrettoPublic::try_from(&txo.target_key).unwrap(),
            &RistrettoPublic::try_from(&txo.public_key).unwrap(),
        );
        let subaddress_index = 0;

        let received_block_height = 144;

        let txo_hex = walletdb
            .create_received_txo(
                txo.clone(),
                subaddress_index,
                key_image.to_vec(),
                value,
                received_block_height,
                &account_id_hex,
            )
            .unwrap();

        let txos = walletdb.list_txos(&account_id_hex).unwrap();
        assert_eq!(txos.len(), 1);

        let expected_txo = Txo {
            txo_id_hex: txo_hex,
            value: value as i64,
            target_key: txo.target_key.as_bytes().to_vec(),
            public_key: txo.public_key.as_bytes().to_vec(),
            e_fog_hint: txo.e_fog_hint.to_bytes().to_vec(),
            subaddress_index: subaddress_index as i64,
            key_image: Some(key_image.as_bytes().to_vec()),
            received_block_height: Some(received_block_height as i64),
            spent_tombstone_block_height: None,
            spent_block_height: None,
            proof: None,
        };
        assert_eq!(txos[0], expected_txo);
    }
}
