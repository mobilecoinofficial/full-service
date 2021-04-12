// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing Txos.

use crate::{
    db::{
        account::AccountID,
        models::Txo,
        txo::{TxoDetails, TxoID, TxoModel},
        WalletDbError,
    },
    WalletService,
};
use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

/// Errors for the Txo Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TxoServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Minted Txo should contain confirmation: {0}
    MissingConfirmation(String),
}

impl From<WalletDbError> for TxoServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for TxoServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// Txos.
pub trait TxoService {
    /// List the Txos for a given account in the wallet.
    fn list_txos(
        &self,
        account_id: &AccountID,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<TxoDetails>, TxoServiceError>;

    /// Get a Txo from the wallet.
    fn get_txo(&self, txo_id: &TxoID) -> Result<TxoDetails, TxoServiceError>;

    /// List the Txos for a given address for an account in the wallet.
    fn get_all_txos_for_address(&self, address: &str) -> Result<Vec<TxoDetails>, TxoServiceError>;
}

impl<T, FPR> TxoService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_txos(
        &self,
        account_id: &AccountID,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<TxoDetails>, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(Txo::list_for_account(
            &account_id.to_string(),
            limit,
            offset,
            &conn,
        )?)
    }

    fn get_txo(&self, txo_id: &TxoID) -> Result<TxoDetails, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(Txo::get(&txo_id.to_string(), &conn)?)
    }

    fn get_all_txos_for_address(&self, address: &str) -> Result<Vec<TxoDetails>, TxoServiceError> {
        let conn = self.wallet_db.get_conn()?;

        Ok(Txo::list_for_address(address, &conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            b58_encode,
            models::{
                TXO_STATUS_PENDING, TXO_STATUS_SECRETED, TXO_STATUS_UNSPENT, TXO_TYPE_MINTED,
                TXO_TYPE_RECEIVED,
            },
        },
        service::{
            account::AccountService, balance::BalanceService, transaction::TransactionService,
        },
        test_utils::{
            add_block_to_ledger_db, get_test_ledger, setup_wallet_service, wait_for_sync, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::{
        logger::{test_with_logger, Logger},
        HashSet,
    };
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use std::iter::FromIterator;

    #[test_with_logger]
    fn test_txo_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger);
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()))
            .unwrap();

        // Add a block with a transaction for this recipient
        // Add a block with a txo for this address
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&ledger_db, &service.wallet_db, &alice_account_id, 13);

        // Verify balance for Alice
        let balance = service.get_balance_for_account(&alice_account_id).unwrap();

        assert_eq!(balance.unspent, 100 * MOB as u128);

        // Verify that we have 1 txo
        let txos = service.list_txos(&alice_account_id, None, None).unwrap();
        assert_eq!(txos.len(), 1);
        assert_eq!(
            txos[0].received_to_account.as_ref().unwrap().txo_status,
            TXO_STATUS_UNSPENT
        );

        // Add another account
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();

        // Construct a new transaction to Bob
        let bob_account_key: AccountKey = mc_util_serial::decode(&bob.account_key).unwrap();
        let tx_proposal = service
            .build_transaction(
                &alice.account_id_hex,
                &vec![(
                    b58_encode(&bob_account_key.subaddress(bob.main_subaddress_index as u64))
                        .unwrap(),
                    "42000000000000".to_string(),
                )],
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let _submitted = service
            .submit_transaction(tx_proposal, None, Some(alice.account_id_hex.clone()))
            .unwrap();

        // We should now have 3 txos - one pending, two minted (one of which will be
        // change)
        let txos = service
            .list_txos(&AccountID(alice.account_id_hex.clone()), None, None)
            .unwrap();
        assert_eq!(txos.len(), 3);
        // The Pending Tx
        let pending: Vec<TxoDetails> = txos
            .iter()
            .cloned()
            .filter(|t| {
                if let Some(txo_deets) = &t.received_to_account {
                    txo_deets.txo_status == TXO_STATUS_PENDING
                } else {
                    false
                }
            })
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].received_to_account.as_ref().unwrap().txo_type,
            TXO_TYPE_RECEIVED
        );
        assert_eq!(pending[0].txo.value, 100000000000000);
        let minted: Vec<TxoDetails> = txos
            .iter()
            .cloned()
            .filter(|t| t.minted_from_account.is_some())
            .collect();
        assert_eq!(minted.len(), 2);
        assert_eq!(
            minted[0].minted_from_account.as_ref().unwrap().txo_status,
            TXO_STATUS_SECRETED
        );
        assert_eq!(
            minted[1].minted_from_account.as_ref().unwrap().txo_type,
            TXO_TYPE_MINTED
        );
        let minted_value_set = HashSet::from_iter(minted.iter().map(|m| m.txo.value.clone()));
        assert!(minted_value_set.contains(&(57990000000000 as i64)));
        assert!(minted_value_set.contains(&(42000000000000 as i64)));

        // Our balance should reflect the various statuses of our txos
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex))
            .unwrap();
        assert_eq!(balance.unspent, 0);
        assert_eq!(balance.pending, 100000000000000);
        assert_eq!(balance.spent, 0);
        assert_eq!(balance.secreted, 99990000000000);
        assert_eq!(balance.orphaned, 0);

        // FIXME: How to make the transaction actually hit the test ledger?
    }
}
