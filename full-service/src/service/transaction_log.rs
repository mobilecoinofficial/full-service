// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction logs.

use crate::{
    db::{
        account::AccountID,
        models::TransactionLog,
        transaction_log::{AssociatedTxos, TransactionLogModel},
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
        account_id: &AccountID,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;

    /// Get a specific transaction log.
    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos), TransactionLogServiceError>;

    /// Get all transaction logs for a given block.
    fn get_all_transaction_logs_for_block(
        &self,
        block_index: u64,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;

    /// Get all transaction logs ordered by finalized_block_index.
    fn get_all_transaction_logs_ordered_by_block(
        &self,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError>;
}

impl<T, FPR> TransactionLogService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn list_transaction_logs(
        &self,
        account_id: &AccountID,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        let conn = &self.wallet_db.get_conn()?;
        Ok(TransactionLog::list_all(
            &account_id.to_string(),
            offset,
            limit,
            conn,
        )?)
    }

    fn get_transaction_log(
        &self,
        transaction_id_hex: &str,
    ) -> Result<(TransactionLog, AssociatedTxos), TransactionLogServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction_log = TransactionLog::get(transaction_id_hex, &conn)?;
        let associated = transaction_log.get_associated_txos(&conn)?;

        Ok((transaction_log, associated))
    }

    fn get_all_transaction_logs_for_block(
        &self,
        block_index: u64,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction_logs = TransactionLog::get_all_for_block_index(block_index, &conn)?;
        let mut res: Vec<(TransactionLog, AssociatedTxos)> = Vec::new();
        for transaction_log in transaction_logs {
            res.push((
                transaction_log.clone(),
                transaction_log.get_associated_txos(&conn)?,
            ));
        }
        Ok(res)
    }

    fn get_all_transaction_logs_ordered_by_block(
        &self,
    ) -> Result<Vec<(TransactionLog, AssociatedTxos)>, WalletServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let transaction_logs = TransactionLog::get_all_ordered_by_block_index(&conn)?;
        let mut res: Vec<(TransactionLog, AssociatedTxos)> = Vec::new();
        for transaction_log in transaction_logs {
            res.push((
                transaction_log.clone(),
                transaction_log.get_associated_txos(&conn)?,
            ));
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::account::AccountID,
        service::{account::AccountService, transaction_log::TransactionLogService},
        test_utils::{
            add_block_to_ledger_db, get_test_ledger, manually_sync_account, setup_wallet_service,
            MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;
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
            .list_transaction_logs(&alice_account_id, None, None, None, None)
            .unwrap();

        assert_eq!(0, tx_logs.len());

        // block_index 12
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // block_index 13
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // block_index 14
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // block_index 15
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // block_index 16
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(&ledger_db, &service.wallet_db, &alice_account_id, &logger);

        let tx_logs = service
            .list_transaction_logs(&alice_account_id, None, None, None, None)
            .unwrap();

        assert_eq!(5, tx_logs.len());

        let tx_logs = service
            .list_transaction_logs(&alice_account_id, None, None, Some(15), None)
            .unwrap();

        assert_eq!(2, tx_logs.len());

        let tx_logs = service
            .list_transaction_logs(&alice_account_id, None, None, None, Some(13))
            .unwrap();

        assert_eq!(2, tx_logs.len());

        let tx_logs = service
            .list_transaction_logs(&alice_account_id, None, None, Some(13), Some(15))
            .unwrap();

        assert_eq!(3, tx_logs.len());
    }
}
