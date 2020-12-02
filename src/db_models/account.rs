// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Account model

use crate::db_models::assigned_subaddress::AssignedSubaddressModel;
use crate::error::WalletDbError;
use crate::models::{Account, AssignedSubaddress, NewAccount};

use mc_account_keys::AccountKey;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use std::iter::FromIterator;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::RunQueryDsl;

// Schema Tables
use crate::schema::accounts as schema_accounts;

use crate::db::AccountID;

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

        // FIXME: It's concerning to lose a bit of precision in casting to i64
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
            &account_id.to_string(),
            None, // FIXME: Address Book Entry if details provided, or None always for main?
            main_subaddress_index,
            "Main",
            &conn,
        )?;

        let _change_subaddress_b58 = AssignedSubaddress::create(
            account_key,
            &account_id.to_string(),
            None, // FIXME: Address Book Entry if details provided, or None always for main?
            change_subaddress_index,
            "Change",
            &conn,
        )?;

        Ok((account_id.to_string(), main_subaddress_b58))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_transaction_core::encrypted_fog_hint::EncryptedFogHint;
    use mc_transaction_core::onetime_keys::recover_public_subaddress_spend_key;
    use mc_transaction_core::ring_signature::KeyImage;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashSet;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);
        let (account_id_hex, _public_address_b58) = Account::create(
            &account_key,
            0,
            1,
            2,
            0,
            1,
            "Alice's Main Account",
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let res = wallet_db.list_accounts().unwrap();
        assert_eq!(res.len(), 1);

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
