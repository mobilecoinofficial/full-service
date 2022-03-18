// Copyright (c) 2020-2022 MobileCoin Inc.

//! DB impl for the view-only Txo model.

use crate::db::{
    models::{NewViewOnlyTxo, ViewOnlyAccount, ViewOnlyTxo},
    schema,
    txo::TxoID,
    view_only_account::ViewOnlyAccountModel,
    WalletDbError,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use mc_transaction_core::tx::TxOut;

pub trait ViewOnlyTxoModel {
    /// insert a new txo linked to a view-only-account
    fn create(
        tx_out: TxOut,
        value: i64,
        view_only_account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTxo, WalletDbError>;

    /// Get the details for a specific view only Txo.
    ///
    /// Returns:
    /// * ViewOnlyTxo
    fn get(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTxo, WalletDbError>;

    /// mark a view-only-txo as spent
    ///
    /// Returns:
    /// * ()
    fn mark_spent(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// list view only txos for a view only account
    ///
    /// Returns:
    /// * Vec<ViewOnlyTxo>
    fn list_for_account(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ViewOnlyTxo>, WalletDbError>;

    /// delete all view only txos for a view-only account
    fn delete_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl ViewOnlyTxoModel for ViewOnlyTxo {
    fn create(
        tx_out: TxOut,
        value: i64,
        view_only_account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTxo, WalletDbError> {
        use schema::view_only_txos;

        // Verify that the account exists.
        ViewOnlyAccount::get(view_only_account_id_hex, conn)?;

        let txo_id = TxoID::from(&tx_out);

        let new_txo = NewViewOnlyTxo {
            txo: &mc_util_serial::encode(&tx_out),
            txo_id_hex: &txo_id.to_string(),
            value,
            view_only_account_id_hex,
        };

        diesel::insert_into(view_only_txos::table)
            .values(&new_txo)
            .execute(conn)?;

        ViewOnlyTxo::get(&txo_id.to_string(), conn)
    }

    fn get(
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<ViewOnlyTxo, WalletDbError> {
        use schema::view_only_txos;

        let txo = match view_only_txos::table
            .filter(view_only_txos::txo_id_hex.eq(txo_id_hex))
            .get_result::<ViewOnlyTxo>(conn)
        {
            Ok(t) => t,
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::TxoNotFound(txo_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(txo)
    }

    fn list_for_account(
        account_id_hex: &str,
        offset: Option<i64>,
        limit: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ViewOnlyTxo>, WalletDbError> {
        use schema::view_only_txos;

        let txos_query = view_only_txos::table
            .filter(view_only_txos::view_only_account_id_hex.eq(account_id_hex));

        let txos: Vec<ViewOnlyTxo> = if let (Some(o), Some(l)) = (offset, limit) {
            txos_query.offset(o).limit(l).load(conn)?
        } else {
            txos_query.load(conn)?
        };

        Ok(txos)
    }

    fn mark_spent(
        &self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use schema::view_only_txos::dsl::{
            spent as dsl_spent, txo_id_hex as dsl_txo_id, view_only_txos,
        };

        diesel::update(view_only_txos.filter(dsl_txo_id.eq(&self.txo_id_hex)))
            .set(dsl_spent.eq(true))
            .execute(conn)?;
        Ok(())
    }

    fn delete_all_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use schema::view_only_txos::dsl::{
            view_only_account_id_hex as dsl_account_id, view_only_txos,
        };

        diesel::delete(view_only_txos.filter(dsl_account_id.eq(account_id_hex))).execute(conn)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;

    use crate::db::models::ViewOnlyAccount;

    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_transaction_core::encrypted_fog_hint::EncryptedFogHint;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_view_only_txo_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);
        let conn = wallet_db.get_conn().unwrap();

        // make fake txo
        let value: i64 = 420;
        let tx_private_key = RistrettoPrivate::from_random(&mut rng);
        let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
        let public_address = PublicAddress::new(
            &RistrettoPublic::from_random(&mut rng),
            &RistrettoPublic::from_random(&mut rng),
        );
        let fake_tx_out = TxOut::new(value as u64, &public_address, &tx_private_key, hint).unwrap();

        // make sure it fails if no matching account

        let view_only_account_id = "accountId";

        let err = ViewOnlyTxo::create(fake_tx_out.clone(), value, view_only_account_id, &conn);

        assert!(err.is_err());

        // make sure it passes with a matching account

        let view_only_account = ViewOnlyAccount::create(
            view_only_account_id,
            &RistrettoPrivate::from_random(&mut rng),
            0 as i64,
            0 as i64,
            "catcoin_name",
            &conn,
        )
        .unwrap();

        let txo_id = TxoID::from(&fake_tx_out);
        let expected = ViewOnlyTxo {
            id: 1,
            txo_id_hex: txo_id.to_string(),
            view_only_account_id_hex: view_only_account.account_id_hex.to_string(),
            txo: mc_util_serial::encode(&fake_tx_out),
            value,
            spent: false,
        };

        let created = ViewOnlyTxo::create(
            fake_tx_out.clone(),
            value,
            &view_only_account.account_id_hex,
            &conn,
        )
        .unwrap();

        assert_eq!(expected, created);

        // test marking as spent

        created.mark_spent(&conn).unwrap();

        let updated = ViewOnlyTxo::get(&txo_id.to_string(), &conn).unwrap();

        assert!(updated.spent);

        // test list for account

        let value: i64 = 420;
        let tx_private_key = RistrettoPrivate::from_random(&mut rng);
        let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
        let public_address = PublicAddress::new(
            &RistrettoPublic::from_random(&mut rng),
            &RistrettoPublic::from_random(&mut rng),
        );
        let fake_txo_two =
            TxOut::new(value as u64, &public_address, &tx_private_key, hint).unwrap();

        ViewOnlyTxo::create(
            fake_txo_two.clone(),
            value,
            &view_only_account.account_id_hex,
            &conn,
        )
        .unwrap();

        let listed =
            ViewOnlyTxo::list_for_account(&view_only_account.account_id_hex, None, None, &conn)
                .unwrap();

        assert_eq!(listed.len(), 2);

        // test delete all for account

        ViewOnlyTxo::delete_all_for_account(&view_only_account.account_id_hex, &conn).unwrap();
        let listed =
            ViewOnlyTxo::list_for_account(&view_only_account.account_id_hex, None, None, &conn)
                .unwrap();
        assert_eq!(listed.len(), 0);
    }
}
