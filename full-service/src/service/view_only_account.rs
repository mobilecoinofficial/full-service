// Copyright (c) 2020-2022 MobileCoin Inc.

//! Service for managing view-only-accounts.

use crate::{
    db::{
        models::{ViewOnlyAccount, ViewOnlySubaddress},
        transaction,
        view_only_account::ViewOnlyAccountModel,
        view_only_subaddress::ViewOnlySubaddressModel,
    },
    service::{account::AccountServiceError, WalletService},
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_fog_report_validation::FogPubkeyResolver;

/// Trait defining the ways in which the wallet can interact with and manage
/// view-only accounts.
pub trait ViewOnlyAccountService {
    /// Import an existing view-only-account to the wallet using the mnemonic.
    #[allow(clippy::too_many_arguments)]
    fn import_view_only_account(
        &self,
        account_id_hex: &str,
        view_private_key: &RistrettoPrivate,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        name: &str,
        subaddresses: Vec<(String, u64, String, RistrettoPublic)>,
    ) -> Result<ViewOnlyAccount, AccountServiceError>;

    fn import_subaddresses(
        &self,
        account_id_hex: &str,
        subaddresses: Vec<(String, u64, String, RistrettoPublic)>,
    ) -> Result<Vec<String>, AccountServiceError>;

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
        account_id_hex: &str,
        view_private_key: &RistrettoPrivate,
        main_subaddress_index: u64,
        change_subaddress_index: u64,
        next_subaddress_index: u64,
        name: &str,
        subaddresses: Vec<(String, u64, String, RistrettoPublic)>,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        let conn = &self.wallet_db.get_conn()?;

        transaction(conn, || {
            let view_only_account = ViewOnlyAccount::create(
                account_id_hex,
                view_private_key,
                0,
                0,
                main_subaddress_index,
                change_subaddress_index,
                next_subaddress_index,
                name,
                conn,
            )?;

            for (public_address_b58, subaddress_index, comment, public_spend_key) in
                subaddresses.iter()
            {
                ViewOnlySubaddress::create(
                    &view_only_account,
                    public_address_b58,
                    *subaddress_index,
                    comment,
                    public_spend_key,
                    conn,
                )?;
            }

            Ok(view_only_account)
        })
    }

    fn import_subaddresses(
        &self,
        account_id_hex: &str,
        subaddresses: Vec<(String, u64, String, RistrettoPublic)>,
    ) -> Result<Vec<String>, AccountServiceError> {
        let conn = &self.wallet_db.get_conn()?;

        transaction(conn, || {
            let account = ViewOnlyAccount::get(account_id_hex, conn)?;

            for (public_address_b58, subaddress_index, comment, public_spend_key) in
                subaddresses.iter()
            {
                ViewOnlySubaddress::create(
                    &account,
                    public_address_b58,
                    *subaddress_index,
                    comment,
                    public_spend_key,
                    conn,
                )?;
            }

            let next_subaddress_index = subaddresses
                .iter()
                .map(|(_, index, _, _)| *index)
                .max()
                .unwrap_or(0);

            if next_subaddress_index > account.next_subaddress_index as u64 {
                account.update_next_subaddress_index(next_subaddress_index, conn)?;
            }

            Ok(subaddresses
                .iter()
                .map(|(public_address_b58, _, _, _)| public_address_b58.clone())
                .collect())
        })
    }

    fn get_view_only_account(
        &self,
        account_id: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        log::info!(self.logger, "fetching view-only-account {:?}", account_id);

        let conn = self.wallet_db.get_conn()?;
        Ok(ViewOnlyAccount::get(account_id, &conn)?)
    }

    fn list_view_only_accounts(&self) -> Result<Vec<ViewOnlyAccount>, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(ViewOnlyAccount::list_all(&conn)?)
    }

    fn update_view_only_account_name(
        &self,
        account_id: &str,
        name: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        let conn = self.wallet_db.get_conn()?;
        ViewOnlyAccount::get(account_id, &conn)?.update_name(name, &conn)?;
        Ok(ViewOnlyAccount::get(account_id, &conn)?)
    }

    fn remove_view_only_account(&self, account_id: &str) -> Result<bool, AccountServiceError> {
        log::info!(self.logger, "Deleting view only account {}", account_id,);

        let conn = self.wallet_db.get_conn()?;
        let account = ViewOnlyAccount::get(account_id, &conn)?;
        transaction(&conn, || {
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
        current_block_height: u64,
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
        let current_block_height = 12;
        let service = get_test_service(logger, current_block_height);
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let account_id_hex = AccountID::from(&view_private_key).to_string();
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
            first_block_index: first_block_index as i64,
            next_block_index: first_block_index as i64,
            import_block_index: (current_block_height - 1 + 1) as i64,
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
