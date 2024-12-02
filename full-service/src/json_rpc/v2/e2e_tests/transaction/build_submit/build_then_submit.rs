// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::{account::AccountID, transaction_log::TxStatus},
        json_rpc::v2::{
            api::test_utils::{dispatch, dispatch_expect_error, setup},
            models::{
                account::Account, amount::Amount as AmountJSON, transaction_log::TransactionLog,
                tx_proposal::TxProposal as TxProposalJSON,
            },
        },
        service::models::tx_proposal::TxProposal,
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx, add_block_with_tx_outs,
            manually_sync_account,
        },
        util::b58::b58_decode_public_address,
    };

    use mc_blockchain_types::BlockVersion;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPrivate;
    use mc_ledger_db::Ledger;
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{
        ring_signature::KeyImage, tokens::Mob, tx::TxOut, Amount, Token, TokenId,
    };
    use mc_util_from_random::FromRandom;

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
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
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
                "amount": { "value": "42000000000000", "token_id": "0"}, // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal: TxProposalJSON =
            serde_json::from_value(result.get("tx_proposal").unwrap().clone()).unwrap();

        assert_eq!(
            tx_proposal.fee_amount,
            AmountJSON::new(Mob::MINIMUM_FEE, Mob::ID)
        );

        // Transaction builder attempts to use as many inputs as we have txos
        assert_eq!(tx_proposal.input_txos.len(), 2);

        // One payload txo
        assert_eq!(tx_proposal.payload_txos.len(), 1);

        assert_eq!(tx_proposal.change_txos.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 100 default
        // tombstone)
        assert_eq!(tx_proposal.tombstone_block_index, "114");

        // Get current balance
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
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

        let payments_tx_proposal = TxProposal::try_from(&tx_proposal).unwrap();

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
        let transaction_log: TransactionLog =
            serde_json::from_value(result.get("transaction_log").unwrap().clone()).unwrap();

        let pmob_value = transaction_log.value_map.get(&Mob::ID.to_string()).unwrap();
        assert_eq!(pmob_value, "42000000000000");

        assert_eq!(
            transaction_log.output_txos[0].recipient_public_address_b58,
            b58_public_address
        );

        assert_eq!(
            transaction_log.fee_amount,
            AmountJSON::new(Mob::MINIMUM_FEE, Mob::ID)
        );

        assert_eq!(transaction_log.status, TxStatus::Succeeded.to_string());
        assert_eq!(transaction_log.submitted_block_index.unwrap(), "14");
        assert_eq!(transaction_log.id, transaction_id);

        // Get All Transaction Logs
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_logs",
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
            "method": "get_transaction_logs",
            "params": {
                "min_block_index": "14",
                "max_block_index": "14",
            }
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
            &[KeyImage::from(rng.next_u64())],
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
        let tx_proposal: TxProposalJSON =
            serde_json::from_value(result.get("tx_proposal").unwrap().clone()).unwrap();

        assert_eq!(
            tx_proposal.fee_amount.token_id.expose_secret().to_string(),
            TokenId::from(1).to_string()
        );

        assert_eq!(tx_proposal.input_txos.len(), 1);

        assert_eq!(tx_proposal.payload_txos.len(), 1);

        assert_eq!(tx_proposal.change_txos.len(), 1);

        // Tombstone block = ledger height (12 to start + 4 new blocks + 100 default
        // tombstone)
        assert_eq!(tx_proposal.tombstone_block_index, "116");

        // Get current balance
        assert_eq!(ledger_db.num_blocks().unwrap(), 16);
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
        let payments_tx_proposal = TxProposal::try_from(&tx_proposal).unwrap();

        // Get balance after submission
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
        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);

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
            "method": "get_account_status",
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

    #[test_with_logger]
    fn test_build_then_submit_transaction_multiple_accounts(logger: Logger) {
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
        let alice_account_id = account_obj.get("id").unwrap().as_str().unwrap();
        let alice_b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let alice_public_address = b58_decode_public_address(alice_b58_public_address).unwrap();

        // Add a second account
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

        // Add a block with a txo for this address (note that value is smaller than
        // MINIMUM_FEE, so it is a "dust" TxOut that should get opportunistically swept
        // up when we construct the transaction)
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![
                alice_public_address.clone(),
                alice_public_address.clone(),
                alice_public_address.clone(),
                alice_public_address.clone(),
                alice_public_address,
            ],
            100000000000000,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            &logger,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": alice_account_id,
                "recipient_public_address": bob_b58_public_address,
                "amount": { "value": "42000000000000", "token_id": "0"}, // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal: TxProposalJSON =
            serde_json::from_value(result.get("tx_proposal").unwrap().clone()).unwrap();

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
        assert_eq!(unspent, "500000000000000");

        // Submit the tx_proposal
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": alice_account_id,
            }
        });
        let _res = dispatch(&client, body, &logger);

        let payments_tx_proposal = TxProposal::try_from(&tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);
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

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": bob_account_id,
                "recipient_public_address": bob_b58_public_address,
                "amount": { "value": "10000000000000", "token_id": "0"}, // 42.0 MOB
            }
        });
        let _res = dispatch(&client, body, &logger);

        // Get balance after submission
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
        let max_spendable = balance_mob["max_spendable"].as_str().unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        assert_eq!(max_spendable, "457999200000000");
        assert_eq!(unspent, "457999600000000");
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000000");
        assert_eq!(secreted, "42000000000000");
        assert_eq!(orphaned, "0");

        // Get balance after submission
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
        let max_spendable = balance_mob["max_spendable"].as_str().unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        assert_eq!(
            max_spendable,
            (42000000000000 - Mob::MINIMUM_FEE).to_string()
        );
        assert_eq!(unspent, "42000000000000");
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");
    }

    #[test_with_logger]
    fn test_reuse_input_txo_then_submit_transaction(logger: Logger) {
        // This test mimics a situation where multiple services are using the same
        // Full-Service instance. Both services are able to `build_transaction`s,
        // which consume the same input txo(s). Both of the services are able to submit
        // the transactions prior to either transaction landing on the blockchain. The
        // blockchain will only allow one of these transactions to succeed.
        // Previous versions of Full-Service had a bug that would mark both transactions
        // as succeeded. This was because Full-Service would mark a transaction as
        // succeeded if the inputs were consumed. This test ensures that this
        // bug has been fixed and only the successful transaction is marked as such.
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
        let account: Account =
            serde_json::from_value(result.get("account").unwrap().clone()).unwrap();
        let moving_funds = 100000000000;
        let starting_funds = 10 * moving_funds;

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![b58_decode_public_address(&account.main_address).unwrap()],
            starting_funds,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account.id.clone()),
            &logger,
        );

        let tx_log_id_and_proposals = (0..2)
            .into_iter()
            .map(|_| {
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "build_transaction",
                    "params": {
                        "account_id": account.id,
                        "recipient_public_address": account.main_address,
                        "amount": { "value": moving_funds.to_string(), "token_id": "0"},
                    }
                });
                let res = dispatch(&client, body, &logger);
                let result = res.get("result").unwrap();
                let tx_log_id =
                    serde_json::from_value(result.get("transaction_log_id").unwrap().clone())
                        .unwrap();
                let tx_proposal =
                    serde_json::from_value(result.get("tx_proposal").unwrap().clone()).unwrap();
                (tx_log_id, tx_proposal)
            })
            .collect::<Vec<(String, TxProposalJSON)>>();

        // In order for this test to prove the point the transaction logs should be
        // different while the input txos are the same.
        assert_ne!(tx_log_id_and_proposals[0].0, tx_log_id_and_proposals[1].0);
        assert_eq!(
            tx_log_id_and_proposals[0].1.input_txos,
            tx_log_id_and_proposals[1].1.input_txos
        );

        tx_log_id_and_proposals
            .iter()
            .for_each(|(tx_log_id, tx_proposal)| {
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "submit_transaction",
                    "params": {
                        "tx_proposal": tx_proposal,
                        "account_id": account.id,
                    }
                });
                let res = dispatch(&client, body, &logger);
                let result = res.get("result").unwrap();
                let submitted_tx_log_id = result
                    .get("transaction_log")
                    .unwrap()
                    .get("id")
                    .unwrap()
                    .as_str()
                    .unwrap();
                assert_eq!(tx_log_id, submitted_tx_log_id);
            });

        let tx_proposal = TxProposal::try_from(&tx_log_id_and_proposals[0].1).unwrap();

        add_block_with_tx(&mut ledger_db, tx_proposal.tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account.id),
            &logger,
        );

        // The first transaction log will have succeeded, because it's outputs
        // show up on the ledger.
        // The second transaction log will still be pending, because it's outputs
        // don't exist in the ledger. This second transaction log will
        // eventually fail once the tombstone block is reached.
        let expected_statuses = [TxStatus::Succeeded, TxStatus::Pending];

        tx_log_id_and_proposals
            .iter()
            .zip(expected_statuses)
            .for_each(|((tx_log_id, _), status)| {
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
                let tx_log: TransactionLog =
                    serde_json::from_value(result.get("transaction_log").unwrap().clone()).unwrap();

                assert_eq!(tx_log.status, status.to_string());
                assert_eq!(&tx_log.id, tx_log_id);
            });
    }
}
