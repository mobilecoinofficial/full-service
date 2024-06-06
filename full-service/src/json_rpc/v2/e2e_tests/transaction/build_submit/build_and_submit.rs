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
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx, add_block_with_tx_outs,
            manually_sync_account,
        },
        util::b58::b58_decode_public_address,
    };

    use mc_blockchain_types::BlockVersion;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPrivate};
    use mc_ledger_db::Ledger;
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{
        ring_signature::KeyImage, tokens::Mob, tx::TxOut, Amount, Token, TokenId,
    };
    use mc_util_from_random::FromRandom;

    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

    use std::convert::{TryFrom, TryInto};

    #[test_with_logger]
    fn test_build_and_submit_transaction(logger: Logger) {
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

        // Add a block with significantly more MOB
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            100_000_000_000_000, // 100.0 MOB
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
        assert_eq!(unspent, "100000000000100");
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Create a tx proposal to ourselves
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "amount": { "value": "42000000000000", "token_id": "0" }, // 42.0 MOB
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

        // Transaction builder attempts to use as many inputs as we have txos
        assert_eq!(tx_proposal.input_txos.len(), 2);

        // One destination
        assert_eq!(tx_proposal.payload_txos.len(), 1);

        assert_eq!(tx_proposal.change_txos.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 10 default
        // tombstone)
        assert_eq!(tx_proposal.tombstone_block_index, "24");

        let payments_tx_proposal = TxProposal::try_from(&tx_proposal).unwrap();

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 15);

        // Get balance after submission, since we are sending it to ourselves, the
        // unspent balance should be the original balance - the fee
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
        assert_eq!(unspent, &(100000000000100 - Mob::MINIMUM_FEE).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

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
        assert_eq!(unspent, "1000000000000".to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Create a tx proposal to ourselves, but this should fail because we cannot yet
        // do mixed token transactions
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_and_submit_transaction",
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
            "method": "build_and_submit_transaction",
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

        // 1024 is the known minimum fee for token id 1, it just isn't in the mobilecoin
        // library anywhere yet as a const
        assert_eq!(
            tx_proposal.fee_amount,
            AmountJSON::new(1024, TokenId::from(1))
        );

        assert_eq!(tx_proposal.input_txos.len(), 1);

        // One destination
        assert_eq!(tx_proposal.payload_txos.len(), 1);

        assert_eq!(tx_proposal.change_txos.len(), 1);

        // Tombstone block = ledger height (14 to start + 2 new blocks + 10 default
        // tombstone)
        assert_eq!(tx_proposal.tombstone_block_index, "26");

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

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);

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
}
