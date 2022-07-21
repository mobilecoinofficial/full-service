// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc,
        json_rpc::v1::api::test_utils::{dispatch, setup},
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_proposal, manually_sync_account, MOB,
        },
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};

    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_tx_status_failed_when_tombstone_block_index_exceeded(logger: Logger) {
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

        // Create a tx proposal to ourselves with a tombstone block of 1
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value_pmob": "42000000000000", // 42.0 MOB
                "tombstone_block": "16",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_log = result.get("transaction_log").unwrap();
        let tx_log_status = tx_log.get("status").unwrap();
        let tx_log_id = tx_log.get("transaction_log_id").unwrap();

        assert_eq!(tx_log_status, "tx_status_pending");

        // Add a block with 1 MOB
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            1, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

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
        assert_eq!(unspent, "1");
        assert_eq!(pending, "100000000000100");

        // Add a block with 1 MOB to increment height 2 times,
        // which should cause the previous transaction to
        // become invalid and free up the TXO as well as mark
        // the transaction log as TX_STATUS_FAILED
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            1, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            1, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        assert_eq!(ledger_db.num_blocks().unwrap(), 17);

        // Get tx log after syncing is finished
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_log",
            "params": {
                "transaction_log_id": tx_log_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_log = result.get("transaction_log").unwrap();
        let tx_log_status = tx_log.get("status").unwrap();

        assert_eq!(tx_log_status, "tx_status_failed");

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
        assert_eq!(unspent, "100000000000103".to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
    }

    #[test_with_logger]
    fn test_paginate_transactions(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add some transactions.
        for _ in 0..10 {
            add_block_to_ledger_db(
                &mut ledger_db,
                &vec![public_address.clone()],
                100,
                &vec![KeyImage::from(rng.next_u64())],
                &mut rng,
            );
        }

        assert_eq!(ledger_db.num_blocks().unwrap(), 22);
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // Check that we can paginate txo output.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txos_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let txos_all = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos_all.len(), 10);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txos_for_account",
            "params": {
                "account_id": account_id,
                "offset": "2",
                "limit": "5",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let txos_page = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos_page.len(), 5);
        assert_eq!(txos_all[2..7].len(), 5);
        assert_eq!(txos_page[..], txos_all[2..7]);

        // Check that we can paginate transaction log output.
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
        let tx_logs_all = result
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(tx_logs_all.len(), 10);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
                "offset": "3",
                "limit": "6",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_logs_page = result
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(tx_logs_page.len(), 6);
        assert_eq!(tx_logs_all[3..9].len(), 6);
        assert_eq!(tx_logs_page[..], tx_logs_all[3..9]);
    }

    #[test_with_logger]
    fn test_receipts(logger: Logger) {
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
        let alice_account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let alice_b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let alice_public_address = b58_decode_public_address(alice_b58_public_address).unwrap();

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            &logger,
        );

        // Add Bob's account to our wallet
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
        let bob_account_obj = result.get("account").unwrap();
        let bob_account_id = bob_account_obj.get("account_id").unwrap().as_str().unwrap();
        let bob_b58_public_address = bob_account_obj
            .get("main_address")
            .unwrap()
            .as_str()
            .unwrap();

        // Construct a transaction proposal from Alice to Bob
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": alice_account_id,
                "recipient_public_address": bob_b58_public_address,
                "value_pmob": "42000000000000", // 42 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        // Get the receipts from the tx_proposal
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_receiver_receipts",
            "params": {
                "tx_proposal": tx_proposal
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let receipts = result["receiver_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 1);
        let receipt = &receipts[0];

        // Bob checks status (should be pending before the block is added to the ledger)
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check_receiver_receipt_status",
            "params": {
                "address": bob_b58_public_address,
                "receiver_receipt": receipt,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let status = result["receipt_transaction_status"].as_str().unwrap();
        assert_eq!(status, "TransactionPending");

        // Add the block to the ledger with the tx proposal
        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);

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

        // Bob checks status (should be successful after added to the ledger)
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check_receiver_receipt_status",
            "params": {
                "address": bob_b58_public_address,
                "receiver_receipt": receipt,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let status = result["receipt_transaction_status"].as_str().unwrap();
        assert_eq!(status, "TransactionSuccess");
    }
}
