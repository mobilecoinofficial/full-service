// Copyright (c) &2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_misc {
    use crate::json_rpc::v2::api::test_utils::{
        dispatch, dispatch_with_header, dispatch_with_header_expect_error, setup,
        setup_with_api_key,
    };

    use mc_common::logger::{test_with_logger, Logger};

    use mc_transaction_core::{tokens::Mob, BlockVersion, Token};

    use rand::{rngs::StdRng, SeedableRng};
    use rocket::http::{Header, Status};

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
}
