#[cfg(test)]
mod migration_testing {
    use crate::{
        db::{
            account::AccountID,
            migration_testing::seed_entities::{seed_accounts, seed_txos, test_txos},
            models::{Account, Txo},
            txo::{TxoID, TxoModel},
        },
        service::account::AccountService,
        test_utils::{get_test_ledger, setup_wallet_service, WalletDbTestContext},
    };
    use diesel_migrations::{revert_latest_migration, run_pending_migrations};
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_latest_migration(logger: Logger) {
        // set up wallet and service. this will run all migrations
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let db_test_context = WalletDbTestContext::default();
        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let wallet_db = &service.wallet_db;
        let conn = wallet_db.get_conn().unwrap();

        // revert the last migration
        revert_latest_migration(&conn).unwrap();

        // seed the entities
        let accounts = seed_accounts(&service);
        let txos = seed_txos(&conn, &mut ledger_db, &wallet_db, &logger, &accounts[0]);

        // validate expected txo states
        let account_id =
            AccountID::from(&mc_util_serial::decode(&accounts[0].account_key).unwrap());
        test_txos(account_id.clone(), &conn);

        // run the last migration
        run_pending_migrations(&conn).unwrap();

        // validate expected txo states again
        test_txos(account_id, &conn);

        // compare entities from seeding to entities found now
        // assert_eq!(accounts, service.list_accounts().unwrap());
        // test_entities(service, seeded);
    }
}
