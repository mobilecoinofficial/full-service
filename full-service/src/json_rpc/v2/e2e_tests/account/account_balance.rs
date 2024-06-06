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
    fn test_e2e_get_balance(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            42 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let max_spendable = balance_mob["max_spendable"].as_str().unwrap();
        assert_eq!(unspent, (42 * MOB).to_string());
        assert_eq!(max_spendable, (42 * MOB - Mob::MINIMUM_FEE).to_string());
    }

    #[test_with_logger]
    fn test_balance_for_address(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let account_id = res["result"]["account"]["id"].as_str().unwrap();
        let b58_public_address = res["result"]["account"]["main_address"].as_str().unwrap();

        let alice_public_address = b58_decode_public_address(b58_public_address)
            .expect("Could not b58_decode public address");
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address],
            42 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        //
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_status",
            "params": {
                "address": b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        assert_eq!(unspent, (42 * MOB).to_string(),);
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Create a subaddress
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "comment": "For Bob",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let from_bob_b58_public_address = result
            .get("address")
            .unwrap()
            .get("public_address_b58")
            .unwrap()
            .as_str()
            .unwrap();
        let from_bob_public_address =
            b58_decode_public_address(from_bob_b58_public_address).unwrap();

        // Add a block to the ledger with a transaction "From Bob"
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![from_bob_public_address],
            64 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        //
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_status",
            "params": {
                "address": from_bob_b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, (64 * MOB).to_string());
    }
}
