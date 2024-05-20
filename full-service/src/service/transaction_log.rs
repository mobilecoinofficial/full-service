// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction logs.

use std::ops::DerefMut;

use crate::{
    db::{
        models::TransactionLog,
        transaction_log::{AssociatedTxos, TransactionId, TransactionLogModel, ValueMap},
        WalletDbError,
    },
    error::WalletServiceError,
    WalletService,
};
use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

/// Errors for the Transaction Log Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionLogServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),
}

impl From<WalletDbError> for TransactionLogServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for TransactionLogServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// transaction logs.
#[rustfmt::skip]
#[allow(clippy::result_large_err)]
pub trait TransactionLogService {
    /// List all transactions associated with the given Account ID.
    ///
    /// # Arguments
    /// 
    ///| Name              | Purpose                                                   | Notes                              |
    ///|-------------------|-----------------------------------------------------------|------------------------------------|
    ///| `account_id`      | The account id to scan for transaction logs               | Account must exist in the database |
    ///| `offset`          | The pagination offset. Results start at the offset index. | Optional, defaults to 0            |
    ///| `limit`           | Limit for the number of results.                          | Optional                           |
    ///| `min_block_index` | The minimum block index to find transaction logs from     |                                    |
    ///| `max_block_index` | The maximum block index to find transaction logs from     |                                    |
    ///
    fn list_transaction_logs(
        &self,
        account_id: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError>;

    /// Get a specific transaction log.
    ///
    /// # Arguments
    ///
    ///| Name                 | Purpose                        | Notes                                     |
    ///|----------------------|--------------------------------|-------------------------------------------|
    ///| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |
    ///
    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos, ValueMap), TransactionLogServiceError>;
}

impl<T, FPR> TransactionLogService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_transaction_logs(
        &self,
        account_id: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        Ok(TransactionLog::list_all(
            account_id,
            offset,
            limit,
            min_block_index,
            max_block_index,
            conn,
        )?)
    }

    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos, ValueMap), TransactionLogServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let transaction_log =
            TransactionLog::get(&TransactionId(transaction_id_hex.to_string()), conn)?;
        let associated = transaction_log.get_associated_txos(conn)?;
        let value_map = transaction_log.value_map(conn)?;

        Ok((transaction_log, associated, value_map))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::models::amount::Amount,
        service::{
            account::AccountService,
            address::AddressService,
            transaction::{TransactionMemo, TransactionService},
            transaction_log::TransactionLogService,
        },
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_outs, get_test_ledger, manually_sync_account,
            setup_wallet_service, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{async_test_with_logger, Logger};
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    #[async_test_with_logger]
    async fn test_list_transaction_logs_for_account_with_min_and_max_block_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(0, tx_logs.len());

        // add 5 txos to alices account
        for _ in 0..5 {
            add_block_to_ledger_db(
                &mut ledger_db,
                &vec![alice_public_address.clone()],
                100 * MOB,
                &[KeyImage::from(rng.next_u64())],
                &mut rng,
            );
        }

        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        let address = service
            .assign_address_for_account(&alice_account_id, None)
            .unwrap();

        for _ in 0..5 {
            let (_, _, _, tx_proposal) = service
                .build_sign_and_submit_transaction(
                    &alice_account_id.to_string(),
                    &[(
                        address.public_address_b58.clone(),
                        Amount::new(50 * MOB, Mob::ID),
                    )],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    TransactionMemo::RTH {
                        subaddress_index: None,
                    },
                    None,
                    None,
                )
                .await
                .unwrap();

            {
                let key_images: Vec<KeyImage> = tx_proposal
                    .input_txos
                    .iter()
                    .map(|txo| txo.key_image)
                    .collect();

                // Note: This block doesn't contain the fee output.
                add_block_with_tx_outs(
                    &mut ledger_db,
                    &[
                        tx_proposal.change_txos[0].tx_out.clone(),
                        tx_proposal.payload_txos[0].tx_out.clone(),
                    ],
                    &key_images,
                    &mut rng,
                );
            }

            manually_sync_account(
                &ledger_db,
                service.wallet_db.as_ref().unwrap(),
                &alice_account_id,
                &logger,
            );
        }

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(5, tx_logs.len());

        let tx_logs = service
            .list_transaction_logs(
                Some(alice_account_id.to_string()),
                None,
                None,
                Some(20),
                None,
            )
            .unwrap();

        assert_eq!(2, tx_logs.len());

        let tx_logs = service
            .list_transaction_logs(
                Some(alice_account_id.to_string()),
                None,
                None,
                None,
                Some(18),
            )
            .unwrap();

        assert_eq!(2, tx_logs.len());

        let tx_logs = service
            .list_transaction_logs(
                Some(alice_account_id.to_string()),
                None,
                None,
                Some(18),
                Some(20),
            )
            .unwrap();

        assert_eq!(3, tx_logs.len());
    }
}
