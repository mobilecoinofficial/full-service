// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::{
            api::test_utils::{dispatch, setup},
            models::tx_proposal::TxProposal as TxProposalJSON,
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

    use itertools::Itertools;
    use std::convert::TryFrom;

    // The narrative for this test is that an exchange creates subaccounts for its
    // users, Alice, Bob, and Carol. They receive 100, 200, and 300 MOB
    // respectively from an external source. Bob sends 42 MOB to Alice, and the
    // balances should end up as [Alice: 142 MOB, Bob: 158 MOB, Carol: 300 MOB].
    #[test_with_logger]
    fn test_build_and_submit_transaction_with_spend_subaddress(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([3u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Exchange Main Account",
                "require_spend_subaddress": true,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();

        let (
            (alice_public_address, alice_b58_public_address),
            (bob_public_address, bob_b58_public_address),
            (carol_public_address, carol_b58_public_address),
        ) = [
            "Subaddress for Alice",
            "Subaddress for Bob",
            "Subaddress for Carol",
        ]
        .iter()
        .map(|metadata| {
            let body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "assign_address_for_account",
                "params": {
                    "account_id": account_id,
                    "metadata": metadata,
                }
            });
            let res = dispatch(&client, body, &logger);
            let result = res.get("result").unwrap();
            let address = result.get("address").unwrap();
            let b58_address = address.get("public_address_b58").unwrap().as_str().unwrap();
            let public_address = b58_decode_public_address(b58_address).unwrap();
            (public_address, b58_address.to_string())
        })
        .collect_tuple()
        .unwrap();

        // Add a block with a txo for Alice
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        // Add a block with a txo for Bob
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![bob_public_address.clone()],
            200 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        // Add a block with a txo for Carol
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![carol_public_address.clone()],
            300 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 15);

        // Get balance for the exchange, which should include all three subaddress
        // balances. The state of the wallet should be:
        //
        // Overall Balance: 600 MOB
        // Alice Balance: 100 MOB
        // Bob Balance: 200 MOB
        // Carol Balance: 300 MOB
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
        assert_eq!(unspent, (600 * MOB).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        for (public_address, amount) in [
            (&alice_b58_public_address, 100),
            (&bob_b58_public_address, 200),
            (&carol_b58_public_address, 300),
        ]
        .iter()
        {
            let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_status",
            "params": {
                "address": public_address,
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
            assert_eq!(unspent, (amount * MOB).to_string(),);
            assert_eq!(pending, "0");
            assert_eq!(spent, "0");
            assert_eq!(secreted, "0");
            assert_eq!(orphaned, "0");
        }

        // Imagine that Bob is sending 42.0 MOB to Alice
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": alice_b58_public_address,
                "amount": { "value": (42 * MOB).to_string(), "token_id": "0" },
                "spend_subaddress": bob_b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal: TxProposalJSON =
            serde_json::from_value(result.get("tx_proposal").unwrap().clone()).unwrap();
        let payments_tx_proposal = TxProposal::try_from(&tx_proposal).unwrap();

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 16);

        // Get balance after submission, since it is staying in the same wallet, the
        // unspent balance should be the original balance - the fee
        // The state of the wallet should now be:
        //
        // Overall Balance: 600 MOB - 0.0004 MOB
        // Alice Balance: 142 MOB
        // Bob Balance: 158 MOB - 0.0004 MOB
        // Carol Balance: 300 MOB
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
        assert_eq!(unspent, &(600 * MOB - Mob::MINIMUM_FEE).to_string());
        assert_eq!(pending, "0");
        // The SPENT value is calculated by the txos that have key_images that have been
        // burned. It does not take into account change. Bob's 200 MOB TXO was
        // burned to send 42 MOB to Alice.
        assert_eq!(spent, (200 * MOB).to_string());
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Get balance for each address after submission
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_status",
            "params": {
                "address": alice_b58_public_address,
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
        assert_eq!(unspent, (142 * MOB).to_string(),);
        assert_eq!(pending, "0");
        assert_eq!(spent, (0).to_string(),);
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_status",
            "params": {
                "address": bob_b58_public_address,
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
        assert_eq!(
            unspent,
            ((200 * MOB) - (42 * MOB) - Mob::MINIMUM_FEE).to_string(),
        );
        assert_eq!(pending, "0");
        assert_eq!(spent, (200 * MOB).to_string());
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_status",
            "params": {
                "address": carol_b58_public_address,
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
        assert_eq!(unspent, (300 * MOB).to_string(),);
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");
    }

    #[test_with_logger]
    fn test_build_and_submit_transaction_with_require_spend_subaddress_mismatch_fails_if_set(
        logger: Logger,
    ) {
        use crate::error::WalletTransactionBuilderError::NullSubaddress as transaction_error;
        let mut rng: StdRng = SeedableRng::from_seed([3u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Exchange Main Account",
                "require_spend_subaddress": true,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();

        let (
            (_alice_public_address, alice_b58_public_address),
            (bob_public_address, _bob_b58_public_address),
        ) = ["Subaddress for Alice", "Subaddress for Bob"]
            .iter()
            .map(|metadata| {
                let body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "assign_address_for_account",
                "params": {
                "account_id": account_id,
                "metadata": metadata,
                }
                });
                let res = dispatch(&client, body, &logger);
                let result = res.get("result").unwrap();
                let address = result.get("address").unwrap();
                let b58_address = address.get("public_address_b58").unwrap().as_str().unwrap();
                let public_address = b58_decode_public_address(b58_address).unwrap();
                (public_address, b58_address.to_string())
            })
            .collect_tuple()
            .unwrap();

        // Add a block with a txo for Bob
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![bob_public_address.clone()],
            200 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // Imagine that Bob is sending 42.0 MOB to Alice
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": alice_b58_public_address,
                "amount": { "value": (42 * MOB).to_string(), "token_id": "0" },
            }
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();
        let data = error.get("data").unwrap();
        let details = data.get("details").unwrap();
        assert!(details
            .to_string()
            .contains(&transaction_error("This account requires subaddresses be specified when spending. Please provide a subaddress to spend from.".to_string()).to_string()));
    }

    #[test_with_logger]
    fn test_build_and_submit_transaction_with_require_spend_subaddress_mismatch_fails_if_not_set(
        logger: Logger,
    ) {
        let mut rng: StdRng = SeedableRng::from_seed([3u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Exchange Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();

        let (
            (_alice_public_address, alice_b58_public_address),
            (bob_public_address, bob_b58_public_address),
        ) = ["Subaddress for Alice", "Subaddress for Bob"]
            .iter()
            .map(|metadata| {
                let body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "assign_address_for_account",
                "params": {
                "account_id": account_id,
                "metadata": metadata,
                }
                });
                let res = dispatch(&client, body, &logger);
                let result = res.get("result").unwrap();
                let address = result.get("address").unwrap();
                let b58_address = address.get("public_address_b58").unwrap().as_str().unwrap();
                let public_address = b58_decode_public_address(b58_address).unwrap();
                (public_address, b58_address.to_string())
            })
            .collect_tuple()
            .unwrap();

        // Add a block with a txo for Bob
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![bob_public_address.clone()],
            200 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // Imagine that Bob is sending 42.0 MOB to Alice
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": alice_b58_public_address,
                "amount": { "value": (42 * MOB).to_string(), "token_id": "0" },
                "spend_subaddress": bob_b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        assert!(res.get("result").is_some());
    }
}
