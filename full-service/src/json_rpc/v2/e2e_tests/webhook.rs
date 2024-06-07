// Copyright (c) &2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_webhook {
    use crate::{
        json_rpc::v2::api::test_utils::{dispatch, setup},
        test_utils::MOB,
    };
    use std::{ops::DerefMut, thread, time::Duration};

    use mc_common::logger::{log, test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_rand::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;

    use crate::{
        config::WebhookConfig,
        db::{
            account::{AccountID, AccountModel},
            models::Account,
        },
        test_utils::add_block_to_ledger_db,
        util::b58::b58_decode_public_address,
    };
    use httpmock::{Method::POST, MockServer};
    use rand::{rngs::StdRng, SeedableRng};
    use reqwest::{
        blocking::Client,
        header::{HeaderMap, HeaderValue, CONTENT_TYPE},
        Url,
    };
    use serde_json::json;

    #[test_with_logger]
    fn test_webhook(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        // The mock webhook server is listening for which accounts received txos
        let server = MockServer::start();
        // Create a mock on the server.
        let mut sanity_webhook_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/received_txos")
                .body(json!(
                    {
                        "accounts": ["6c683070306016817d5fc963b5d3f794eb8c120428fb14c0ff33ed39ce9bd062"],
                        "restart": false
                    }
                ).to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(json!({"received": true}).to_string()); // FIXME: we don't really care about the response body
        });
        let webhook_url = Url::parse(&server.url("/received_txos")).unwrap();
        let webhook_config = WebhookConfig {
            url: webhook_url.clone(),
        };

        let (client, mut ledger_db, db_ctx, _network_state) =
            setup(&mut rng, Some(webhook_config), logger.clone());
        // NOTE: the webhook should fire on startup as soon as it is caught up, before
        // any accounts are added // FIXME

        let reqwest_client = Client::builder().build().unwrap();
        let mut reqwest_json_headers = HeaderMap::new();
        reqwest_json_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        log::debug!(logger, "calling webhook sanity check");

        // Sanity check: we can hit the webhook server with reqwest
        let response = reqwest_client
            .post(webhook_url)
            .body(json!(
                {
                    "accounts": ["6c683070306016817d5fc963b5d3f794eb8c120428fb14c0ff33ed39ce9bd062"],
                    "restart": false
                }
            ).to_string())
            .send()
            .unwrap()
            .error_for_status()
            .unwrap();
        assert_eq!(response.status(), 200);
        sanity_webhook_mock.assert(); // assert exact contents of the "when" above
        sanity_webhook_mock.assert_hits(1);
        sanity_webhook_mock.delete();

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

        let webhook_mock =
            server.mock(|when, then| {
                when.method(POST).path("/received_txos").body(
                    json!(
                        {
                            "accounts": [account_id],
                            "restart": false
                        }
                    )
                    .to_string(),
                );
                then.status(200)
                    .header("content-type", "application/json")
                    .body(json!({"received": true}).to_string()); // FIXME: we don't really care about the response body
            });

        for i in 0..10 {
            add_block_to_ledger_db(
                &mut ledger_db,
                &vec![public_address.clone()],
                100 * MOB + (i as u64),
                &[KeyImage::from(rng.next_u64())],
                &mut rng,
            );
        }

        // Wait for the account sync thread to finish syncing
        let wallet_db = &db_ctx.get_db_instance(logger.clone());
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        loop {
            let account = Account::get(&AccountID(account_id.to_string()), conn).unwrap();
            if account.next_block_index as u64 >= ledger_db.num_blocks().unwrap() {
                // We have to give the account sync thread a chance to set
                // the shared accounts_with_deposits, and then the webhook thread
                // the chance to fire the webhook
                thread::sleep(Duration::from_millis(100));
                break;
            }
        }

        assert_eq!(ledger_db.num_blocks().unwrap(), 22);
        log::debug!(logger, "webhook was called {} times", webhook_mock.hits());
        assert!(webhook_mock.hits() >= 1); // Should call at least once during
                                           // syncing

        // TODO: Would be great to validate how many txos had been flagged, but
        // it's ok that this is relatively lossy in this test, because
        // what we care about is that the webhook _is_ getting called
        // when there are new txos
    }
}
