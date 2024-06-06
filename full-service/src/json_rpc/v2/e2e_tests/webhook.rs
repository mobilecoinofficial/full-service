// Copyright (c) &2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_webhook {
    use crate::{
        json_rpc::v2::api::test_utils::{dispatch, setup},
        test_utils::MOB,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_rand::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;

    use crate::{
        db::account::AccountID,
        test_utils::{add_block_to_ledger_db, manually_sync_account},
        util::b58::b58_decode_public_address,
    };
    use rand::{rngs::StdRng, SeedableRng};
    use rocket::http::{ContentType, Status};
    use serde_json::json;

    #[test_with_logger]
    fn test_webhook(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        // FIXME: will need to set up with webhook
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Confirm we can test a webhook at all with our setup
        let payload = json!({"field1": "test", "field2": 10});
        let res = client
            .get("/webhook")
            .header(ContentType::JSON)
            .body(payload.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // Add an account and force it to sync
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let account_obj = res.get("result").unwrap().get("account").unwrap();
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
        let public_address =
            b58_decode_public_address(account_obj.get("main_address").unwrap().as_str().unwrap())
                .unwrap();

        for i in 1..10 {
            add_block_to_ledger_db(
                &mut ledger_db,
                &vec![public_address.clone()],
                100 * MOB + (i as u64),
                &[KeyImage::from(rng.next_u64())],
                &mut rng,
            );
        }

        // During "sync_account_next_chunk" the webhook will be called
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 21);
    }
}
