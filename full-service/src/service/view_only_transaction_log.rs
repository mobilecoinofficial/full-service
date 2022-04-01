// Copyright (c) 2020-2022 MobileCoin Inc.

//! Service for managing view-only transaction logs.

use crate::{
    db::{
        models::ViewOnlyTransactionLog, txo::TxoID,
        view_only_transaction_log::ViewOnlyTransactionLogModel, WalletDbError,
    },
    WalletService,
};
use diesel::prelude::*;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::tx::TxOut;

/// Trait defining the ways in which the wallet can interact with and manage
/// view only transaction logs.
pub trait ViewOnlyTransactionLogService {
    /// create view only transaction logs from a transaction proposal
    fn create_view_only_transaction_logs_from_proposal(
        &self,
        transaction_proposal: TxProposal,
    ) -> Result<Vec<ViewOnlyTransactionLog>, WalletDbError>;

    fn find_all_view_only_transaction_logs_by_change_txo_id(
        &self,
        txo_id: &str,
    ) -> Result<Vec<ViewOnlyTransactionLog>, WalletDbError>;
}

impl<T, FPR> ViewOnlyTransactionLogService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn create_view_only_transaction_logs_from_proposal(
        &self,
        transaction_proposal: TxProposal,
    ) -> Result<Vec<ViewOnlyTransactionLog>, WalletDbError> {
        let conn = self.wallet_db.get_conn()?;

        let mut input_txo_ids: Vec<String> = vec![];

        // get all of the inputs for the transaction
        for utxo in transaction_proposal.utxos.iter() {
            let txo_id_hex = TxoID::from(&utxo.tx_out).to_string();
            input_txo_ids.push(txo_id_hex);
        }

        // get change txo
        if let Some(change_txo) = get_change_txout_from_proposal(&transaction_proposal) {
            let change_txo_id = TxoID::from(change_txo).to_string();

            // create a view only log for each input txo
            conn.transaction::<Vec<ViewOnlyTransactionLog>, WalletDbError, _>(|| {
                let mut logs = vec![];

                for txo_id in input_txo_ids {
                    logs.push(ViewOnlyTransactionLog::create(
                        &change_txo_id,
                        &txo_id,
                        &conn,
                    )?);
                }
                Ok(logs)
            })
        } else {
            Err(WalletDbError::UnexpectedNumberOfChangeOutputs)
        }
    }

    fn find_all_view_only_transaction_logs_by_change_txo_id(
        &self,
        txo_id: &str,
    ) -> Result<Vec<ViewOnlyTransactionLog>, WalletDbError> {
        let conn = self.wallet_db.get_conn()?;

        conn.transaction(|| ViewOnlyTransactionLog::find_all_by_change_txo_id(txo_id, &conn))
    }
}

fn get_change_txout_from_proposal(tx_proposal: &TxProposal) -> Option<&TxOut> {
    // The change TXO is the output txo that is not present as a value in the outlay
    // index to output index map.
    let change: Vec<(usize, &TxOut)> = tx_proposal
        .tx
        .prefix
        .outputs
        .iter()
        .enumerate()
        .filter(|(index, _output)| {
            !tx_proposal
                .outlay_index_to_tx_out_index
                .values()
                .any(|&value| &value == index)
        })
        .collect::<Vec<(usize, &TxOut)>>();

    // If there is none or more than one txo that meets our definition of change,
    // return None and let the function calling this handle it/throw an error
    if change.len() != 1 {
        None
    } else {
        let (_index, txo) = change[0];
        Some(txo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_account_keys::{AccountKey, PublicAddress, RootIdentity};
    use mc_common::logger::{log, test_with_logger, Logger};
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        db::{
            account::{AccountID, AccountModel},
            models::Account,
        },
        service::{sync::SyncThread, transaction_builder::WalletTransactionBuilder},
        test_utils::{
            get_resolver_factory, get_test_ledger, random_account_with_seed_values,
            setup_wallet_service, WalletDbTestContext, MOB,
        },
    };

    #[test_with_logger]
    fn test_create_view_only_transaction_logs_from_proposal(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // The account which will receive the Txo
        log::info!(logger, "Creating account");
        let root_id = RootIdentity::from_random(&mut rng);
        let recipient_account_key = AccountKey::from(&root_id);
        Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            "Alice",
            "".to_string(),
            "".to_string(),
            "".to_string(),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        // Start sync thread
        log::info!(logger, "Starting sync thread");
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        log::info!(logger, "Creating a random sender account");
        let sender_account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![10 * MOB, 10 * MOB, 10 * MOB, 10 * MOB, 10 * MOB],
            &mut rng,
            &logger,
        );

        // Create TxProposal from the sender account, which contains the Confirmation
        // Number
        log::info!(logger, "Creating transaction builder");
        let mut builder: WalletTransactionBuilder<MockFogPubkeyResolver> =
            WalletTransactionBuilder::new(
                AccountID::from(&sender_account_key).to_string(),
                wallet_db.clone(),
                ledger_db.clone(),
                get_resolver_factory(&mut rng).unwrap(),
                logger.clone(),
            );
        builder
            .add_recipient(recipient_account_key.default_subaddress(), 40 * MOB)
            .unwrap();
        builder.select_txos(None, false).unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build().unwrap();

        // find change txo from proposal
        let change_txo = get_change_txout_from_proposal(&proposal).unwrap();
        let change_txo_id = TxoID::from(change_txo).to_string();

        // create logs from proposal
        service
            .create_view_only_transaction_logs_from_proposal(proposal.clone())
            .unwrap();

        // find view only tx logs
        let found_logs = service
            .find_all_view_only_transaction_logs_by_change_txo_id(&change_txo_id)
            .unwrap();

        let input_ids: Vec<String> = proposal
            .utxos
            .iter()
            .map(|txo| TxoID::from(&txo.tx_out).to_string())
            .collect();

        // assert one log for each input
        assert_eq!(&input_ids.len(), &found_logs.len());
        for log in &found_logs {
            assert!(input_ids.iter().any(|id| id == &log.input_txo_id_hex));
            assert!(&log.change_txo_id_hex == &change_txo_id);
        }
    }
}
