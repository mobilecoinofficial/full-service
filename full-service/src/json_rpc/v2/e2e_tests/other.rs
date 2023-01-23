// Copyright (c) &2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_misc {
    use crate::{
        json_rpc::v2::{
            api::{
                test_utils::{
                    dispatch, dispatch_with_header, dispatch_with_header_expect_error, setup,
                    setup_no_wallet_db, setup_with_api_key, wait_for_sync,
                },
                wallet::RECENT_BLOCKS_DEFAULT_LIMIT,
            },
            models::{
                block::{Block, BlockContents},
                network_status::NetworkStatus,
            },
        },
        test_utils::{
            add_block_with_tx_outs, create_test_received_txo, random_account_with_seed_values, MOB,
        },
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Amount, BlockVersion, Token};

    use rand::{rngs::StdRng, SeedableRng};
    use rocket::http::{Header, Status};
    use serde_json::json;

    #[test_with_logger]
    fn test_wallet_status(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let _result = dispatch(&client, body, &logger).get("result").unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_wallet_status",
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let status = result.get("wallet_status").unwrap();
        assert_eq!(status.get("network_block_height").unwrap(), "12");
        assert_eq!(status.get("local_block_height").unwrap(), "12");
        // Syncing will have already started, so we can't determine what the min synced
        // index is.
        assert!(status.get("min_synced_block_index").is_some());
        let balance_per_token = status.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string());
        assert!(balance_mob.is_none());
    }

    #[test_with_logger]
    fn test_request_with_correct_api_key(logger: Logger) {
        let api_key = "mobilecats";

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) =
            setup_with_api_key(&mut rng, logger.clone(), api_key.to_string());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            },
        });

        let header = Header::new("X-API-KEY", api_key);

        dispatch_with_header(&client, body, header, &logger);
    }

    #[test_with_logger]
    fn test_request_with_bad_api_key(logger: Logger) {
        let api_key = "mobilecats";

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) =
            setup_with_api_key(&mut rng, logger.clone(), api_key.to_string());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            },
        });

        let header = Header::new("X-API-KEY", "wrong-header");

        dispatch_with_header_expect_error(&client, body, header, &logger, Status::Unauthorized);
    }

    #[test_with_logger]
    fn test_get_network_status(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_network_status"
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let status = result.get("network_status").unwrap();
        assert_eq!(status.get("network_block_height").unwrap(), "12");
        assert_eq!(status.get("local_block_height").unwrap(), "12");
        assert_eq!(status.get("local_num_txos").unwrap(), "60");
        assert_eq!(
            status.get("block_version").unwrap(),
            &BlockVersion::MAX.to_string()
        );

        let fees = status.get("fees").unwrap().as_object().unwrap();
        assert_eq!(
            fees.get(&Mob::ID.to_string()).unwrap().as_str().unwrap(),
            &Mob::MINIMUM_FEE.to_string()
        );
    }

    #[test_with_logger]
    fn test_get_txo_block_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());
        let wallet_db = db_ctx.get_db_instance(logger.clone());
        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB],
            &mut rng,
            &logger,
        );

        let (_, tx_out, _) = create_test_received_txo(
            &account_key,
            0,
            Amount::new(70, Mob::ID),
            13,
            &mut rng,
            &wallet_db,
        );

        add_block_with_tx_outs(
            &mut ledger_db,
            &[tx_out.clone()],
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_sync(&client, &ledger_db, &network_state, &logger);

        // A valid public key on the ledger
        let public_key = hex::encode(tx_out.public_key.as_bytes());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txo_block_index",
            "params": {
                "public_key": public_key
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert_eq!(result.get("block_index").unwrap(), "13");

        // An invalid public key on the ledger
        let target_key = hex::encode(tx_out.target_key.as_bytes());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txo_block_index",
            "params": {
                "public_key": target_key
            }
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();
        assert_eq!(
            error.get("data").unwrap().get("server_error").unwrap(),
            "LedgerDB(Record not found)"
        );
    }

    #[test_with_logger]
    fn test_get_block_by_txo_public_key(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());
        let wallet_db = db_ctx.get_db_instance(logger.clone());
        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB],
            &mut rng,
            &logger,
        );

        let (_, tx_out, _) = create_test_received_txo(
            &account_key,
            0,
            Amount::new(70, Mob::ID),
            13,
            &mut rng,
            &wallet_db,
        );

        add_block_with_tx_outs(
            &mut ledger_db,
            &[tx_out.clone()],
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_sync(&client, &ledger_db, &network_state, &logger);

        // A valid public key on the ledger
        let public_key = hex::encode(tx_out.public_key.as_bytes());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_block",
            "params": {
                "txo_public_key": public_key
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let block = result.get("block").unwrap();

        assert_eq!(block.get("index").unwrap(), "13");

        // An invalid public key on the ledger
        let target_key = hex::encode(tx_out.target_key.as_bytes());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txo_block_index",
            "params": {
                "public_key": target_key
            }
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();
        assert_eq!(
            error.get("data").unwrap().get("server_error").unwrap(),
            "LedgerDB(Record not found)"
        );
    }

    #[test_with_logger]
    fn test_get_block_by_block_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _, _, _) = setup(&mut rng, logger.clone());

        // A valid block index on the ledger
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_block",
            "params": {
                "block_index": "11"
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let block = result.get("block").unwrap();

        assert_eq!("11", block.get("index").unwrap());

        // An invalid block index on the ledger
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_block",
            "params": {
                "block_index": "13"
            }
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();

        assert_eq!(
            error.get("data").unwrap().get("server_error").unwrap(),
            "LedgerDB(Record not found)"
        );
    }

    #[test_with_logger]
    fn test_get_recent_blocks(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _, _, _) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_recent_blocks",
            "params": {
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let blocks: Vec<Block> =
            serde_json::from_value(result.get("blocks").unwrap().clone()).unwrap();
        let block_contents: Vec<BlockContents> =
            serde_json::from_value(result.get("block_contents").unwrap().clone()).unwrap();
        let network_status: NetworkStatus =
            serde_json::from_value(result.get("network_status").unwrap().clone()).unwrap();

        assert_eq!(network_status.network_block_height, "12");
        assert_eq!(network_status.local_block_height, "12");

        assert_eq!(blocks.len(), block_contents.len());
        assert_eq!(blocks.len(), RECENT_BLOCKS_DEFAULT_LIMIT);

        // The most recent block should be the last one
        assert_eq!(blocks[0].index, "11");

        // Blocks should be in decending order
        assert!(blocks
            .windows(2)
            .all(|w| w[0].index.parse::<u64>().unwrap() == w[1].index.parse::<u64>().unwrap() + 1));

        // Limit should work
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_recent_blocks",
            "params": {
                "limit": 3,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let blocks: Vec<Block> =
            serde_json::from_value(result.get("blocks").unwrap().clone()).unwrap();
        let block_contents: Vec<BlockContents> =
            serde_json::from_value(result.get("block_contents").unwrap().clone()).unwrap();
        let network_status: NetworkStatus =
            serde_json::from_value(result.get("network_status").unwrap().clone()).unwrap();

        assert_eq!(network_status.network_block_height, "12");
        assert_eq!(network_status.local_block_height, "12");

        assert_eq!(blocks.len(), block_contents.len());
        assert_eq!(blocks.len(), 3);

        // The most recent block should be the last one
        assert_eq!(blocks[0].index, "11");

        // Blocks should be in decending order
        assert!(blocks
            .windows(2)
            .all(|w| w[0].index.parse::<u64>().unwrap() == w[1].index.parse::<u64>().unwrap() + 1));
    }

    #[test_with_logger]
    fn test_get_blocks(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _, _, _) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_blocks",
            "params": {
                "first_block_index": "5",
                "limit": 3,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let blocks: Vec<Block> =
            serde_json::from_value(result.get("blocks").unwrap().clone()).unwrap();
        let block_contents: Vec<BlockContents> =
            serde_json::from_value(result.get("block_contents").unwrap().clone()).unwrap();

        assert_eq!(blocks.len(), block_contents.len());
        assert_eq!(blocks.len(), 3);

        // The first block should be the one we requested
        assert_eq!(blocks[0].index, "5");

        // Blocks should be in ascending order
        assert!(blocks
            .windows(2)
            .all(|w| w[0].index.parse::<u64>().unwrap() == w[1].index.parse::<u64>().unwrap() - 1));
    }

    #[test_with_logger]
    fn test_no_wallet_db(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) =
            setup_no_wallet_db(&mut rng, logger.clone());

        // Because we are not using a ledger_db, this should return an error!
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_accounts",
            "params": {},
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();
        let data = error.get("data").unwrap();
        assert_eq!(
            data.get("server_error").unwrap(),
            "Database(WalletFunctionsDisabled)"
        );

        // This should work just fine since it doesn't interact with the wallet_db
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_network_status"
        });
        let res = dispatch(&client, body, &logger);

        // Check that we got a result! (We don't really care what it is, just that it's
        // working)
        let _ = res.get("result").unwrap();
    }
}
