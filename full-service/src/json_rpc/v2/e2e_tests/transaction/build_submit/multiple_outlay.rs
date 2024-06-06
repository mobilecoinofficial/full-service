// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::{
            api::test_utils::{dispatch, setup},
            models::{amount::Amount, tx_proposal::TxProposal as TxProposalJSON},
        },
        service::models::tx_proposal::TxProposal,
        test_utils::{add_block_to_ledger_db, add_block_with_tx, manually_sync_account, MOB},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};

    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_multiple_outlay_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

        // Add some accounts.
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
        let alice_account_id = account_obj.get("id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let alice_public_address = b58_decode_public_address(b58_public_address).unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Bob Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let bob_account_id = account_obj.get("id").unwrap().as_str().unwrap();
        let bob_b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Charlie Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let charlie_account_id = account_obj.get("id").unwrap().as_str().unwrap();
        let charlie_b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();

        // Add some money to Alice's account.
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address],
            100000000000000, // 100.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            &logger,
        );

        // Create a two-output tx proposal to Bob and Charlie.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": alice_account_id,
                "addresses_and_amounts": [
                    [bob_b58_public_address, {"value": "42000000000000", "token_id": "0"}], // 42.0 MOB
                    [charlie_b58_public_address, {"value": "43000000000000", "token_id": "0"}], // 43.0 MOB
                ]
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();

        let tx_proposal = result.get("tx_proposal").unwrap();

        let fee_amount: Amount =
            serde_json::from_value(tx_proposal.get("fee_amount").unwrap().clone()).unwrap();

        assert_eq!(fee_amount, Amount::new(Mob::MINIMUM_FEE, Mob::ID));

        // Two destinations.
        let payload_txos = tx_proposal.get("payload_txos").unwrap().as_array().unwrap();
        assert_eq!(payload_txos.len(), 2);

        let change_txos = tx_proposal.get("change_txos").unwrap().as_array().unwrap();
        assert_eq!(change_txos.len(), 1);

        // Get balances before submitting.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, "100000000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": bob_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string());
        assert!(balance_mob.is_none());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": charlie_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string());
        assert!(balance_mob.is_none());

        // Submit the tx_proposal
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_id = result
            .get("transaction_log")
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();

        let json_tx_proposal: TxProposalJSON = serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal = TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Wait for accounts to sync.
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            &logger,
        );
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(bob_account_id.to_string()),
            &logger,
        );
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(charlie_account_id.to_string()),
            &logger,
        );

        // Get balances after submission
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, &(15 * MOB - Mob::MINIMUM_FEE).to_string());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": bob_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, "42000000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_account_status",
            "params": {
                "account_id": charlie_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, "43000000000000");

        // Get the transaction log and verify it contains what we expect
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_log",
            "params": {
                "transaction_log_id": transaction_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log = result.get("transaction_log").unwrap();

        let value_map = transaction_log.get("value_map").unwrap();

        let pmob_value = value_map.get("0").unwrap();
        assert_eq!(pmob_value.as_str().unwrap(), "85000000000000");

        let mut output_addresses: Vec<String> = transaction_log
            .get("output_txos")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                t.get("recipient_public_address_b58")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .into()
            })
            .collect();
        output_addresses.sort();
        let mut target_addresses = vec![bob_b58_public_address, charlie_b58_public_address];
        target_addresses.sort();
        assert_eq!(output_addresses, target_addresses);

        transaction_log.get("account_id").unwrap().as_str().unwrap();
        let fee_amount = transaction_log.get("fee_amount").unwrap();
        let fee_value = fee_amount.get("value").unwrap().as_str().unwrap();
        let fee_token_id = fee_amount.get("token_id").unwrap().as_str().unwrap();
        assert_eq!(fee_value, &Mob::MINIMUM_FEE.to_string());
        assert_eq!(fee_token_id, &Mob::ID.to_string());
        assert_eq!(
            transaction_log.get("status").unwrap().as_str().unwrap(),
            "succeeded"
        );
        assert_eq!(
            transaction_log
                .get("submitted_block_index")
                .unwrap()
                .as_str()
                .unwrap(),
            "13"
        );
        assert_eq!(
            transaction_log.get("id").unwrap().as_str().unwrap(),
            transaction_id
        );
    }
}
