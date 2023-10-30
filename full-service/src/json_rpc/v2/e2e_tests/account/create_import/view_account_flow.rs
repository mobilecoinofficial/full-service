// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::api::test_utils::{dispatch, setup},
        test_utils::{add_block_to_ledger_db, manually_sync_account, MOB},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};

    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

    #[test_with_logger]
    fn test_e2e_view_only_account_flow(logger: Logger) {
        // create normal account
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());
        let wallet_db = db_ctx.get_db_instance(logger.clone());

        // Create Account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            },
        });
        let res = dispatch(&client, body, &logger);
        assert_eq!(res.get("jsonrpc").unwrap(), "2.0");

        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        assert!(account_obj.get("id").is_some());
        assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
        let account_id = account_obj.get("id").unwrap();
        let main_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let main_account_address = b58_decode_public_address(main_address).unwrap();

        // add some funds to that account
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![main_account_address],
            100 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.as_str().unwrap().to_string()),
            &logger,
        );

        // confirm that the regular account has the correct balance
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, "100000000000000");

        // export view only import package
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_view_only_account_import_request",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        assert_eq!(res.get("jsonrpc").unwrap(), "2.0");
        let result = res.get("result").unwrap();
        let request = result.get("json_rpc_request").unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert!(result["removed"].as_bool().unwrap());

        // import vo account
        let body = json!(request);
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account = result.get("account").unwrap();
        let vo_account_id = account.get("id").unwrap();
        assert_eq!(vo_account_id, account_id);

        // sync the view only account
        manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(vo_account_id.as_str().unwrap().to_string()),
            &logger,
        );

        // confirm that the view only account has the correct balance
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": vo_account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unverified = balance_mob["unverified"].as_str().unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unverified, "100000000000000");
        assert_eq!(unspent, "0");

        let account = result.get("account").unwrap();
        let vo_account_id = account.get("id").unwrap();
        assert_eq!(vo_account_id, account_id);

        // test update name
        let name = "Look at these coins";
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "update_account_name",
            "params": {
                "account_id": vo_account_id,
                "name": name,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account = result.get("account").unwrap();
        let account_name = account.get("name").unwrap();
        assert_eq!(name, account_name);

        // test creating unsigned tx
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "build_unsigned_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": main_address,
                "amount": { "value": "50000000000000", "token_id": "0"},
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let _tx = result.get("unsigned_tx_proposal").unwrap();

        // test create sync account request
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "create_view_only_account_sync_request",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let txo_sync_request = result.get("txo_sync_request").unwrap();
        let txos_unsynced = txo_sync_request.get("txos").unwrap().as_array().unwrap();
        assert_eq!(txos_unsynced.len(), 1);

        // test remove
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": vo_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let removed = result.get("removed").unwrap().as_bool().unwrap();
        assert!(removed);

        // test get-all
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_accounts",
            "params": {}
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_ids = result.get("account_ids").unwrap().as_array().unwrap();
        assert_eq!(account_ids.len(), 0);
    }
}
