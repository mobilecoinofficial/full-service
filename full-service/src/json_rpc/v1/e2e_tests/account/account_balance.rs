// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::{
        db::account::AccountID,
        json_rpc::v1::api::test_utils::{dispatch, setup},
        test_utils::{add_block_to_ledger_db, manually_sync_account, MOB},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;

    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_e2e_get_balance(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            42 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
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
            "method": "get_balance_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance = result.get("balance").unwrap();
        assert_eq!(
            balance
                .get("unspent_pmob")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            (42 * MOB).to_string()
        );
        assert_eq!(
            balance
                .get("max_spendable_pmob")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            (42 * MOB - Mob::MINIMUM_FEE).to_string()
        );

        assert_eq!(
            balance["account_block_height"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            10
        );
    }

    #[test_with_logger]
    fn test_balance_for_address(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let account_id = res["result"]["account"]["account_id"].as_str().unwrap();
        let b58_public_address = res["result"]["account"]["main_address"].as_str().unwrap();

        let alice_public_address = b58_decode_public_address(&b58_public_address)
            .expect("Could not b58_decode public address");
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address],
            42 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
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
            "api_version": "2",
            "id": 1,
            "method": "get_balance_for_address",
            "params": {
                "address": b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let balance = res["result"]["balance"].clone();
        assert_eq!(
            balance["unspent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            42 * MOB
        );
        assert_eq!(
            balance["pending_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );
        assert_eq!(
            balance["spent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );
        assert_eq!(
            balance["secreted_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );
        assert_eq!(
            balance["orphaned_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );

        // Create a subaddress
        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
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
            .get("public_address")
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
            &vec![KeyImage::from(rng.next_u64())],
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
            "api_version": "2",
            "id": 1,
            "method": "get_balance_for_address",
            "params": {
                "address": from_bob_b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let balance = res["result"]["balance"].clone();
        assert_eq!(
            balance["unspent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            64 * MOB
        );
    }
}
