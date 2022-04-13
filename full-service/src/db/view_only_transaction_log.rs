// Copyright (c) 2020-2022 MobileCoin Inc.

//! DB impl for the view-only transaction log model.

use crate::db::{
    models::{NewViewOnlyTransactionLog, ViewOnlyTransactionLog},
    schema, WalletDbError,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};

pub trait ViewOnlyTransactionLogModel {
    /// insert a new view only transaction log
    fn create(
        change_txo_id_hex: &str,
        input_txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTransactionLog, WalletDbError>;

    /// get a view only transaction log by change txo id
    fn get_by_change_txo_id(
        change_txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTransactionLog, WalletDbError>;

    /// get a all view only transaction logs for a change txo id
    fn find_all_by_change_txo_id(
        change_txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ViewOnlyTransactionLog>, WalletDbError>;
}

impl ViewOnlyTransactionLogModel for ViewOnlyTransactionLog {
    fn create(
        change_txo_id_hex: &str,
        input_txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTransactionLog, WalletDbError> {
        use schema::view_only_transaction_logs;

        let new_log = NewViewOnlyTransactionLog {
            change_txo_id_hex,
            input_txo_id_hex,
        };

        diesel::insert_into(view_only_transaction_logs::table)
            .values(&new_log)
            .execute(conn)?;

        ViewOnlyTransactionLog::get_by_change_txo_id(&change_txo_id_hex.to_string(), conn)
    }

    fn get_by_change_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTransactionLog, WalletDbError> {
        use schema::view_only_transaction_logs::dsl::{
            change_txo_id_hex, view_only_transaction_logs,
        };

        match view_only_transaction_logs
            .filter((change_txo_id_hex).eq(&txo_id_hex))
            .get_result::<ViewOnlyTransactionLog>(conn)
        {
            Ok(a) => Ok(a),
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::TransactionLogNotFound(
                txo_id_hex.to_string(),
            )),
            Err(e) => Err(e.into()),
        }
    }

    fn find_all_by_change_txo_id(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ViewOnlyTransactionLog>, WalletDbError> {
        use schema::view_only_transaction_logs::dsl::{
            change_txo_id_hex, view_only_transaction_logs,
        };

        match view_only_transaction_logs
            .filter((change_txo_id_hex).eq(&txo_id_hex))
            .load(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => Err(WalletDbError::TransactionLogNotFound(
                txo_id_hex.to_string(),
            )),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            models::{ViewOnlyAccount, ViewOnlyTransactionLog},
            txo::TxoID,
            view_only_account::ViewOnlyAccountModel,
        },
        test_utils::WalletDbTestContext,
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_transaction_core::{encrypted_fog_hint::EncryptedFogHint, tx::TxOut};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_view_only_transaction_log_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

        // make fake txos, view only txos & account
        let view_only_account_id = "accountId";
        let value = 420;
        let tx_private_key_1 = RistrettoPrivate::from_random(&mut rng);
        let tx_private_key_2 = RistrettoPrivate::from_random(&mut rng);
        let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
        let public_address = PublicAddress::new(
            &RistrettoPublic::from_random(&mut rng),
            &RistrettoPublic::from_random(&mut rng),
        );
        let fake_input_tx_out =
            TxOut::new(value, &public_address, &tx_private_key_1, hint.clone()).unwrap();
        let fake_change_tx_out =
            TxOut::new(value, &public_address, &tx_private_key_2, hint.clone()).unwrap();

        ViewOnlyAccount::create(
            view_only_account_id,
            &RistrettoPrivate::from_random(&mut rng),
            0,
            0,
            "catcoin_name",
            &conn,
        )
        .unwrap();

        let input_txo_id = TxoID::from(&fake_input_tx_out);
        let change_txo_id = TxoID::from(&fake_change_tx_out);

        let expected = ViewOnlyTransactionLog {
            id: 1,
            change_txo_id_hex: change_txo_id.to_string(),
            input_txo_id_hex: input_txo_id.to_string(),
        };

        let created = ViewOnlyTransactionLog::create(
            &change_txo_id.to_string(),
            &input_txo_id.to_string(),
            &conn,
        )
        .unwrap();

        assert_eq!(created, expected);

        // test find all by change id
        let tx_private_key_3 = RistrettoPrivate::from_random(&mut rng);
        let fake_input_tx_out_two = TxOut::new(
            value as u64,
            &public_address,
            &tx_private_key_3,
            hint.clone(),
        )
        .unwrap();
        let input_txo_id_2 = TxoID::from(&fake_input_tx_out_two);

        ViewOnlyTransactionLog::create(
            &change_txo_id.to_string(),
            &input_txo_id_2.to_string(),
            &conn,
        )
        .unwrap();

        let found_all =
            ViewOnlyTransactionLog::find_all_by_change_txo_id(&change_txo_id.to_string(), &conn)
                .unwrap();

        assert_eq!(found_all.len(), 2);
    }
}
