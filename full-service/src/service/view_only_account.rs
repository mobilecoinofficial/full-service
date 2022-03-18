// Copyright (c) 2020-2022 MobileCoin Inc.

//! Service for managing view-only-accounts.

use crate::{
    db::{
        models::{ViewOnlyAccount, ViewOnlyTxo},
        view_only_account::{ViewOnlyAccountID, ViewOnlyAccountModel},
        view_only_txo::ViewOnlyTxoModel,
    },
    service::{account::AccountServiceError, WalletService},
    util::constants::DEFAULT_FIRST_BLOCK_INDEX,
};
use diesel::Connection;
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::RistrettoPrivate;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;

/// Trait defining the ways in which the wallet can interact with and manage
/// view-only accounts.
pub trait ViewOnlyAccountService {
    /// Import an existing view-only-account to the wallet using the mnemonic.
    fn import_view_only_account(
        &self,
        view_private_key: RistrettoPrivate,
        name: &str,
        first_block_index: Option<i64>,
    ) -> Result<ViewOnlyAccount, AccountServiceError>;

    /// Get a view only account by view private key
    fn get_view_only_account(
        &self,
        account_id: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError>;

    // List all view only accounts
    fn list_view_only_accounts(&self) -> Result<Vec<ViewOnlyAccount>, AccountServiceError>;

    /// Update the name for a view only account.
    fn update_view_only_account_name(
        &self,
        account_id: &str,
        name: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError>;

    /// Remove a view only account from the wallet.
    fn remove_view_only_account(&self, account_id: &str) -> Result<bool, AccountServiceError>;
}

impl<T, FPR> ViewOnlyAccountService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn import_view_only_account(
        &self,
        view_private_key: RistrettoPrivate,
        name: &str,
        first_block_index: Option<i64>,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing view-only-account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

        let account_id_hex = ViewOnlyAccountID::from(&view_private_key).to_string();

        // We record the local highest block index because that is the earliest we could
        // start scanning.
        let import_block_index: i64 = self.ledger_db.num_blocks()? as i64 - 1;

        // set first block to 0 if none provided
        let first_block_i: i64 = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX as i64);

        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| {
            Ok(ViewOnlyAccount::create(
                &account_id_hex,
                &view_private_key,
                first_block_i,
                import_block_index,
                name,
                &conn,
            )?)
        })
    }

    fn get_view_only_account(
        &self,
        account_id: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        log::info!(self.logger, "fetching view-only-account {:?}", account_id);

        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| Ok(ViewOnlyAccount::get(account_id, &conn)?))
    }

    fn list_view_only_accounts(&self) -> Result<Vec<ViewOnlyAccount>, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| Ok(ViewOnlyAccount::list_all(&conn)?))
    }

    fn update_view_only_account_name(
        &self,
        account_id: &str,
        name: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| {
            ViewOnlyAccount::get(account_id, &conn)?.update_name(name, &conn)?;
            Ok(ViewOnlyAccount::get(account_id, &conn)?)
        })
    }

    fn remove_view_only_account(&self, account_id: &str) -> Result<bool, AccountServiceError> {
        log::info!(self.logger, "Deleting view only account {}", account_id,);

        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| {
            // delete associated view-only-txos
            ViewOnlyTxo::delete_all_for_account(account_id, &conn)?;

            let account = ViewOnlyAccount::get(account_id, &conn)?;
            account.delete(&conn)?;

            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::{get_test_ledger, setup_wallet_service},
        util::encoding_helpers::ristretto_to_vec,
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_connection_test_utils::MockBlockchainConnection;
    use mc_crypto_keys::RistrettoPrivate;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::LedgerDB;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    fn get_test_service(
        logger: Logger,
        current_block_height: i64,
    ) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(
            5,
            &known_recipients,
            current_block_height as usize,
            &mut rng,
        );

        setup_wallet_service(ledger_db.clone(), logger.clone())
    }

    #[test_with_logger]
    fn service_view_only_account_crud(logger: Logger) {
        let current_block_height: i64 = 12;
        let service = get_test_service(logger, current_block_height);
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let account_id_hex = ViewOnlyAccountID::from(&view_private_key).to_string();
        let name = "coins for cats";
        let first_block_index = 25;

        // test import
        service
            .import_view_only_account(view_private_key.clone(), &name, Some(first_block_index))
            .unwrap();

        // test get
        let expected_account = ViewOnlyAccount {
            id: 1,
            account_id_hex: account_id_hex.clone(),
            view_private_key: ristretto_to_vec(&view_private_key),
            first_block_index,
            next_block_index: first_block_index,
            import_block_index: current_block_height - 1,
            name: name.to_string(),
        };

        let gotten_account = service.get_view_only_account(&account_id_hex).unwrap();

        assert_eq!(gotten_account, expected_account);

        // test update name
        let new_name = "coinzzzz";
        let updated = service
            .update_view_only_account_name(&account_id_hex, new_name)
            .unwrap();
        assert_eq!(updated.name, new_name.to_string());

        // test list all
        let view_private_key2 = RistrettoPrivate::from_random(&mut rng);
        service
            .import_view_only_account(view_private_key2, &name, Some(first_block_index))
            .unwrap();

        let all_accounts = service.list_view_only_accounts().unwrap();
        assert_eq!(all_accounts.len(), 2);

        // test remove account
        assert!(service.remove_view_only_account(&account_id_hex).unwrap());
        let not_found = service.get_view_only_account(&account_id_hex);
        assert!(not_found.is_err());
    }
}
