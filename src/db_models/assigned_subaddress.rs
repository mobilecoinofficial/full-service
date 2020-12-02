// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the AssignedSubaddress model

use crate::db::b58_encode;
use crate::db_models::account::AccountID;
use crate::error::WalletDbError;
use crate::models::{AssignedSubaddress, NewAssignedSubaddress};

use mc_account_keys::AccountKey;
use mc_crypto_keys::RistrettoPublic;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

pub trait AssignedSubaddressModel {
    fn create(
        public_address: &AccountKey,
        address_book_entry: Option<i64>,
        subaddress_index: u64,
        comment: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;

    fn get(
        public_addres_b58: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssignedSubaddress, WalletDbError>;

    fn find_by_subaddress_spend_public_key(
        subaddress_spend_public_key: &RistrettoPublic,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(i64, String), WalletDbError>;
}

impl AssignedSubaddressModel for AssignedSubaddress {
    fn create(
        account_key: &AccountKey,
        address_book_entry: Option<i64>,
        subaddress_index: u64,
        comment: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        use crate::schema::assigned_subaddresses;

        let account_id = AccountID::from(account_key);

        let subaddress = account_key.subaddress(subaddress_index);
        let subaddress_b58 = b58_encode(&subaddress)?;
        let subaddress_entry = NewAssignedSubaddress {
            assigned_subaddress_b58: &subaddress_b58,
            account_id_hex: &account_id.to_string(),
            address_book_entry,
            public_address: &mc_util_serial::encode(&subaddress),
            subaddress_index: subaddress_index as i64,
            comment,
            subaddress_spend_key: &mc_util_serial::encode(subaddress.spend_public_key()),
        };

        diesel::insert_into(assigned_subaddresses::table)
            .values(&subaddress_entry)
            .execute(conn)?;

        Ok(subaddress_b58)
    }

    fn get(
        public_address_b58: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<AssignedSubaddress, WalletDbError> {
        use crate::schema::assigned_subaddresses::dsl::assigned_subaddresses;
        let assigned_subaddress: AssignedSubaddress = match assigned_subaddresses
            .find(&public_address_b58)
            .get_result::<AssignedSubaddress>(conn)
        {
            Ok(t) => t,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::NotFound(public_address_b58.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        Ok(assigned_subaddress)
    }

    fn find_by_subaddress_spend_public_key(
        subaddress_spend_public_key: &RistrettoPublic,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(i64, String), WalletDbError> {
        use crate::schema::assigned_subaddresses::dsl::assigned_subaddresses;
        use crate::schema::assigned_subaddresses::{
            account_id_hex, subaddress_index, subaddress_spend_key,
        };

        let matches = assigned_subaddresses
            .select((subaddress_index, account_id_hex))
            .filter(subaddress_spend_key.eq(mc_util_serial::encode(subaddress_spend_public_key)))
            .load::<(i64, String)>(conn)?;

        if matches.len() == 0 {
            Err(WalletDbError::NotFound(format!(
                "{:?}",
                subaddress_spend_public_key
            )))
        } else if matches.len() > 1 {
            Err(WalletDbError::DuplicateEntries(format!(
                "{:?}",
                subaddress_spend_public_key
            )))
        } else {
            Ok(matches[0].clone())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_subaddress_by_spk(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);

        let subaddress_b58 =
            AssignedSubaddress::create(&account_key, None, 0, "", &wallet_db.get_conn().unwrap())
                .unwrap();

        let expected_subaddress = account_key.subaddress(0);
        let expected_subaddress_spk = expected_subaddress.spend_public_key();
        let matching = AssignedSubaddress::find_by_subaddress_spend_public_key(
            expected_subaddress_spk,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(matching.0, 0);
        assert_eq!(matching.1, AccountID::from(&account_key).to_string());
        assert_eq!(subaddress_b58, b58_encode(&expected_subaddress).unwrap());
    }
}
