// #[cfg(test)]
// mod migration_testing {
//     use crate::{
//         db::{
//             account::AccountID,
//             migration_testing::{
//                 seed_accounts::{seed_accounts, test_accounts},
//                 seed_gift_codes::{seed_gift_codes, test_gift_codes},
//                 seed_txos::{seed_txos, test_txos},
//             },
//         },
//         test_utils::{get_test_ledger, setup_wallet_service,
// WalletDbTestContext},     };
//     use diesel_migrations::{revert_latest_migration, run_pending_migrations};
//     use mc_account_keys::{AccountKey, PublicAddress};
//     use mc_common::logger::{test_with_logger, Logger};
//     use rand::{rngs::StdRng, SeedableRng};

//     #[test_with_logger]
//     fn test_latest_migration(logger: Logger) {
//         // set up wallet and service. this will run all migrations
//         let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
//         let known_recipients: Vec<PublicAddress> = Vec::new();
//         let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut
// rng);         let _db_test_context = WalletDbTestContext::default();
//         let service = setup_wallet_service(ledger_db.clone(),
// logger.clone());         let wallet_db = &service.wallet_db;
//         let conn = wallet_db.get_conn().unwrap();

//         // revert the last migration
//         // revert_latest_migration(&conn).unwrap();

//         // seed the entities
//         let (txo_account, gift_code_account, gift_code_receiver_account) =
// seed_accounts(&service);         seed_txos(&conn, &mut ledger_db, &wallet_db,
// &logger, &txo_account);         let gift_codes = seed_gift_codes(
//             &conn,
//             &mut ledger_db,
//             &wallet_db,
//             &service,
//             &logger,
//             &gift_code_account,
//             &gift_code_receiver_account,
//         );

//         let account_key: AccountKey =
//             
// mc_util_serial::decode(txo_account.account_key.as_slice()).unwrap();
//         let txo_account_id = AccountID::from(&account_key);

//         // run the last migration
//         // run_pending_migrations(&conn).unwrap();

//         // validate expected state of entities in DB again, post-migration
//         test_accounts(&service);
//         test_txos(txo_account_id, &conn);
//         test_gift_codes(&gift_codes, &service);
//     }
// }
