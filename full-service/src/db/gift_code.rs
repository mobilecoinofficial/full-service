// Copyright (c) 2020 MobileCoin Inc.

//! The Gift Code Model.

use crate::{
    db::models::{GiftCode, NewGiftCode},
    error::WalletDbError,
};
use mc_crypto_keys::CompressedRistrettoPublic;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    RunQueryDsl,
};

pub trait GiftCodeModel {
    /// Create a gift code.
    ///
    /// Gift code includes:
    /// * entropy
    /// * txo public key
    /// * memo
    ///
    /// GiftCodeModel::create should be called after the account has already been inserted into
    /// the DB, the txo has already been deposited to that account, and the transaction_log has
    /// been stored for that deposit, all of which are handled by the GiftCodeService.
    ///
    /// Returns:
    /// * Gift code encoded as b58 string.
    fn create(
        entropy: &[u8],
        txo_public_key: &CompressedRistrettoPublic,
        memo: String,
        account_id: usize,
        build_log_id: usize,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError>;
}

impl GiftCodeModel for GiftCode {
    fn create(
        entropy: &[u8],
        txo_public_key: &CompressedRistrettoPublic,
        memo: String,
        account_id: usize,
        build_log_id: usize,
        conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<String, WalletDbError> {
        use crate::db::schema::gift_codes;

        // Create the gift_code_b58 using the printable wrapper for a TransferPayload.
        let mut gift_code_payload = mc_mobilecoind_api::printable::TransferPayload::new();
        gift_code_payload.set_entropy(entropy.to_vec());
        gift_code_payload.set_tx_out_public_key(txo_public_key.into());
        gift_code_payload.set_memo(memo.clone());

        let mut gift_code_wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
        gift_code_wrapper.set_transfer_payload(gift_code_payload);

        let gift_code_b58 = gift_code_wrapper.b58_encode()?;

        // Insert the gift code to our gift code table.
        let new_gift_code = NewGiftCode {
            gift_code_b58: &gift_code_b58,
            entropy: &entropy.to_vec(),
            txo_public_key: &txo_public_key.as_bytes().to_vec(),
            memo: &memo,
            account_id: account_id as i32,
            build_log_id: Some(build_log_id as i32),
            consume_log_id: None,
            consumed_block: None,
        };

        diesel::insert_into(gift_codes::table)
            .values(&new_gift_code)
            .execute(conn)?;

        Ok(gift_code_b58)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPublic;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    // Basic test of create.
    #[test_with_logger]
    fn test_create_gift_code(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_identity = RootIdentity::from_random(&mut rng);
        let entropy = root_identity.root_entropy;

        // The Txo we would have sent to fund this entropy
        let txo_public_key: CompressedRistrettoPublic =
            RistrettoPublic::from_random(&mut rng).into();

        let memo = "Test".to_string();
        let account_id = 132;
        let build_log_id = 6873;

        let gift_code_b58 = GiftCode::create(
            entropy.as_ref(),
            &txo_public_key,
            memo,
            account_id,
            build_log_id,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        assert_eq!(gift_code_b58, "gk7CcXuK5RKNW13LvrWY156ZLjaoHaXxLedqACZsw3w6FfF6TR4TVzaAQkH5EHxaw54DnGWRJPA31PpcmvGLoArZbDRj1kBhcTusE8AVW4Mj7QT5");
    }
}
