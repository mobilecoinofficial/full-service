// Copyright (c) 2020 MobileCoin Inc.

//! DB impl for the Txo model.

use crate::{
    db::b58_encode,
    db_models::{
        account::AccountModel, account_txo_status::AccountTxoStatusModel,
        assigned_subaddress::AssignedSubaddressModel,
    },
    error::WalletDbError,
    models::{Account, AccountTxoStatus, AssignedSubaddress, NewTxo, Txo},
};

use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_transaction_core::{constants::MAX_INPUTS, ring_signature::KeyImage, tx::TxOut};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use std::iter::FromIterator;

#[derive(Debug)]
pub struct TxoID(String);

impl From<&TxOut> for TxoID {
    fn from(src: &TxOut) -> TxoID {
        /// The txo ID is derived from the contents of the txo
        #[derive(Digestible)]
        struct ConstTxoData {
            pub txo: TxOut,
        }
        let const_data = ConstTxoData { txo: src.clone() };
        let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"txo_data");
        Self(hex::encode(temp))
    }
}

impl TxoID {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub trait TxoModel {
    /// Create a received Txo.
    fn create_received(
        txo: TxOut,
        subaddress_index: Option<i64>,
        key_image: Option<KeyImage>,
        value: u64,
        received_block_height: i64,
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;

    /// Update an existing Txo when it is received in the ledger.
    /// A Txo can be created before being received if it is minted, for example.
    fn update_received(
        &self,
        account_id_hex: &str,
        subaddress_index: Option<i64>,
        key_image: Option<KeyImage>,
        received_block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    /// Update an existing Txo to spendable by including its subaddress_index and key_image.
    fn update_to_spendable(
        &self,
        received_subaddress_index: Option<i64>,
        received_key_image: Option<KeyImage>,
        block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;

    fn list_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError>;

    fn list_by_status(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<HashMap<String, Vec<Txo>>, WalletDbError>;

    fn get(
        account_id_hex: &str,
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(Txo, AccountTxoStatus, Option<AssignedSubaddress>), WalletDbError>;

    fn select_by_id(
        txo_ids: &Vec<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError>;

    fn select_unspent_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        max_spendable_value: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError>;
}

impl TxoModel for Txo {
    fn create_received(
        txo: TxOut,
        subaddress_index: Option<i64>,
        key_image: Option<KeyImage>,
        value: u64,
        received_block_height: i64,
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        use crate::schema::txos::dsl::txos;

        let txo_id = TxoID::from(&txo);

        // If we already have this TXO (e.g. from minting in a previous transaction), we need to update it
        match txos.find(&txo_id.to_string()).get_result::<Txo>(conn) {
            Ok(txo) => {
                txo.update_received(
                    account_id_hex,
                    subaddress_index,
                    key_image,
                    received_block_height,
                    conn,
                )?;
            }
            // If we don't already have this TXO, create a new entry
            Err(diesel::result::Error::NotFound) => {
                let key_image_bytes = key_image.map(|k| mc_util_serial::encode(&k));
                let new_txo = NewTxo {
                    txo_id_hex: &txo_id.to_string(),
                    value: value as i64,
                    target_key: &mc_util_serial::encode(&txo.target_key),
                    public_key: &mc_util_serial::encode(&txo.public_key),
                    e_fog_hint: &mc_util_serial::encode(&txo.e_fog_hint),
                    txo: &mc_util_serial::encode(&txo),
                    subaddress_index,
                    key_image: key_image_bytes.as_ref(),
                    received_block_height: Some(received_block_height as i64),
                    spent_tombstone_block_height: None,
                    spent_block_height: None,
                    proof: None,
                };

                diesel::insert_into(crate::schema::txos::table)
                    .values(&new_txo)
                    .execute(conn)?;

                let status = if subaddress_index.is_some() {
                    "unspent"
                } else {
                    // Note: An orphaned Txo cannot be spent until the subaddress is recovered.
                    "orphaned"
                };
                AccountTxoStatus::create(
                    account_id_hex,
                    &txo_id.to_string(),
                    status,
                    "received",
                    conn,
                )?;
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(txo_id.to_string())
    }

    fn update_received(
        &self,
        account_id_hex: &str,
        received_subaddress_index: Option<i64>,
        received_key_image: Option<KeyImage>,
        received_block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        // get the type of this TXO
        let account_txo_status = AccountTxoStatus::get(account_id_hex, &self.txo_id_hex, &conn)?;

        // For TXOs that we sent previously, they are either change, or we sent to ourselves
        // for some other reason. Their status will be "secreted" in either case.
        if account_txo_status.txo_type == "minted" {
            // Update received block height and subaddress index
            self.update_to_spendable(
                received_subaddress_index,
                received_key_image,
                received_block_height,
                &conn,
            )?;

            // Update the status to unspent - all TXOs set lifecycle to unspent when first received
            account_txo_status.set_unspent(&conn)?;
        } else if account_txo_status.txo_type == "received".to_string() {
            // If the existing Txo subaddress is null and we have the received subaddress
            // now, then we want to update to received subaddress. Otherwise, it will remain orphaned.
            // Do not update to unspent, because this Txo may have already been processed and is
            // annotated correctly if spent.
            if received_subaddress_index.is_some() {
                self.update_to_spendable(
                    received_subaddress_index,
                    received_key_image,
                    received_block_height,
                    &conn,
                )?;
            }
        } else {
            panic!("New txo_type must be handled");
        }

        // If this Txo was previously orphaned, we can now update it, and make it spendable
        if account_txo_status.txo_status == "orphaned" {
            self.update_to_spendable(
                received_subaddress_index,
                received_key_image,
                received_block_height,
                &conn,
            )?;
            account_txo_status.set_unspent(conn)?;
        }

        return Ok(());
    }

    fn update_to_spendable(
        &self,
        received_subaddress_index: Option<i64>,
        received_key_image: Option<KeyImage>,
        block_height: i64,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::schema::txos::{key_image, received_block_height, subaddress_index};

        // Verify that we have a subaddress, otherwise this transaction will be
        // unspendable.
        if received_subaddress_index.is_none() || received_key_image.is_none() {
            return Err(WalletDbError::NullSubaddressOnReceived);
        }

        let encoded_key_image = received_key_image.map(|k| mc_util_serial::encode(&k));
        diesel::update(self)
            .set((
                received_block_height.eq(Some(block_height)),
                subaddress_index.eq(received_subaddress_index),
                key_image.eq(encoded_key_image),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn list_for_account(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError> {
        use crate::schema::account_txo_statuses;
        use crate::schema::txos;

        // FIXME: join 3 tables to also get AssignedSubaddresses
        let results: Vec<(Txo, AccountTxoStatus)> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))),
            )
            .select((txos::all_columns, account_txo_statuses::all_columns))
            .load(conn)?;
        Ok(results)
    }

    fn list_by_status(
        account_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<HashMap<String, Vec<Txo>>, WalletDbError> {
        use crate::schema::account_txo_statuses;
        use crate::schema::txos;

        // FIXME: don't do 4 queries
        let unspent: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq("unspent"))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        let pending: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq("pending"))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        let spent: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq("spent"))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        // FIXME: Maybe we don't want to expose this in the balance
        let secreted: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq("secreted"))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        let orphaned: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq("orphaned"))),
            )
            .select(txos::all_columns)
            .load(conn)?;

        let results = HashMap::from_iter(vec![
            ("unspent".to_string(), unspent),
            ("pending".to_string(), pending),
            ("spent".to_string(), spent),
            ("secreted".to_string(), secreted),
            ("orphaned".to_string(), orphaned),
        ]);
        Ok(results)
    }

    fn get(
        account_id_hex: &str,
        txo_id_hex: &str,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(Txo, AccountTxoStatus, Option<AssignedSubaddress>), WalletDbError> {
        use crate::schema::txos::dsl::txos;

        let txo: Txo = match txos.find(txo_id_hex).get_result::<Txo>(conn) {
            Ok(t) => t,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::NotFound(txo_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        let account_txo_status: AccountTxoStatus =
            AccountTxoStatus::get(account_id_hex, txo_id_hex, conn)?;

        // Get subaddress key from account_key and txo subaddress
        let account: Account = Account::get(account_id_hex, conn)?;

        // Get the subaddress details if assigned
        let assigned_subaddress = if let Some(subaddress_index) = txo.subaddress_index {
            let account_key: AccountKey = mc_util_serial::decode(&account.encrypted_account_key)?;
            let subaddress = account_key.subaddress(subaddress_index as u64);
            let subaddress_b58 = b58_encode(&subaddress)?;

            Some(AssignedSubaddress::get(&subaddress_b58, conn)?)
        } else {
            None
        };

        Ok((txo, account_txo_status, assigned_subaddress))
    }

    fn select_by_id(
        txo_ids: &Vec<String>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<(Txo, AccountTxoStatus)>, WalletDbError> {
        use crate::schema::account_txo_statuses;
        use crate::schema::txos;

        let txos: Vec<(Txo, AccountTxoStatus)> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(txos::txo_id_hex.eq_any(txo_ids))),
            )
            .select((txos::all_columns, account_txo_statuses::all_columns))
            .load(conn)?;
        Ok(txos)
    }

    fn select_unspent_txos_for_value(
        account_id_hex: &str,
        target_value: u64,
        max_spendable_value: Option<i64>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<Txo>, WalletDbError> {
        use crate::schema::account_txo_statuses;
        use crate::schema::txos;

        let mut spendable_txos: Vec<Txo> = txos::table
            .inner_join(
                account_txo_statuses::table.on(txos::txo_id_hex
                    .eq(account_txo_statuses::txo_id_hex)
                    .and(account_txo_statuses::account_id_hex.eq(account_id_hex))
                    .and(account_txo_statuses::txo_status.eq("unspent"))
                    .and(txos::subaddress_index.is_not_null())
                    .and(txos::key_image.is_not_null()) // Could technically recreate with subaddress
                    .and(txos::value.lt(max_spendable_value.unwrap_or(i64::MAX)))),
            )
            .select(txos::all_columns)
            .order_by(txos::value.desc())
            .load(conn)?;

        if spendable_txos.is_empty() {
            return Err(WalletDbError::NoSpendableTxos);
        }

        // The maximum spendable is limited by the maximal number of inputs we can use. Since
        // the txos are sorted by decreasing value, this is the maximum value we can possibly spend
        // in one transaction.
        let max_spendable_in_wallet = spendable_txos
            .iter()
            .take(MAX_INPUTS as usize)
            .map(|utxo| utxo.value as u64)
            .sum();
        if target_value > max_spendable_in_wallet {
            // See if we merged the UTXOs we would be able to spend this amount.
            let total_unspent_value_in_wallet: u64 =
                spendable_txos.iter().map(|utxo| utxo.value as u64).sum();
            if total_unspent_value_in_wallet >= target_value {
                return Err(WalletDbError::InsufficientFundsFragmentedTxos);
            } else {
                return Err(WalletDbError::InsufficientFunds(format!(
                    "Max spendable value in wallet: {:?}, but target value: {:?}",
                    max_spendable_in_wallet, target_value
                )));
            }
        }

        // Select the actual Txos to spend. We want to opportunistically fill up the input slots
        // with dust, from any subaddress, so we take from the back of the Txo vec. This is
        // a knapsack problem, and the selection could be improved. For now, we simply move the
        // window of MAX_INPUTS up from the back of the sorted vector until we have a window with
        // a large enough sum.
        let mut selected_utxos: Vec<Txo> = Vec::new();
        let mut total: u64 = 0;
        loop {
            if total >= target_value {
                break;
            }

            // Grab the next (smallest) utxo, in order to opportunistically sweep up dust
            let next_utxo = spendable_txos
                .pop()
                .ok_or(WalletDbError::InsufficientFunds(format!(
                    "Not enough Txos to sum to target value: {:?}",
                    target_value
                )))?;
            selected_utxos.push(next_utxo.clone());
            total += next_utxo.value as u64;

            // Cap at maximum allowed inputs.
            if selected_utxos.len() > MAX_INPUTS as usize {
                // Remove the lowest utxo.
                selected_utxos.remove(0);
            }
        }

        if selected_utxos.is_empty() || selected_utxos.len() > MAX_INPUTS as usize {
            return Err(WalletDbError::InsufficientFunds(
                "Logic error. Could not select Txos despite having sufficient funds".to_string(),
            ));
        }

        Ok(selected_utxos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::WalletDb;
    use crate::db_models::account::{AccountID, AccountModel};
    use crate::models::Account;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::AccountKey;
    use mc_common::{
        logger::{test_with_logger, Logger},
        HashSet,
    };
    use mc_crypto_keys::RistrettoPrivate;
    use mc_transaction_core::{
        constants::MINIMUM_FEE, encrypted_fog_hint::EncryptedFogHint, ring_signature::KeyImage,
        tx::TxOut,
    };
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::iter::FromIterator;

    const MOB: i64 = 1_000_000_000_000;

    fn create_test_received(
        account_key: &AccountKey,
        recipient_subaddress_index: u64,
        value: u64,
        received_block_height: u64,
        rng: &mut StdRng,
        wallet_db: &WalletDb,
    ) -> (String, TxOut, KeyImage) {
        let recipient = account_key.subaddress(recipient_subaddress_index);
        let tx_private_key = RistrettoPrivate::from_random(rng);
        let hint = EncryptedFogHint::fake_onetime_hint(rng);
        let txo = TxOut::new(value, &recipient, &tx_private_key, hint).unwrap();

        // Get KeyImage from the onetime private key
        let key_image = KeyImage::from(&tx_private_key);

        let txo_id_hex = Txo::create_received(
            txo.clone(),
            Some(recipient_subaddress_index as i64),
            Some(key_image),
            value,
            received_block_height as i64,
            &AccountID::from(account_key).to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        (txo_id_hex, txo, key_image)
    }

    #[test_with_logger]
    fn test_received_tx_lifecycle(logger: Logger) {
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

        // Create TXO for the account
        let (txo_hex, txo, key_image) =
            create_test_received(&account_key, 0, 10, 144, &mut rng, &wallet_db);

        let txos = Txo::list_for_account(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(txos.len(), 1);

        let expected_txo = Txo {
            txo_id_hex: txo_hex.clone(),
            value: 10,
            target_key: mc_util_serial::encode(&txo.target_key),
            public_key: mc_util_serial::encode(&txo.public_key),
            e_fog_hint: mc_util_serial::encode(&txo.e_fog_hint),
            txo: mc_util_serial::encode(&txo),
            subaddress_index: Some(0),
            key_image: Some(mc_util_serial::encode(&key_image)),
            received_block_height: Some(144),
            spent_tombstone_block_height: None,
            spent_block_height: None,
            proof: None,
        };
        // Verify that the statuses table was updated correctly
        let expected_txo_status = AccountTxoStatus {
            account_id_hex: account_id_hex.to_string(),
            txo_id_hex: txo_hex,
            txo_status: "unspent".to_string(),
            txo_type: "received".to_string(),
        };
        assert_eq!(txos[0].0, expected_txo);
        assert_eq!(txos[0].1, expected_txo_status);

        // Verify that the status filter works as well
        let balances =
            Txo::list_by_status(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(balances["unspent"].len(), 1);

        // Now we'll "spend" the TXO
        // FIXME TODO: construct transaction proposal to spend it, maybe needs a helper in test_utils
        // self.update_submitted_transaction(tx_proposal)?;

        // Now we'll process the ledger and verify that the TXO was spent
        let spent_block_height = 365;

        wallet_db
            .update_spent_and_increment_next_block(
                &account_id_hex,
                spent_block_height,
                vec![key_image],
            )
            .unwrap();

        let txos = Txo::list_for_account(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(txos.len(), 1);
        assert_eq!(
            txos[0].0.spent_block_height.unwrap(),
            spent_block_height as i64
        );
        assert_eq!(txos[0].1.txo_status, "spent".to_string());

        // Verify that the next block height is + 1
        let account = Account::get(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(account.next_block, spent_block_height + 1);

        // Verify that there are no unspent txos
        let balances =
            Txo::list_by_status(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        assert!(balances["unspent"].is_empty());
    }

    #[test_with_logger]
    fn test_select_txos_for_value(logger: Logger) {
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

        // Create some TXOs for the account
        // [100, 200, 300, ... 2000]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received(
                &account_key,
                0,
                (100 * MOB * i) as u64, // 100.0 MOB * i
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        // Greedily take smallest to exact value
        let txos_for_value = Txo::select_unspent_txos_for_value(
            &account_id_hex,
            300 * MOB as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value));
        assert_eq!(
            result_set,
            HashSet::<i64>::from_iter(vec![100 * MOB, 200 * MOB])
        );

        // Once we include the fee, we need another txo
        let txos_for_value = Txo::select_unspent_txos_for_value(
            &account_id_hex,
            300 * MOB as u64 + MINIMUM_FEE,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value));
        assert_eq!(
            result_set,
            HashSet::<i64>::from_iter(vec![100 * MOB, 200 * MOB, 300 * MOB])
        );

        // Setting max spendable value gives us insufficient funds - only allows 100
        let res = Txo::select_unspent_txos_for_value(
            &account_id_hex,
            300 * MOB as u64 + MINIMUM_FEE,
            Some(200 * MOB),
            &wallet_db.get_conn().unwrap(),
        );
        match res {
            Err(WalletDbError::InsufficientFunds(_)) => {}
            Ok(_) => panic!("Should error with InsufficientFunds"),
            Err(_) => panic!("Should error with InsufficientFunds"),
        }

        // sum(300..1800) to get a window where we had to increase past the smallest txos,
        // and also fill up all 16 input slots.
        let txos_for_value = Txo::select_unspent_txos_for_value(
            &account_id_hex,
            16800 * MOB as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let result_set = HashSet::from_iter(txos_for_value.iter().map(|t| t.value));
        assert_eq!(
            result_set,
            HashSet::<i64>::from_iter(vec![
                300 * MOB,
                400 * MOB,
                500 * MOB,
                600 * MOB,
                700 * MOB,
                800 * MOB,
                900 * MOB,
                1000 * MOB,
                1100 * MOB,
                1200 * MOB,
                1300 * MOB,
                1400 * MOB,
                1500 * MOB,
                1600 * MOB,
                1700 * MOB,
                1800 * MOB
            ])
        );
    }

    #[test_with_logger]
    fn test_select_txos_fragmented(logger: Logger) {
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

        // Create some TXOs for the account. Total value is 2000, but max can spend is 1600
        // [100, 100, ... 100]
        for i in 1..20 {
            let (_txo_hex, _txo, _key_image) = create_test_received(
                &account_key,
                0,
                (100 * MOB) as u64,
                (144 + i) as u64,
                &mut rng,
                &wallet_db,
            );
        }

        let res = Txo::select_unspent_txos_for_value(
            &account_id_hex,
            1800 * MOB as u64,
            None,
            &wallet_db.get_conn().unwrap(),
        );
        match res {
            Err(WalletDbError::InsufficientFundsFragmentedTxos) => {}
            Ok(_) => panic!("Should error with InsufficientFundsFragmentedTxos"),
            Err(e) => panic!(
                "Should error with InsufficientFundsFragmentedTxos but got {:?}",
                e
            ),
        }
    }

    // FIXME: once we have create_spent, then select_txos test with no spendable
}
