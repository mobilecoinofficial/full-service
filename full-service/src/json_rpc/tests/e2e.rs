// Copyright (c) 2020-2021 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

use crate::{
    db::{
        b58_decode,
        models::{TXO_PENDING, TXO_RECEIVED, TXO_SECRETED, TXO_SPENT, TXO_UNSPENT},
    },
    json_rpc::api_test_utils::{dispatch, dispatch_expect_error, setup, wait_for_sync},
    test_utils::add_block_to_ledger_db,
};
use mc_common::logger::{test_with_logger, Logger};
use mc_crypto_rand::rand_core::RngCore;
use mc_transaction_core::ring_signature::KeyImage;
use rand::{rngs::StdRng, SeedableRng};

#[test_with_logger]
fn test_account_crud(logger: Logger) {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
    let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

    // Create Account
    let body = json!({
        "jsonrpc": "2.0",
        "method": "create_account",
        "params": {
            "name": "Alice Main Account",
        },
        "api_version": "2",
    });
    let res = dispatch(&client, body, &logger);
    assert_eq!(res.get("jsonrpc").unwrap(), "2.0");

    let result = res.get("result").unwrap();
    let account_obj = result.get("account").unwrap();
    assert!(account_obj.get("account_id").is_some());
    assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
    assert_eq!(account_obj.get("network_height").unwrap(), "12");
    assert_eq!(account_obj.get("local_height").unwrap(), "12");
    assert_eq!(account_obj.get("account_height").unwrap(), "0");
    assert_eq!(account_obj.get("is_synced").unwrap(), false);
    assert_eq!(account_obj.get("available_pmob").unwrap(), "0");
    assert_eq!(account_obj.get("pending_pmob").unwrap(), "0");
    assert!(account_obj.get("main_address").is_some());
    assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
    assert_eq!(account_obj.get("recovery_mode").unwrap(), false);

    // The initial creation of an account returns the entropy and account_key for
    // safe keeping.
    assert!(account_obj.get("entropy").is_some());
    assert!(account_obj.get("account_key").is_some());

    let account_id = account_obj.get("account_id").unwrap();
}
