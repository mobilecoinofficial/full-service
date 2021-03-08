// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Gift Code Model.

use crate::{
    db::{
        models::{GiftCode, NewGiftCode},
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
        entropy: &RootEntropy,
        txo_public_key: &CompressedRistrettoPublic,
        value: i64,
        memo: String,
        account_id: i32,
        build_log_id: Option<i32>,
        consume_log_id: Option<i32>,
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

    /// Update the consume_log_id for the given gift code.
    ///
    /// This method is used when a gift code was created in this wallet, and is
    /// later consumed by an account also in this wallet.
    fn update_consume_log_id(
        &self,
        consume_log_id: i32,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError>;
}

impl GiftCodeModel for GiftCode {
    fn create(
        entropy: &RootEntropy,
        txo_public_key: &CompressedRistrettoPublic,
        value: i64,
        memo: String,
        account_id: i32,
        build_log_id: Option<i32>,
        consume_log_id: Option<i32>,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<GiftCode, WalletDbError> {
        use crate::db::schema::gift_codes;

        let proto_tx_pubkey: mc_api::external::CompressedRistretto = txo_public_key.into();

        // Create the gift_code_b58 using the printable wrapper for a TransferPayload.
        let mut gift_code_payload = mc_mobilecoind_api::printable::TransferPayload::new();
        gift_code_payload.set_entropy(entropy.as_ref().to_vec());
        gift_code_payload.set_tx_out_public_key(proto_tx_pubkey);
        gift_code_payload.set_memo(memo.clone());

        let mut gift_code_wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
        gift_code_wrapper.set_transfer_payload(gift_code_payload);

        let gift_code_b58 = gift_code_wrapper.b58_encode()?;

        // Insert the gift code to our gift code table.
        let new_gift_code = NewGiftCode {
            gift_code_b58: &gift_code_b58,
            entropy: &entropy.bytes.to_vec(),
            txo_public_key: &txo_public_key.as_bytes().to_vec(),
            value,
            memo: &memo,
            account_id,
            build_log_id,
            consume_log_id,
        };

        diesel::insert_into(gift_codes::table)
            .values(&new_gift_code)
            .execute(conn)?;

        let gift_code = GiftCode::get(&EncodedGiftCode(gift_code_b58), conn)?;
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

    fn update_consume_log_id(
        &self,
        consume_log_id: i32,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::gift_codes::dsl::{gift_code_b58, gift_codes};

        diesel::update(gift_codes.filter(gift_code_b58.eq(&self.gift_code_b58)))
            .set(crate::db::schema::gift_codes::consume_log_id.eq(consume_log_id))
            .execute(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
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
        let entropy = root_identity.root_entropy;

        // The Txo we would have sent to fund this entropy
        let txo_public_key: CompressedRistrettoPublic =
            RistrettoPublic::from_random(&mut rng).into();
        // Note: This value isn't actually associated with the txo_public_key, but is
        // sufficient for this test to merely log a value.
        let value = rng.next_u64();

        let memo = "Test".to_string();
        let account_id = 132;
        let build_log_id = 6873;

        let gift_code = GiftCode::create(
            &entropy,
            &txo_public_key,
            value as i64,
            memo.clone(),
            account_id,
            Some(build_log_id),
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(gift_code.gift_code_b58, "gk7CcXuK5RKNW13LvrWY156ZLjaoHaXxLedqACZsw3w6FfF6TR4TVzaAQkH5EHxaw54DnGWRJPA31PpcmvGLoArZbDRj1kBhcTusE8AVW4Mj7QT5");

        let gotten = GiftCode::get(
            &EncodedGiftCode(gift_code.gift_code_b58),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let expected_gift_code = GiftCode {
            id: 1,
            gift_code_b58: gotten.gift_code_b58.clone(),
            entropy: entropy.bytes.to_vec(),
            txo_public_key: txo_public_key.as_bytes().to_vec(),
            value: value as i64,
            memo,
            account_id,
            build_log_id: Some(build_log_id),
            consume_log_id: None,
        };
        assert_eq!(gotten, expected_gift_code);
        assert_eq!(gotten.entropy, entropy.bytes.to_vec());

        let all_gift_codes = GiftCode::list_all(&wallet_db.get_conn().unwrap()).unwrap();
        assert_eq!(all_gift_codes.len(), 1);
        assert_eq!(all_gift_codes[0], expected_gift_code);

        // Test update
        gotten
            .update_consume_log_id(16, &wallet_db.get_conn().unwrap())
            .unwrap();
        let gotten2 = GiftCode::get(
            &EncodedGiftCode(gotten.gift_code_b58),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        assert_eq!(gotten2.consume_log_id.unwrap(), 16);
    }
}
