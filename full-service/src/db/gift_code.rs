// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Gift Code Model.

use crate::{
    db::{
        account::AccountID,
        models::{GiftCode, NewGiftCode},
        txo::TxoID,
        WalletDbError,
    },
    service::gift_code::EncodedGiftCode,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};
use displaydoc::Display;
use mc_account_keys::RootEntropy;
use mc_crypto_keys::CompressedRistrettoPublic;

#[derive(Display, Debug)]
pub enum GiftCodeDbError {
    /// Could not get gift code: {0}
    GiftCodeNotFound(String),
}

pub trait GiftCodeModel {
    /// Create a gift code.
    ///
    /// Gift code includes:
    /// * entropy
    /// * txo public key
    /// * memo
    ///
    /// GiftCodeModel::create should be called after the account has already
    /// been inserted into the DB, the txo has already been deposited to
    /// that account, and the transaction_log has been stored for that
    /// deposit, all of which are handled by the GiftCodeService.
    ///
    /// Returns:
    /// * Gift code encoded as b58 string.
    #[allow(clippy::too_many_arguments)]
    fn create(
        gift_code_b58: &EncodedGiftCode,
        root_entropy: Option<&RootEntropy>,
        bip39_entropy: Option<&Vec<u8>>,
        txo_public_key: &CompressedRistrettoPublic,
        value: i64,
        memo: &str,
        account_id: &AccountID,
        txo_id: &TxoID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<GiftCode, WalletDbError>;

    /// Get the details of a specific Gift Code.
    fn get(
        gift_code_b58: &EncodedGiftCode,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<GiftCode, WalletDbError>;

    /// Get all Gift Codes in this wallet.
    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<GiftCode>, WalletDbError>;

    /// Delete a gift code.
    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl GiftCodeModel for GiftCode {
    fn create(
        gift_code_b58: &EncodedGiftCode,
        root_entropy: Option<&RootEntropy>,
        bip39_entropy: Option<&Vec<u8>>,
        txo_public_key: &CompressedRistrettoPublic,
        value: i64,
        memo: &str,
        account_id: &AccountID,
        txo_id: &TxoID,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<GiftCode, WalletDbError> {
        use crate::db::schema::gift_codes;

        // Insert the gift code to our gift code table.
        let new_gift_code = NewGiftCode {
            gift_code_b58: &gift_code_b58.to_string(),
            root_entropy: root_entropy.map(|entropy| entropy.bytes.to_vec()),
            bip39_entropy: bip39_entropy.cloned(),
            txo_public_key: &mc_util_serial::encode(txo_public_key),
            value,
            memo,
            account_id_hex: &account_id.to_string(),
            txo_id_hex: &txo_id.to_string(),
        };
        diesel::insert_into(gift_codes::table)
            .values(&new_gift_code)
            .execute(conn)?;

        let gift_code = GiftCode::get(gift_code_b58, conn)?;
        Ok(gift_code)
    }

    fn get(
        gift_code_b58: &EncodedGiftCode,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<GiftCode, WalletDbError> {
        use crate::db::schema::gift_codes::dsl::{gift_code_b58 as dsl_gift_code_b58, gift_codes};

        match gift_codes
            .filter(dsl_gift_code_b58.eq(gift_code_b58.to_string()))
            .get_result::<GiftCode>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(GiftCodeDbError::GiftCodeNotFound(gift_code_b58.to_string()).into())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn list_all(
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<GiftCode>, WalletDbError> {
        use crate::db::schema::gift_codes;

        Ok(gift_codes::table
            .select(gift_codes::all_columns)
            .load::<GiftCode>(conn)?)
    }

    fn delete(
        self,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::gift_codes::dsl::{gift_code_b58, gift_codes};

        diesel::delete(gift_codes.filter(gift_code_b58.eq(&self.gift_code_b58))).execute(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_txo_for_recipient, WalletDbTestContext};
    use mc_account_keys::{AccountKey, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPublic;
    use mc_crypto_rand::rand_core::RngCore;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    // Basic test of gift codes in database.
    #[test_with_logger]
    fn test_gift_code_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_identity = RootIdentity::from_random(&mut rng);
        let gift_code_account_key = AccountKey::from(&root_identity);
        let entropy = root_identity.root_entropy;

        // The Txo we would have sent to fund this entropy
        let txo_public_key: CompressedRistrettoPublic =
            RistrettoPublic::from_random(&mut rng).into();
        // Note: This value isn't actually associated with the txo_public_key, but is
        // sufficient for this test to merely log a value.
        let value = rng.next_u64();

        let (tx_out, _key_image) =
            create_test_txo_for_recipient(&gift_code_account_key, 0, value, &mut rng);

        let memo = String::from("Test");

        let mut tx_log_bytes = [0u8; 32];
        rng.fill_bytes(&mut tx_log_bytes);

        let gift_code = GiftCode::create(
            &EncodedGiftCode("gk7CcXuK5RKNW13LvrWY156ZLjaoHaXxLedqACZsw3w6FfF6TR4TVzaAQkH5EHxaw54DnGWRJPA31PpcmvGLoArZbDRj1kBhcTusE8AVW4Mj7QT5".to_string()),
            Some(&entropy),
            None,
            &txo_public_key,
            value as i64,
            &memo,
            &AccountID::from(&gift_code_account_key),
            &TxoID::from(&tx_out),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let gotten = GiftCode::get(
            &EncodedGiftCode(gift_code.gift_code_b58),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let expected_gift_code = GiftCode {
            id: 1,
            gift_code_b58: gotten.gift_code_b58.clone(),
            root_entropy: Some(entropy.bytes.to_vec()),
            bip39_entropy: None,
            txo_public_key: mc_util_serial::encode(&txo_public_key),
            value: value as i64,
            memo,
            account_id_hex: AccountID::from(&gift_code_account_key).to_string(),
            txo_id_hex: TxoID::from(&tx_out).to_string(),
        };
        assert_eq!(gotten, expected_gift_code);
        assert_eq!(gotten.root_entropy, Some(entropy.bytes.to_vec()));
        assert_eq!(gotten.bip39_entropy, None);

        let all_gift_codes = GiftCode::list_all(&wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(all_gift_codes.len(), 1);
        assert_eq!(all_gift_codes[0], expected_gift_code);
    }
}
