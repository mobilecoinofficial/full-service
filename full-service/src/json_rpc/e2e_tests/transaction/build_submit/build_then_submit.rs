// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc::{
            api_test_utils::{dispatch, dispatch_expect_error, setup},
            tx_proposal::TxProposal as TxProposalJSON,
        },
        service::models::tx_proposal::TxProposal,
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_outs, add_block_with_tx_proposal,
            manually_sync_account,
        },
        util::b58::b58_decode_public_address,
    };

    use mc_blockchain_types::BlockVersion;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPrivate;
    use mc_crypto_rand::rand_core::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{
        ring_signature::KeyImage, tokens::Mob, tx::TxOut, Amount, Token, TokenId,
    };
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

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
            &vec![KeyImage::from(rng.next_u64())],
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
                "amount": { "value": "42", "token_id": "0"},
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
            &vec![public_address.clone()],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
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
                "amount": { "value": "42000000000000", "token_id": "0"}, // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        let fee = tx_proposal.get("fee").unwrap();
        let fee_token_id = tx_proposal.get("fee_token_id").unwrap();
        assert_eq!(fee, &Mob::MINIMUM_FEE.to_string());
        assert_eq!(fee_token_id, &Mob::ID.to_string());

        // Transaction builder attempts to use as many inputs as we have txos
        let inputs = tx_proposal.get("input_txos").unwrap().as_array().unwrap();
        assert_eq!(inputs.len(), 2);

        // One payload txo
        let payload_txos = tx_proposal.get("payload_txos").unwrap().as_array().unwrap();
        assert_eq!(payload_txos.len(), 1);

        let change_txos = tx_proposal.get("change_txos").unwrap().as_array().unwrap();
        assert_eq!(change_txos.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 10 default
        // tombstone)
        let tombstone_block_index = tx_proposal.get("tombstone_block_index").unwrap();
        assert_eq!(tombstone_block_index, "24");

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
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
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
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();
        // Note - we cannot test here that the transaction ID is consistent, because
        // there is randomness in the transaction creation.

        let json_tx_proposal: TxProposalJSON = serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal = TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal, &mut rng);
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
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
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
        let value_map = transaction_log.get("value_map").unwrap();

        let pmob_value = value_map.get("0").unwrap();
        assert_eq!(pmob_value.as_str().unwrap(), "42000000000000");

        assert_eq!(
            transaction_log.get("output_txos").unwrap()[0]
                .get("recipient_public_address_b58")
                .unwrap()
                .as_str()
                .unwrap(),
            b58_public_address
        );
        transaction_log.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            transaction_log.get("fee_value").unwrap().as_str().unwrap(),
            &Mob::MINIMUM_FEE.to_string()
        );
        assert_eq!(
            transaction_log
                .get("fee_token_id")
                .unwrap()
                .as_str()
                .unwrap(),
            Mob::ID.to_string()
        );
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
            "14"
        );
        assert_eq!(
            transaction_log.get("id").unwrap().as_str().unwrap(),
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
        // We have a transaction log for the sent
        assert_eq!(transaction_log_ids.len(), 1);

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
            "succeeded"
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
        assert_eq!(transaction_log_map.len(), 1);

        let txo = TxOut::new(
            BlockVersion::MAX,
            Amount::new(1000000000000, TokenId::from(1)),
            &public_address,
            &RistrettoPrivate::from_random(&mut rng),
            Default::default(),
        )
        .unwrap();

        add_block_with_tx_outs(
            &mut ledger_db,
            &[txo],
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // Create a tx proposal to ourselves, but this should fail because we cannot yet
        // do mixed token transactions
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "amount": { "value": "500000000000", "token_id": "1" },
                "fee_token_id": "0",
            }
        });
        let res = dispatch(&client, body, &logger);
        let err = res.get("error");
        assert!(err.is_some());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "amount": { "value": "500000000000", "token_id": "1" }
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        let fee_token_id = tx_proposal.get("fee_token_id").unwrap();
        assert_eq!(fee_token_id, "1");

        let inputs = tx_proposal.get("input_txos").unwrap().as_array().unwrap();
        assert_eq!(inputs.len(), 1);

        // One destination
        let payload_txos = tx_proposal.get("payload_txos").unwrap().as_array().unwrap();
        assert_eq!(payload_txos.len(), 1);

        let change_txos = tx_proposal.get("change_txos").unwrap().as_array().unwrap();
        assert_eq!(change_txos.len(), 1);

        // Tombstone block = ledger height (12 to start + 4 new blocks + 10 default
        // tombstone)
        let tombstone_block_index = tx_proposal.get("tombstone_block_index").unwrap();
        assert_eq!(tombstone_block_index, "26");

        // Get current balance
        assert_eq!(ledger_db.num_blocks().unwrap(), 16);
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
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_1 = balance_per_token.get("1").unwrap();
        let unspent = balance_1["unspent"].as_str().unwrap();
        assert_eq!(unspent, "1000000000000");

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
        let _result = res.get("result").unwrap();
        let json_tx_proposal: TxProposalJSON = serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal = TxProposal::try_from(&json_tx_proposal).unwrap();

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
        let balance_per_token = result.get("balance_per_token").unwrap();

        // Balance of MOB should be unchanged
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        assert_eq!(unspent, &(100000000000100 - Mob::MINIMUM_FEE).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // There should be a pending balance for this token now
        let balance_1 = balance_per_token.get("1").unwrap();
        let unspent = balance_1["unspent"].as_str().unwrap();
        let pending = balance_1["pending"].as_str().unwrap();
        let spent = balance_1["spent"].as_str().unwrap();
        let secreted = balance_1["secreted"].as_str().unwrap();
        let orphaned = balance_1["orphaned"].as_str().unwrap();
        assert_eq!(unspent, "0");
        assert_eq!(pending, "1000000000000");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal, &mut rng);

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 17);

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
        let balance_per_token = result.get("balance_per_token").unwrap();

        // Balance of MOB should be unchanged
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        assert_eq!(unspent, &(100000000000100 - Mob::MINIMUM_FEE).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        let balance_1 = balance_per_token.get("1").unwrap();
        let unspent = balance_1["unspent"].as_str().unwrap();
        let pending = balance_1["pending"].as_str().unwrap();
        let spent = balance_1["spent"].as_str().unwrap();
        let secreted = balance_1["secreted"].as_str().unwrap();
        let orphaned = balance_1["orphaned"].as_str().unwrap();
        assert_eq!(unspent, "999999998976".to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "1000000000000");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");
    }
}
