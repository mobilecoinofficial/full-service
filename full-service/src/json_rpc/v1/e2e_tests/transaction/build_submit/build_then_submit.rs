// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc,
        json_rpc::v1::api::test_utils::{dispatch, dispatch_expect_error, setup},
        test_utils::{add_block_to_ledger_db, add_block_with_tx, manually_sync_account},
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
    fn test_build_then_submit_transaction(logger: Logger) {
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

        // Add a block with a txo for this address (note that value is smaller than
        // MINIMUM_FEE, so it is a "dust" TxOut that should get opportunistically swept
        // up when we construct the transaction)
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            100,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        // Create a tx proposal to ourselves
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value_pmob": "42",
            }
        });
        // We will fail because we cannot afford the fee
        dispatch_expect_error(
            &client,
            body,
            &logger,
            json!({
                "method": "build_transaction",
                "error": json!({
                    "code": -32603,
                    "message": "InternalError",
                    "data": json!({
                        "server_error": format!("TransactionBuilder(WalletDb(InsufficientFundsUnderMaxSpendable(\"Max spendable value in wallet: 0, but target value: {}\")))", 42 + Mob::MINIMUM_FEE),
                        "details": format!("Error building transaction: Wallet DB Error: Insufficient funds from Txos under max_spendable_value: Max spendable value in wallet: 0, but target value: {}", 42 + Mob::MINIMUM_FEE),
                    })
                }),
                "jsonrpc": "2.0",
                "id": 1,
            }).to_string(),
        );

        // Add a block with significantly more MOB
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100000000000000, // 100.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Create a tx proposal to ourselves
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value_pmob": "42000000000000", // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();
        let tx = tx_proposal.get("tx").unwrap();
        let tx_prefix = tx.get("prefix").unwrap();

        // Assert the fee is correct in both places
        let prefix_fee = tx_prefix.get("fee").unwrap().as_str().unwrap();
        let fee = tx_proposal.get("fee").unwrap();
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we
        // are not losing precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee, &Mob::MINIMUM_FEE.to_string());
        assert_eq!(fee, prefix_fee);

        // Transaction builder attempts to use as many inputs as we have txos
        let inputs = tx_proposal.get("input_list").unwrap().as_array().unwrap();
        assert_eq!(inputs.len(), 2);
        let prefix_inputs = tx_prefix.get("inputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_inputs.len(), inputs.len());

        // One destination
        let outlays = tx_proposal.get("outlay_list").unwrap().as_array().unwrap();
        assert_eq!(outlays.len(), 1);

        // Map outlay -> tx_out, should have one entry for one outlay
        let outlay_index_to_tx_out_index = tx_proposal
            .get("outlay_index_to_tx_out_index")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_index_to_tx_out_index.len(), 1);

        // Two outputs in the prefix, one for change
        let prefix_outputs = tx_prefix.get("outputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_outputs.len(), 2);

        // One outlay confirmation number for our one outlay (no receipt for change)
        let outlay_confirmation_numbers = tx_proposal
            .get("outlay_confirmation_numbers")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_confirmation_numbers.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 100 default
        // tombstone)
        let prefix_tombstone = tx_prefix.get("tombstone_block").unwrap();
        assert_eq!(prefix_tombstone, "114");

        // Get current balance
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
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
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, "100000000000100");

        // Submit the tx_proposal
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": account_id,
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
        // Note - we cannot test here that the transaction ID is consistent, because
        // there is randomness in the transaction creation.

        let json_tx_proposal: json_rpc::v1::models::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 15);

        // Get balance after submission
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
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let pending = balance_status
            .get("pending_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let spent = balance_status.get("spent_pmob").unwrap().as_str().unwrap();
        let secreted = balance_status
            .get("secreted_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let orphaned = balance_status
            .get("orphaned_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, "99999600000100");
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Get the transaction_id and verify it contains what we expect
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
            "42000000000000"
        );
        assert_eq!(
            transaction_log.get("output_txos").unwrap()[0]
                .get("recipient_address_id")
                .unwrap()
                .as_str()
                .unwrap(),
            b58_public_address
        );
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
            "14"
        );
        assert_eq!(
            transaction_log
                .get("transaction_log_id")
                .unwrap()
                .as_str()
                .unwrap(),
            transaction_id
        );

        // Get All Transaction Logs
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log_ids = result
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();
        // We have a transaction log for each of the received, as well as the sent.
        assert_eq!(transaction_log_ids.len(), 5);

        // Check the contents of the transaction log associated txos
        let transaction_log_map = result.get("transaction_log_map").unwrap();
        let transaction_log = transaction_log_map.get(transaction_id).unwrap();
        assert_eq!(
            transaction_log
                .get("output_txos")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            transaction_log
                .get("input_txos")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            transaction_log
                .get("change_txos")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );

        assert_eq!(
            transaction_log.get("status").unwrap().as_str().unwrap(),
            "tx_status_succeeded"
        );

        // Get all Transaction Logs for a given Block

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_transaction_logs_ordered_by_block",
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log_map = result
            .get("transaction_log_map")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(transaction_log_map.len(), 5);
    }
}
