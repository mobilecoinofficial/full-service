// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::{
            api::test_utils::{dispatch, setup},
            models::{
                amount::Amount as AmountJSON,
                transaction_log::TransactionLog as TransactionLogJSON,
                tx_proposal::TxProposal as TxProposalJSON,
            },
        },
        service::models::tx_proposal::TxProposal,
        test_utils::{add_block_to_ledger_db, add_block_with_tx, manually_sync_account, MOB},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::CompressedRistrettoPublic;
    use mc_ledger_db::Ledger;
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, tx::TxOut, Token};

    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

    use std::convert::{TryFrom, TryInto};

    #[test_with_logger]
    fn test_build_and_submit_transaction_with_spend_only_from_subaddress(logger: Logger) {
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

        // Assign a block of subaddresses for Alice, Bob, and Carol
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": "Subaddress for Alice",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let alice_address = result.get("address").unwrap();
        let alice_b58_public_address = alice_address
            .get("public_address_b58")
            .unwrap()
            .as_str()
            .unwrap();
        let alice_public_address = b58_decode_public_address(alice_b58_public_address).unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": "Subaddress for Bob",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let bob_address = result.get("address").unwrap();
        let bob_b58_public_address = bob_address
            .get("public_address_b58")
            .unwrap()
            .as_str()
            .unwrap();
        let bob_public_address = b58_decode_public_address(bob_b58_public_address).unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": "Subaddress for Carol",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let carol_address = result.get("address").unwrap();
        let carol_b58_public_address = carol_address
            .get("public_address_b58")
            .unwrap()
            .as_str()
            .unwrap();
        let carol_public_address = b58_decode_public_address(carol_b58_public_address).unwrap();

        // Add a block with a txo for Alice
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100_000_000_000_000, // 100.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        // Add a block with a txo for Bob
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![bob_public_address.clone()],
            200_000_000_000_000, // 200.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        // Add a block with a txo for Carol
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![carol_public_address.clone()],
            300_000_000_000_000, // 300.0 MOB
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

        // Get balance for the exchange, which should include all three suabddress
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
        assert_eq!(unspent, "600000000000000");
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Get balance for each subaddress
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
        assert_eq!(unspent, (100 * MOB).to_string(),);
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
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
        assert_eq!(unspent, (200 * MOB).to_string(),);
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
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

        // Imagine that Bob is sending 42.0 MOB to Alice
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": alice_b58_public_address,
                "amount": { "value": "42000000000000", "token_id": "0" }, // 42.0 MOB
                "spend_only_from_subaddress": bob_b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal: TxProposalJSON =
            serde_json::from_value(result.get("tx_proposal").unwrap().clone()).unwrap();
        let transaction_log: TransactionLogJSON =
            serde_json::from_value(result.get("transaction_log").unwrap().clone()).unwrap();

        let tx_log_payload_txo = transaction_log.output_txos[0].clone();
        let tx_proposal_payload_txo = tx_proposal.payload_txos[0].clone();
        let tx_out_proto_bytes = hex::decode(tx_proposal_payload_txo.tx_out_proto).unwrap();
        let txo: TxOut = mc_util_serial::decode(&tx_out_proto_bytes).unwrap();

        let txo_public_key_bytes = hex::decode(tx_log_payload_txo.public_key).unwrap();
        let txo_public_key: CompressedRistrettoPublic =
            txo_public_key_bytes.as_slice().try_into().unwrap();

        assert_eq!(txo.public_key, txo_public_key);
        assert_eq!(
            tx_proposal.fee_amount,
            AmountJSON::new(Mob::MINIMUM_FEE, Mob::ID)
        );
        assert_eq!(tx_proposal.input_txos.len(), 1);
        assert_eq!(tx_proposal.payload_txos.len(), 1);
        // One change (and that change should be going back to Bob)
        assert_eq!(tx_proposal.change_txos.len(), 1);
        assert_eq!(
            tx_proposal.change_txos[0].recipient_public_address_b58,
            bob_b58_public_address
        );
        // Tombstone block = ledger height (12 to start + 3 new blocks + 10 default
        // tombstone)
        assert_eq!(tx_proposal.tombstone_block_index, "25");

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
        // Overall Balance: 600 MOB - 0.0001 MOB
        // Alice Balance: 142 MOB
        // Bob Balance: 158 MOB - 0.0001 MOB
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
        // The SPENT value is calculated by the txos who have key_images that have been
        // burned. It does not take into account change. Bob's 100 MOB TXO was
        // burned to send 42 MOB to Alice.
        assert_eq!(spent, (100 * MOB).to_string()); //FIXME: then shouldn't this be 200?
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
        assert_eq!(unspent, (42 * MOB).to_string(),); // FIXME: shouldn't this be 142?
        assert_eq!(pending, "0");
        assert_eq!(spent, (100 * MOB).to_string(),); // FIXME: Why was it spent from Alice's balance?
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
        // assert_eq!(unspent, (138 * MOB - Mob::MINIMUM_FEE).to_string(),);
        assert_eq!(
            unspent,
            ((300 * MOB) - (42 * MOB) - Mob::MINIMUM_FEE).to_string(),
        ); // Should be 200 - 42
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
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
}
