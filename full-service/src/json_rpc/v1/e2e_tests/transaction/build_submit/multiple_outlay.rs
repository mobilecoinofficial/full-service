// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc,
        json_rpc::v1::api::test_utils::{dispatch, setup},
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
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
        let alice_account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
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
        let bob_account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
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
        let charlie_account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
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
                "addresses_and_values": [
                    [bob_b58_public_address, "42000000000000"], // 42.0 MOB
                    [charlie_b58_public_address, "43000000000000"], // 43.0 MOB
                ]
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();

        let tx_proposal = result.get("tx_proposal").unwrap();
        let tx = tx_proposal.get("tx").unwrap();
        let tx_prefix = tx.get("prefix").unwrap();
        let transaction_log_id = result.get("transaction_log_id").unwrap();

        // Assert the fee is correct in both places
        let prefix_fee = tx_prefix.get("fee").unwrap().as_str().unwrap();
        let fee = tx_proposal.get("fee").unwrap();
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we
        // are not losing precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee, &Mob::MINIMUM_FEE.to_string());
        assert_eq!(fee, prefix_fee);

        // Two destinations.
        let outlays = tx_proposal.get("outlay_list").unwrap().as_array().unwrap();
        assert_eq!(outlays.len(), 2);

        // Map outlay -> tx_out, should have one entry for one outlay
        let outlay_index_to_tx_out_index = tx_proposal
            .get("outlay_index_to_tx_out_index")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_index_to_tx_out_index.len(), 2);

        // Three outputs in the prefix, one for change
        let prefix_outputs = tx_prefix.get("outputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_outputs.len(), 3);

        // Two outlay confirmation numbers for our two outlays (no receipt for change)
        let outlay_confirmation_numbers = tx_proposal
            .get("outlay_confirmation_numbers")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_confirmation_numbers.len(), 2);

        // Get balances before submitting.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let alice_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(alice_unspent, "100000000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": bob_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let bob_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(bob_unspent, "0");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": charlie_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let charlie_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(charlie_unspent, "0");

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
            .get("transaction_log_id")
            .unwrap()
            .as_str()
            .unwrap();

        assert_eq!(transaction_log_id, transaction_id);
        let json_tx_proposal: json_rpc::v1::models::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

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
            "method": "get_balance_for_account",
            "params": {
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, &(15 * MOB - Mob::MINIMUM_FEE).to_string());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": bob_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let bob_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(bob_unspent, "42000000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": charlie_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let charlie_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(charlie_unspent, "43000000000000");

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
        assert_eq!(
            transaction_log.get("direction").unwrap().as_str().unwrap(),
            "tx_direction_sent"
        );
        assert_eq!(
            transaction_log.get("value_pmob").unwrap().as_str().unwrap(),
            "85000000000000"
        );

        let mut output_addresses: Vec<String> = transaction_log
            .get("output_txos")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                t.get("recipient_address_id")
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
        assert_eq!(
            transaction_log.get("fee_pmob").unwrap().as_str().unwrap(),
            &Mob::MINIMUM_FEE.to_string()
        );
        assert_eq!(
            transaction_log.get("status").unwrap().as_str().unwrap(),
            "tx_status_succeeded"
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
            transaction_log
                .get("transaction_log_id")
                .unwrap()
                .as_str()
                .unwrap(),
            transaction_id
        );
    }
}
