// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_transaction {
    use crate::{
        db::account::AccountID,
        json_rpc,
        json_rpc::v1::api::test_utils::{dispatch, setup},
        test_utils::{add_block_to_ledger_db, add_block_with_tx, manually_sync_account},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_rand::rand_core::RngCore;

    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_send_txo_received_from_removed_account(logger: Logger) {
        use crate::db::schema::txos;
        use diesel::{dsl::count, prelude::*};

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let wallet_db = db_ctx.get_db_instance(logger.clone());

        // Add three accounts.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "account 1",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id_1 = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address_1 = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address_1 = b58_decode_public_address(b58_public_address_1).unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "account 2",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id_2 = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address_2 = account_obj.get("main_address").unwrap().as_str().unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "account 3",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id_3 = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address_3 = account_obj.get("main_address").unwrap().as_str().unwrap();

        // Add a block to fund account 1.
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            0
        );
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address_1],
            100000000000000, // 100.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(account_id_1.to_string()),
            &logger,
        );
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            1
        );

        // Send some coins to account 2.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id_1,
                "recipient_public_address": b58_public_address_2,
                "value_pmob": "84000000000000", // 84.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": account_id_1,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result");
        assert!(result.is_some());

        let json_tx_proposal: json_rpc::v1::models::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);

        manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(account_id_2.to_string()),
            &logger,
        );
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            3
        );

        // Remove account 1.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": account_id_1,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert!(result["removed"].as_bool().unwrap(),);
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            1
        );

        // Send coins from account 2 to account 3.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id_2,
                "recipient_public_address": b58_public_address_3,
                "value_pmob": "42000000000000", // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": account_id_2,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result");
        assert!(result.is_some());

        let json_tx_proposal: json_rpc::v1::models::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);

        manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID(account_id_3.to_string()),
            &logger,
        );
        assert_eq!(
            txos::table
                .select(count(txos::id))
                .first::<i64>(&mut wallet_db.get_pooled_conn().unwrap())
                .unwrap(),
            3
        );

        // Check that account 3 received its coins.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": account_id_3,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status["unspent_pmob"].as_str().unwrap();
        assert_eq!(unspent, "42000000000000"); // 42.0 MOB
    }

    /// This test is intended to make sure that when a subaddress is assigned
    /// that it correctly generates and checks the key image against the ledger
    /// db to see if the previously orphaned txo has been spent or not
    #[test_with_logger]
    fn test_mark_orphaned_txo_as_spent(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "2",
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();

        // Assign next subaddress for account.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": "subaddress_index_2",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let address = result.get("address").unwrap();
        let b58_public_address = address.get("public_address").unwrap().as_str().unwrap();
        let public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add a block to fund account at the new subaddress.
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            100000000000000, // 100.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            500000000000000, // 500.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // Remove the account.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert!(result["removed"].as_bool().unwrap(),);

        // Add the same account back.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "2",
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance = result.get("balance").unwrap();
        assert_eq!(balance.get("unspent_pmob").unwrap(), "0");
        assert_eq!(balance.get("spent_pmob").unwrap(), "0");
        assert_eq!(balance.get("orphaned_pmob").unwrap(), "600000000000000");

        // Add back next subaddress. Txos are detected as unspent.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": "subaddress_index_2",
            }
        });
        dispatch(&client, body, &logger);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance = result.get("balance").unwrap();
        assert_eq!(balance.get("unspent_pmob").unwrap(), "600000000000000");
        assert_eq!(balance.get("spent_pmob").unwrap(), "0");
        assert_eq!(balance.get("orphaned_pmob").unwrap(), "0");

        // Create a second account.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "account 2",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id_2 = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address_2 = account_obj.get("main_address").unwrap().as_str().unwrap();

        // Remove the second Account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": *account_id_2,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert!(result["removed"].as_bool().unwrap(),);

        // Send some coins to the removed second account.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address_2,
                "value_pmob": "50000000000000", // 50.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result");
        assert!(result.is_some());

        let json_tx_proposal: json_rpc::v1::models::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // The first account shows the coins are spent.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance = result.get("balance").unwrap();
        assert_eq!(balance.get("unspent_pmob").unwrap(), "549999600000000");
        assert_eq!(balance.get("spent_pmob").unwrap(), "100000000000000");
        assert_eq!(balance.get("orphaned_pmob").unwrap(), "0");

        // Remove the first account and add it back again.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert!(result["removed"].as_bool().unwrap(),);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "2",
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // The unspent pmob shows what wasn't sent to the second account.
        // The orphaned pmob are because we haven't added back the next subaddress.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance = result.get("balance").unwrap();
        assert_eq!(balance.get("unspent_pmob").unwrap(), "49999600000000");
        assert_eq!(balance.get("spent_pmob").unwrap(), "0");
        assert_eq!(balance.get("orphaned_pmob").unwrap(), "600000000000000");
    }

    #[test_with_logger]
    fn test_get_txos(logger: Logger) {
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

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
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
        let txos = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos.len(), 1);
        let txo_map = result.get("txo_map").unwrap().as_object().unwrap();
        let txo = txo_map.get(txos[0].as_str().unwrap()).unwrap();
        let account_status_map = txo
            .get("account_status_map")
            .unwrap()
            .as_object()
            .unwrap()
            .get(account_id)
            .unwrap();
        let txo_status = account_status_map
            .get("txo_status")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_status, "txo_status_unspent");
        let txo_type = account_status_map
            .get("txo_type")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_type, "txo_type_received");
        let value = txo.get("value_pmob").unwrap().as_str().unwrap();
        assert_eq!(value, "100");

        // Check the overall balance for the account
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
        let unspent = balance_status["unspent_pmob"].as_str().unwrap();
        assert_eq!(unspent, "100");
    }

    #[test_with_logger]
    fn test_split_txo(logger: Logger) {
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

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            250000000000,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

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
        let txos = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos.len(), 1);
        let txo_map = result.get("txo_map").unwrap().as_object().unwrap();
        let txo = txo_map.get(txos[0].as_str().unwrap()).unwrap();
        let account_status_map = txo
            .get("account_status_map")
            .unwrap()
            .as_object()
            .unwrap()
            .get(account_id)
            .unwrap();
        let txo_status = account_status_map
            .get("txo_status")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_status, "txo_status_unspent");
        let txo_type = account_status_map
            .get("txo_type")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_type, "txo_type_received");
        let value = txo.get("value_pmob").unwrap().as_str().unwrap();
        assert_eq!(value, "250000000000");
        let txo_id = &txos[0];

        // Check the overall balance for the account
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
        let unspent = balance_status["unspent_pmob"].as_str().unwrap();
        assert_eq!(unspent, "250000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_split_txo_transaction",
            "params": {
                "txo_id": txo_id,
                "output_values": ["20000000000", "80000000000", "30000000000", "70000000000", "40000000000"],
                "fee": "10000000000"
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

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
        let result = res.get("result");
        assert!(result.is_some());

        let json_tx_proposal: json_rpc::v1::models::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx(&mut ledger_db, payments_tx_proposal.tx, &mut rng);

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        // Check the overall balance for the account
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
        let unspent = balance_status["unspent_pmob"].as_str().unwrap();
        assert_eq!(unspent, "240000000000");
    }
}
