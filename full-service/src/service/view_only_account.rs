// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing view-only-accounts.

use crate::service::WalletService;
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;

use crate::{
    db::{
        models::ViewOnlyAccount,
        view_only_account::{ViewOnlyAccountID, ViewOnlyAccountModel},
    },
    service::account::AccountServiceError,
};
use diesel::Connection;
use mc_crypto_keys::RistrettoPrivate;
use std::convert::TryFrom;

const DEFAULT_FIRST_BLOCK_INDEX: i64 = 0;
/// Trait defining the ways in which the wallet can interact with and manage
/// view-only accounts.
pub trait ViewOnlyAccountService {
    /// Import an existing view-only-account to the wallet using the mnemonic.
    fn import_view_only_account(
        &self,
        view_private_key: Vec<u8>,
        name: String,
        first_block_index: Option<i64>,
    ) -> Result<ViewOnlyAccount, AccountServiceError>;

    /// Get a view only account by view private key
    fn get_view_only_account(
        &self,
        // TODO(cc) better to use String or &str here?
        account_id: &str,
    ) -> Result<ViewOnlyAccount, AccountServiceError>;
}

impl<T, FPR> ViewOnlyAccountService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn import_view_only_account(
        &self,
        view_private_key: Vec<u8>,
        name: String,
        first_block_index: Option<i64>,
    ) -> Result<ViewOnlyAccount, AccountServiceError> {
        log::info!(
            self.logger,
            "Importing view-only-account {:?} with first block: {:?}",
            name,
            first_block_index,
        );

        let account_id_hex =
            ViewOnlyAccountID::from(&RistrettoPrivate::try_from(view_private_key.as_slice())?)
                .to_string();

        // We record the local highest block index because that is the earliest we could
        // start scanning.
        let import_block_index: i64 = self.ledger_db.num_blocks()? as i64 - 1;

        // set first block to 0 if none provided
        let first_block_i: i64 = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX);

        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| {
            Ok(ViewOnlyAccount::create(
                &account_id_hex,
                view_private_key,
                first_block_i,
                import_block_index,
                &name,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_test_ledger, setup_wallet_service};
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_connection_test_utils::MockBlockchainConnection;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::LedgerDB;
    use rand::{rngs::StdRng, SeedableRng};

    fn get_test_service(
        logger: Logger,
        current_block_height: i64,
    ) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
        // let current_block_height: i64 = 12;
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
    fn import_view_only_account_service(logger: Logger) {
        let current_block_height: i64 = 12;
        let service = get_test_service(logger, current_block_height);

        let view_private_key: Vec<u8> = [0; 32].to_vec();
        let name = "coins for cats";
        let first_block_index = 25;

        let account_id_hex = ViewOnlyAccountID::from(
            &RistrettoPrivate::try_from(view_private_key.as_slice()).unwrap(),
        )
        .to_string();

        let view_only_account = service
            .import_view_only_account(
                view_private_key.clone(),
                name.to_string(),
                Some(first_block_index),
            )
            .unwrap();

        let expected_account = ViewOnlyAccount {
            id: 1,
            account_id_hex,
            view_private_key,
            first_block_index,
            next_block_index: first_block_index,
            import_block_index: current_block_height - 1,
            name: name.to_string(),
        };

        assert_eq!(view_only_account, expected_account);
    }

    #[test_with_logger]
    fn get_view_only_account(logger: Logger) {
        let current_block_height: i64 = 12;
        let service = get_test_service(logger, current_block_height);

        let view_private_key: Vec<u8> = [0; 32].to_vec();
        let name = "coins for cats";
        let first_block_index = 25;
        let account_id_hex = ViewOnlyAccountID::from(
            &RistrettoPrivate::try_from(view_private_key.as_slice()).unwrap(),
        )
        .to_string();

        service
            .import_view_only_account(
                view_private_key.clone(),
                name.to_string(),
                Some(first_block_index),
            )
            .unwrap();

        let expected_account = ViewOnlyAccount {
            id: 1,
            account_id_hex: account_id_hex.clone(),
            view_private_key,
            first_block_index,
            next_block_index: first_block_index,
            import_block_index: current_block_height - 1,
            name: name.to_string(),
        };

        let gotten_account = service.get_view_only_account(&account_id_hex).unwrap();

        assert_eq!(gotten_account, expected_account);
    }
}
