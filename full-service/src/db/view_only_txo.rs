// Copyright (c) 2020-2022 MobileCoin Inc.

//! DB impl for the view-only Txo model.

use crate::db::{
    models::{NewViewOnlyTxo, ViewOnlyAccount, ViewOnlyTxo},
    schema,
    txo::TxoID,
    view_only_account::ViewOnlyAccountModel,
    Conn, WalletDbError,
};
use diesel::prelude::*;
use mc_common::HashMap;
use mc_transaction_core::{constants::MAX_INPUTS, ring_signature::KeyImage, tx::TxOut};

pub trait ViewOnlyTxoModel {
    /// insert a new txo linked to a view-only-account
    fn create(
        tx_out: TxOut,
        value: u64,
        subaddress_index: Option<u64>,
        received_block_index: Option<u64>,
        view_only_account_id_hex: &str,
        conn: &Conn,
    ) -> Result<ViewOnlyTxo, WalletDbError>;

    /// Get the details for a specific view only Txo.
    ///
    /// Returns:
    /// * ViewOnlyTxo
    fn get(txo_id_hex: &str, conn: &Conn) -> Result<ViewOnlyTxo, WalletDbError>;

    /// list view only txos for a view only account
    ///
    /// Returns:
    /// * Vec<ViewOnlyTxo>
    fn list_for_account(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlyTxo>, WalletDbError>;

    /// list view only txos that are unspent with key images for an account
    fn list_unspent_with_key_images(
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<HashMap<KeyImage, String>, WalletDbError>;

    /// Select a set of unspent view only Txos to reach a given value.
    ///
    /// Returns:
    /// * Vec<ViewOnlyTxo>
    fn select_unspent_view_only_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlyTxo>, WalletDbError>;

    /// get all txouts with no key image for a given account
    ///
    /// Returns:
    /// * Vec<TxOut>
    fn export_txouts_without_key_image(
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<Vec<TxOut>, WalletDbError>;

    /// updates the key image for a given txo
    ///
    /// Returns:
    /// * ViewOnlyTxo
    fn update_key_image(
        txo_id_hex: &str,
        key_image: &KeyImage,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    /// updates the spent block index for a given view only txo
    fn update_spent_block_index(
        txo_id_hex: &str,
        spent_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    /// delete all view only txos for a view-only account
    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError>;
}

impl ViewOnlyTxoModel for ViewOnlyTxo {
    // TODO: This needs to be updated for the new schema.
    fn create(
        tx_out: TxOut,
        value: u64,
        subaddress_index: Option<u64>,
        received_block_index: Option<u64>,
        view_only_account_id_hex: &str,
        conn: &Conn,
    ) -> Result<ViewOnlyTxo, WalletDbError> {
        use schema::view_only_txos;

        // Verify that the account exists.
        ViewOnlyAccount::get(view_only_account_id_hex, conn)?;

        let txo_id = TxoID::from(&tx_out);

        let new_txo = NewViewOnlyTxo {
            txo: &mc_util_serial::encode(&tx_out),
            txo_id_hex: &txo_id.to_string(),
            key_image: None,
            value: value as i64,
            public_key: &mc_util_serial::encode(&tx_out.public_key),
            view_only_account_id_hex,
            subaddress_index: subaddress_index.map(|x| x as i64),
            submitted_block_index: None,
            pending_tombstone_block_index: None,
            received_block_index: received_block_index.map(|x| x as i64),
            spent_block_index: None,
        };

        diesel::insert_into(view_only_txos::table)
            .values(&new_txo)
            .execute(conn)?;

        ViewOnlyTxo::get(&txo_id.to_string(), conn)
    }

    fn get(txo_id_hex: &str, conn: &Conn) -> Result<ViewOnlyTxo, WalletDbError> {
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
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlyTxo>, WalletDbError> {
        use schema::view_only_txos;

        let txos_query = view_only_txos::table
            .filter(view_only_txos::view_only_account_id_hex.eq(account_id_hex));

        let txos: Vec<ViewOnlyTxo> = if let (Some(o), Some(l)) = (offset, limit) {
            txos_query.offset(o as i64).limit(l as i64).load(conn)?
        } else {
            txos_query.load(conn)?
        };

        Ok(txos)
    }

    fn list_unspent_with_key_images(
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<HashMap<KeyImage, String>, WalletDbError> {
        use schema::view_only_txos;

        let results: Vec<(Option<Vec<u8>>, String)> = view_only_txos::table
            .select((view_only_txos::key_image, view_only_txos::txo_id_hex))
            .filter(view_only_txos::view_only_account_id_hex.eq(account_id_hex))
            .filter(view_only_txos::key_image.is_not_null())
            .filter(view_only_txos::subaddress_index.is_not_null())
            .filter(view_only_txos::received_block_index.is_not_null())
            .filter(view_only_txos::spent_block_index.is_null())
            .load(conn)?;

        Ok(results
            .into_iter()
            .filter_map(|(key_image, txo_id_hex)| match key_image {
                Some(key_image_encoded) => {
                    let key_image = mc_util_serial::decode(key_image_encoded.as_slice()).ok()?;
                    Some((key_image, txo_id_hex))
                }
                None => None,
            })
            .collect())
    }

    // This is a direct port of txo selection and
    // the whole things needs a nice big refactor
    // to make it happy.
    fn select_unspent_view_only_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlyTxo>, WalletDbError> {
        use schema::view_only_txos;

        let mut spendable_txos: Vec<ViewOnlyTxo> = view_only_txos::table
            .filter(view_only_txos::view_only_account_id_hex.eq(account_id_hex))
            .filter(view_only_txos::key_image.is_not_null())
            .filter(view_only_txos::subaddress_index.is_not_null())
            .filter(view_only_txos::received_block_index.is_not_null())
            .filter(view_only_txos::spent_block_index.is_null())
            .order_by(view_only_txos::value.desc())
            .load(conn)?;

        if spendable_txos.is_empty() {
            return Err(WalletDbError::NoSpendableTxos);
        }

        let max_spendable_in_wallet: u128 = spendable_txos
            .iter()
            .take(MAX_INPUTS as usize)
            .map(|utxo| (utxo.value as u64) as u128)
            .sum();

        if target_value as u128 > max_spendable_in_wallet {
            // See if we merged the UTXOs we would be able to spend this amount.
            let total_unspent_value_in_wallet: u128 = spendable_txos
                .iter()
                .map(|utxo| (utxo.value as u64) as u128)
                .sum();
            if total_unspent_value_in_wallet >= target_value as u128 {
                return Err(WalletDbError::InsufficientFundsFragmentedTxos);
            } else {
                return Err(WalletDbError::InsufficientFundsUnderMaxSpendable(format!(
                    "Max spendable value in wallet: {:?}, but target value: {:?}",
                    max_spendable_in_wallet, target_value
                )));
            }
        }

        let mut selected_utxos: Vec<ViewOnlyTxo> = Vec::new();
        let mut total: u64 = 0;
        loop {
            if total >= target_value {
                break;
            }

            // Grab the next (smallest) utxo, in order to opportunistically sweep up dust
            let next_utxo = spendable_txos.pop().ok_or_else(|| {
                WalletDbError::InsufficientFunds(format!(
                    "Not enough Txos to sum to target value: {:?}",
                    target_value
                ))
            })?;
            selected_utxos.push(next_utxo.clone());
            total += next_utxo.value as u64;

            // Cap at maximum allowed inputs.
            if selected_utxos.len() > MAX_INPUTS as usize {
                // Remove the lowest utxo.
                let removed = selected_utxos.remove(0);
                total -= removed.value as u64;
            }
        }

        if selected_utxos.is_empty() || selected_utxos.len() > MAX_INPUTS as usize {
            return Err(WalletDbError::InsufficientFunds(
                "Logic error. Could not select Txos despite having sufficient funds".to_string(),
            ));
        }

        Ok(selected_utxos)
    }

    fn update_key_image(
        txo_id_hex: &str,
        key_image: &KeyImage,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use schema::view_only_txos::dsl::{
            key_image as dsl_key_image, txo_id_hex as dsl_txo_id, view_only_txos,
        };

        // assert txo exists
        ViewOnlyTxo::get(txo_id_hex, conn)?;

        diesel::update(view_only_txos.filter(dsl_txo_id.eq(txo_id_hex)))
            .set(dsl_key_image.eq(mc_util_serial::encode(key_image)))
            .execute(conn)?;
        Ok(())
    }

    fn update_spent_block_index(
        txo_id_hex: &str,
        spent_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use schema::view_only_txos;

        diesel::update(view_only_txos::table.filter(view_only_txos::txo_id_hex.eq(txo_id_hex)))
            .set((view_only_txos::spent_block_index.eq(spent_block_index as i64),))
            .execute(conn)?;
        Ok(())
    }

    fn export_txouts_without_key_image(
        account_id_hex: &str,
        conn: &Conn,
    ) -> Result<Vec<TxOut>, WalletDbError> {
        use schema::view_only_txos::dsl::{
            key_image as dsl_key_image, view_only_account_id_hex as dsl_account_id,
        };

        let txos: Vec<ViewOnlyTxo> = schema::view_only_txos::table
            .filter(dsl_account_id.eq(account_id_hex))
            .filter(dsl_key_image.is_null())
            .load(conn)?;

        let mut txouts: Vec<TxOut> = Vec::new();

        for txo in txos {
            let txout: TxOut = mc_util_serial::decode(&txo.txo)?;
            txouts.push(txout);
        }

        Ok(txouts)
    }

    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError> {
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
    use mc_crypto_rand::RngCore;
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
        let value = 420;
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
            0,
            0,
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
            key_image: None,
            public_key: mc_util_serial::encode(&fake_tx_out.public_key),
            value: value as i64,
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

        ViewOnlyTxo::set_spent(vec![txo_id.to_string()], &conn).unwrap();

        let updated = ViewOnlyTxo::get(&txo_id.to_string(), &conn).unwrap();

        assert!(updated.spent);

        // expect error if attempting to mark txo that does not exist
        let err = ViewOnlyTxo::set_spent(vec!["abcd123".to_string()], &conn);
        assert!(err.is_err());

        // test list for account

        let value = 420;
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

        // test update key image

        let value = 420;
        let tx_private_key = RistrettoPrivate::from_random(&mut rng);
        let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
        let public_address = PublicAddress::new(
            &RistrettoPublic::from_random(&mut rng),
            &RistrettoPublic::from_random(&mut rng),
        );
        let fake_txo_three =
            TxOut::new(value as u64, &public_address, &tx_private_key, hint).unwrap();

        let new_txo = ViewOnlyTxo::create(
            fake_txo_three.clone(),
            value,
            &view_only_account.account_id_hex,
            &conn,
        )
        .unwrap();

        let key_image = KeyImage::from(rng.next_u64());
        ViewOnlyTxo::update_key_image(&new_txo.txo_id_hex, &key_image, &conn).unwrap();
        let found = ViewOnlyTxo::get(&new_txo.txo_id_hex, &conn).unwrap();

        assert_eq!(
            key_image,
            mc_util_serial::decode(&found.key_image.unwrap()).unwrap()
        );

        // test export_txouts_without_key_image. assuming 2 vo txos without key image
        // and one with from the tests above

        let txouts =
            ViewOnlyTxo::export_txouts_without_key_image(&view_only_account.account_id_hex, &conn)
                .unwrap();
        assert_eq!(txouts.len(), 2);
        assert!(txouts.contains(&fake_tx_out));
        assert!(txouts.contains(&fake_txo_two));

        // test delete all for account

        ViewOnlyTxo::delete_all_for_account(&view_only_account.account_id_hex, &conn).unwrap();
        let listed =
            ViewOnlyTxo::list_for_account(&view_only_account.account_id_hex, None, None, &conn)
                .unwrap();
        assert_eq!(listed.len(), 0);
    }
}
