// Copyright (c) 2020-2022 MobileCoin Inc.

//! Service for managing view-only Txos.

use crate::{
    db::{models::ViewOnlyTxo, transaction, view_only_txo::ViewOnlyTxoModel},
    service::txo::TxoServiceError,
    WalletService,
};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::{ring_signature::KeyImage, tx::TxOut};

/// Trait defining the ways in which the wallet can interact with and manage
/// view only Txos.
pub trait ViewOnlyTxoService {
    /// List the Txos for a given account in the wallet.
    fn list_view_only_txos(
        &self,
        account_id: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<ViewOnlyTxo>, TxoServiceError>;

    /// update the key image for a list of txos
    fn set_view_only_txos_key_images(
        &self,
        txo_ids_and_key_images: Vec<(String, KeyImage)>,
    ) -> Result<(), TxoServiceError>;

    fn list_incomplete_view_only_txos(
        &self,
        account_id: &str,
    ) -> Result<Vec<TxOut>, TxoServiceError>;
}

impl<T, FPR> ViewOnlyTxoService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_view_only_txos(
        &self,
        account_id: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<ViewOnlyTxo>, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(ViewOnlyTxo::list_for_account(
            account_id, limit, offset, &conn,
        )?)
    }

    fn set_view_only_txos_key_images(
        &self,
        txo_ids_and_key_images: Vec<(String, KeyImage)>,
    ) -> Result<(), TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;

        transaction(&conn, || {
            for (txo_id, key_image) in txo_ids_and_key_images {
                ViewOnlyTxo::update_key_image(&txo_id, &key_image, &conn)?;

                if let Some(block_index) = match self.ledger_db.check_key_image(&key_image) {
                    Ok(block_index) => block_index,
                    Err(_) => None,
                } {
                    ViewOnlyTxo::update_spent_block_index(&txo_id.to_string(), block_index, &conn)?;
                }
            }

            Ok(())
        })
    }

    fn list_incomplete_view_only_txos(
        &self,
        account_id: &str,
    ) -> Result<Vec<TxOut>, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(ViewOnlyTxo::export_txouts_without_key_image_or_subaddress_index(account_id, &conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::account::AccountID,
        service::view_only_account::ViewOnlyAccountService,
        test_utils::{add_block_to_ledger_db, get_test_ledger, setup_wallet_service, MOB},
        util::b58::b58_encode_public_address,
    };
    use mc_account_keys::{
        AccountKey, PublicAddress, CHANGE_SUBADDRESS_INDEX, DEFAULT_SUBADDRESS_INDEX,
    };
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::encrypted_fog_hint::EncryptedFogHint;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_view_only_txo_service(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let current_block_height = 12; //index 11
        let mut ledger_db = get_test_ledger(
            5,
            &known_recipients,
            current_block_height as usize,
            &mut rng,
        );
        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let conn = service.wallet_db.get_conn().unwrap();

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let spend_private_key = RistrettoPrivate::from_random(&mut rng);

        let account_key = AccountKey::new(&spend_private_key, &view_private_key);
        let account_id = AccountID::from(&account_key);
        let main_public_address = account_key.default_subaddress();
        let change_public_address = account_key.change_subaddress();
        let mut subaddresses: Vec<(String, u64, String, RistrettoPublic)> = Vec::new();
        subaddresses.push((
            b58_encode_public_address(&main_public_address).unwrap(),
            DEFAULT_SUBADDRESS_INDEX,
            "Main".to_string(),
            *main_public_address.spend_public_key(),
        ));
        subaddresses.push((
            b58_encode_public_address(&change_public_address).unwrap(),
            CHANGE_SUBADDRESS_INDEX,
            "Change".to_string(),
            *change_public_address.spend_public_key(),
        ));

        let account = service
            .import_view_only_account(
                &account_id.to_string(),
                &view_private_key,
                DEFAULT_SUBADDRESS_INDEX,
                CHANGE_SUBADDRESS_INDEX,
                2,
                "testing",
                subaddresses,
            )
            .unwrap();

        for _ in 0..2 {
            let value = 420;
            let tx_private_key = RistrettoPrivate::from_random(&mut rng);
            let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
            let fake_tx_out =
                TxOut::new(value as u64, &main_public_address, &tx_private_key, hint).unwrap();
            ViewOnlyTxo::create(
                fake_tx_out.clone(),
                value,
                Some(DEFAULT_SUBADDRESS_INDEX),
                Some(11),
                &account.account_id_hex,
                &conn,
            )
            .unwrap();
        }

        let txos = service
            .list_view_only_txos(&account.account_id_hex, None, None)
            .unwrap();

        let txo_id_1 = txos[0].txo_id_hex.clone();
        let txo_id_2 = txos[1].txo_id_hex.clone();

        for txo in &txos {
            assert_eq!(txo.key_image, None);
            assert_eq!(txo.subaddress_index, Some(DEFAULT_SUBADDRESS_INDEX as i64));
            assert_eq!(txo.received_block_index, Some(11));
            assert_eq!(txo.submitted_block_index, None);
            assert_eq!(txo.pending_tombstone_block_index, None);
            assert_eq!(txo.spent_block_index, None);
        }

        let key_image_1 = KeyImage::from(rng.next_u64());
        let key_image_2 = KeyImage::from(rng.next_u64());

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![main_public_address],
            42 * MOB,
            &vec![key_image_1],
            &mut rng,
        );

        let input_vec = [(txo_id_1, key_image_1), (txo_id_2, key_image_2)].to_vec();

        service.set_view_only_txos_key_images(input_vec).unwrap();

        let txos = service
            .list_view_only_txos(&account.account_id_hex, None, None)
            .unwrap();

        for txo in txos {
            assert!(txo.key_image.is_some());
            if txo.key_image.unwrap() == mc_util_serial::encode(&key_image_1) {
                assert_eq!(txo.spent_block_index, Some(12));
            } else {
                assert_eq!(txo.spent_block_index, None);
            }
        }
    }
}
