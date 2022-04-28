// Copyright (c) 2020-2022 MobileCoin Inc.

//! Service for managing view-only Txos.

use crate::{
    db::{models::ViewOnlyTxo, transaction, txo::TxoID, view_only_txo::ViewOnlyTxoModel},
    service::txo::TxoServiceError,
    WalletService,
};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
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

    /// set a group of txos as spent.
    fn set_view_only_txos_spent(&self, txo_ids: Vec<String>) -> Result<bool, TxoServiceError>;

    /// update the key image for a list of txos
    fn set_view_only_txos_key_images(
        &self,
        txos_with_key_images: Vec<(TxOut, KeyImage)>,
    ) -> Result<bool, TxoServiceError>;
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

    fn set_view_only_txos_spent(&self, txo_ids: Vec<String>) -> Result<bool, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;
        transaction(&conn, || {
            ViewOnlyTxo::set_spent(txo_ids, &conn)?;
            Ok(true)
        })
    }

    fn set_view_only_txos_key_images(
        &self,
        txos_with_key_images: Vec<(TxOut, KeyImage)>,
    ) -> Result<bool, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;

        transaction(&conn, || {
            for (txo, key_image) in txos_with_key_images {
                let txo_id = TxoID::from(&txo);
                ViewOnlyTxo::update_key_image(&txo_id.to_string(), &key_image, &conn)?;
            }
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::view_only_account::ViewOnlyAccountService;

    use crate::test_utils::{get_test_ledger, setup_wallet_service};
    use mc_account_keys::PublicAddress;
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
        let current_block_height = 12;
        let ledger_db = get_test_ledger(
            5,
            &known_recipients,
            current_block_height as usize,
            &mut rng,
        );
        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let conn = service.wallet_db.get_conn().unwrap();

        let view_private_key = RistrettoPrivate::from_random(&mut rng);

        let account = service
            .import_view_only_account(view_private_key.clone(), "coins for cats", None)
            .unwrap();

        for _ in 0..2 {
            let value = 420;
            let tx_private_key = RistrettoPrivate::from_random(&mut rng);
            let hint = EncryptedFogHint::fake_onetime_hint(&mut rng);
            let public_address = PublicAddress::new(
                &RistrettoPublic::from_random(&mut rng),
                &RistrettoPublic::from_random(&mut rng),
            );
            let fake_tx_out =
                TxOut::new(value as u64, &public_address, &tx_private_key, hint).unwrap();
            ViewOnlyTxo::create(fake_tx_out.clone(), value, &account.account_id_hex, &conn)
                .unwrap();
        }

        let txos = service
            .list_view_only_txos(&account.account_id_hex, None, None)
            .unwrap();

        for txo in &txos {
            assert_eq!(txo.key_image, None);
        }

        let key_image_1 = KeyImage::from(rng.next_u64());
        let key_image_2 = KeyImage::from(rng.next_u64());

        let input_vec = [
            (mc_util_serial::decode(&txos[0].txo).unwrap(), key_image_1),
            (mc_util_serial::decode(&txos[1].txo).unwrap(), key_image_2),
        ]
        .to_vec();

        service.set_view_only_txos_key_images(input_vec).unwrap();

        let txos = service
            .list_view_only_txos(&account.account_id_hex, None, None)
            .unwrap();

        for txo in txos {
            assert!(txo.key_image.is_some());
        }
    }
}
