// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc::{
            api_test_utils::{dispatch, setup},
            tx_proposal::TxProposalJSON,
        },
        service::models::tx_proposal::TxProposal,
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_proposal, manually_sync_account, MOB,
        },
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;

    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};

    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_gift_codes(logger: Logger) {
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
        // Create a gift code
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_gift_code",
            "params": {
                "account_id": alice_account_id,
                "value_pmob": "42000000000000",
                "memo": "Happy Birthday!",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"].clone();
        let gift_code_b58 = result["gift_code_b58"].as_str().unwrap();
        let tx_proposal = result["tx_proposal"].clone();

        // Check the status of the gift code
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check_gift_code_status",
            "params": {
                "gift_code_b58": gift_code_b58,
            }
        });
        let res = dispatch(&client, body, &logger);
        let status = res["result"]["gift_code_status"].as_str().unwrap();
        assert_eq!(status, "GiftCodeSubmittedPending");
        let memo = res["result"]["gift_code_memo"].as_str().unwrap();
        assert_eq!(memo, "Happy Birthday!");

        // Submit the gift code and tx proposal
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_gift_code",
            "params": {
                "from_account_id": alice_account_id,
                "gift_code_b58": gift_code_b58,
                "tx_proposal": tx_proposal,
            }
        });
        dispatch(&client, body, &logger);

        // Add the TxProposal for the gift code
        let json_tx_proposal: TxProposalJSON = serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal = TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal, &mut rng);

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            &logger,
        );

        // Check the status of the gift code
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check_gift_code_status",
            "params": {
                "gift_code_b58": gift_code_b58,
            }
        });
        let res = dispatch(&client, body, &logger);
        let status = res["result"]["gift_code_status"].as_str().unwrap();
        assert_eq!(status, "GiftCodeAvailable");
        let memo = res["result"]["gift_code_memo"].as_str().unwrap();
        assert_eq!(memo, "Happy Birthday!");

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

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(bob_account_id.to_string()),
            &logger,
        );

        // Get all the gift codes in the wallet
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_gift_codes",
            "params": {}
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"]["gift_codes"].as_array().unwrap();
        assert_eq!(result.len(), 1);

        // Get the specific gift code
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_gift_code",
            "params": {
                "gift_code_b58": gift_code_b58,
            }
        });
        dispatch(&client, body, &logger);

        // Claim the gift code for bob
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "claim_gift_code",
            "params": {
                "account_id": bob_account_id,
                "gift_code_b58": gift_code_b58,
            }
        });
        let res = dispatch(&client, body, &logger);
        let txo_id_hex = res["result"]["txo_id"].as_str().unwrap();
        assert_eq!(txo_id_hex.len(), 64);

        // Now remove that gift code
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "remove_gift_code",
            "params": {
                "gift_code_b58": gift_code_b58,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"]["removed"].as_bool().unwrap();
        assert!(result);

        // Get all the gift codes in the wallet again, should be 0 now
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_gift_codes",
            "params": {}
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"]["gift_codes"].as_array().unwrap();
        assert_eq!(result.len(), 0);
    }
}
