// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc::{
            api_test_utils::{dispatch, setup},
            tx_proposal::TxProposal as TxProposalJSON,
        },
        service::models::tx_proposal::TxProposal,
        test_utils::{add_block_to_ledger_db, add_block_with_tx_proposal, manually_sync_account},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_build_and_submit_transaction(logger: Logger) {
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
            &vec![public_address],
            100_000_000_000_000, // 100.0 MOB
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
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value":["42000000000000", "0"], // 42.0 MOB
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

        // One destination
        let payload_txos = tx_proposal.get("payload_txos").unwrap().as_array().unwrap();
        assert_eq!(payload_txos.len(), 1);

        let change_txos = tx_proposal.get("change_txos").unwrap().as_array().unwrap();
        assert_eq!(change_txos.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 10 default
        // tombstone)
        let tombstone_block_index = tx_proposal.get("tombstone_block_index").unwrap();
        assert_eq!(tombstone_block_index, "24");

        let json_tx_proposal: TxProposalJSON = serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal = TxProposal::try_from(&json_tx_proposal).unwrap();

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
        assert_eq!(unspent, &(100000000000100 - Mob::MINIMUM_FEE).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");
    }
}
