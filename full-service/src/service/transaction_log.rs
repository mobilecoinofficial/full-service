// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction logs.

use crate::{
    db::{
        models::TransactionLog,
        transaction_log::{AssociatedTxos, TransactionID, TransactionLogModel, ValueMap},
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
pub trait TransactionLogService {
    /// List all transactions associated with the given Account ID.
    fn list_transaction_logs(
        &self,
        account_id: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
        min_block_index: Option<u64>,
        max_block_index: Option<u64>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError>;

    /// Get a specific transaction log.
    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos, ValueMap), TransactionLogServiceError>;

    /// Get all transaction logs for a given block.
    fn get_all_transaction_logs_for_block(
        &self,
        block_index: u64,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError>;

    /// Get all transaction logs ordered& by finalized_block_index.
    fn get_all_transaction_logs_ordered_by_block(
        &self,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError>;
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
        let conn = &self.wallet_db.get_conn()?;
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
        let conn = self.wallet_db.get_conn()?;
        let transaction_log =
            TransactionLog::get(&TransactionID(transaction_id_hex.to_string()), &conn)?;
        let associated = transaction_log.get_associated_txos(&conn)?;
        let value_map = transaction_log.value_map(&conn)?;

        Ok((transaction_log, associated, value_map))
    }

    fn get_all_transaction_logs_for_block(
        &self,
        block_index: u64,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction_logs = TransactionLog::get_all_for_block_index(block_index, &conn)?;
        let mut res: Vec<(TransactionLog, AssociatedTxos, ValueMap)> = Vec::new();
        for transaction_log in transaction_logs {
            res.push((
                transaction_log.clone(),
                transaction_log.get_associated_txos(&conn)?,
                transaction_log.value_map(&conn)?,
            ));
        }
        Ok(res)
    }

    fn get_all_transaction_logs_ordered_by_block(
        &self,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos, ValueMap)>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction_logs = TransactionLog::get_all_ordered_by_block_index(&conn)?;
        let mut res: Vec<(TransactionLog, AssociatedTxos, ValueMap)> = Vec::new();
        for transaction_log in transaction_logs {
            res.push((
                transaction_log.clone(),
                transaction_log.get_associated_txos(&conn)?,
                transaction_log.value_map(&conn)?,
            ));
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::account::AccountID,
        service::{
            account::AccountService, address::AddressService, transaction::TransactionService,
            transaction_log::TransactionLogService,
        },
        test_utils::{
            add_block_from_transaction_log, add_block_to_ledger_db, get_test_ledger,
            manually_sync_account, setup_wallet_service, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_list_transaction_logs_for_account_with_min_and_max_block_index(logger: Logger) {
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
                "".to_string(),
            )
            .unwrap();

        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);

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
                &vec![KeyImage::from(rng.next_u64())],
                &mut rng,
            );
        }

        manually_sync_account(&ledger_db, &service.wallet_db, &alice_account_id, &logger);

        let address = service
            .assign_address_for_account(&alice_account_id, None)
            .unwrap();

        for _ in 0..5 {
            let (transaction_log, _, _, _) = service
                .build_and_submit(
                    &alice_account_id.to_string(),
                    &[(
                        address.assigned_subaddress_b58.clone(),
                        (50 * MOB).to_string(),
                        Mob::ID.to_string(),
                    )],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();

            {
                let conn = service.wallet_db.get_conn().unwrap();
                add_block_from_transaction_log(&mut ledger_db, &conn, &transaction_log, &mut rng);
            }

            manually_sync_account(&ledger_db, &service.wallet_db, &alice_account_id, &logger);
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
