// Copyright (c) 2020-2021 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e {
    use crate::{
        db::{
            account::AccountID,
            models::{TXO_STATUS_UNSPENT, TXO_TYPE_RECEIVED},
        },
        json_rpc,
        json_rpc::api_test_utils::{dispatch, dispatch_expect_error, setup, wait_for_sync},
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_proposal,
            wait_for_sync as wait_for_account_sync, MOB,
        },
        util::b58::b58_decode_public_address,
    };
    use bip39::{Language, Mnemonic};
    use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
    use mc_account_keys_slip10::Slip10Key;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{constants::MINIMUM_FEE, ring_signature::KeyImage};
    use rand::{rngs::StdRng, SeedableRng};
    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_e2e_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Create Account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            },
        });
        let res = dispatch(&client, body, &logger);
        assert_eq!(res.get("jsonrpc").unwrap(), "2.0");

        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        assert!(account_obj.get("account_id").is_some());
        assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
        assert!(account_obj.get("main_address").is_some());
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
        assert_eq!(account_obj.get("recovery_mode").unwrap(), false);

        let account_id = account_obj.get("account_id").unwrap();

        // Read Accounts via Get All
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_all_accounts",
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let accounts = result.get("account_ids").unwrap().as_array().unwrap();
        assert_eq!(accounts.len(), 1);
        let account_map = result.get("account_map").unwrap().as_object().unwrap();
        assert_eq!(
            account_map
                .get(accounts[0].as_str().unwrap())
                .unwrap()
                .get("account_id")
                .unwrap(),
            &account_id.clone()
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let name = result.get("account").unwrap().get("name").unwrap();
        assert_eq!("Alice Main Account", name.as_str().unwrap());

        // FIXME: assert balance

        // Update Account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "update_account_name",
            "params": {
                "account_id": *account_id,
                "name": "Eve Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert_eq!(
            result.get("account").unwrap().get("name").unwrap(),
            "Eve Main Account"
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let name = result.get("account").unwrap().get("name").unwrap();
        assert_eq!("Eve Main Account", name.as_str().unwrap());

        // Remove Account
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
        assert_eq!(result["removed"].as_bool().unwrap(), true,);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_all_accounts",
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let accounts = result.get("account_ids").unwrap().as_array().unwrap();
        assert_eq!(accounts.len(), 0);
    }

    #[test_with_logger]
    fn test_e2e_import_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "2",
                "name": "Alice Main Account",
                "first_block_index": "200",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "3CnfxAc2LvKw4FDNRVgj3GndwAhgQDd7v2Cne66GTUJyzBr3WzSikk9nJ5sCAb1jgSSKaqpWQtcEjV1nhoadVKjq2Soa8p3XZy6u2tpHdor");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            account_id,
            "7872edf0d4094643213aabc92aa0d07379cfb58eda0722b21a44868f22f75b4e"
        );

        assert_eq!(
            *account_obj.get("first_block_index").unwrap(),
            serde_json::json!("200")
        );
    }

    #[test_with_logger]
    fn test_e2e_import_account_unknown_version(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "3",
                "name": "",
            }
        });
        dispatch_expect_error(
            &client,
            body,
            &logger,
            json!({
                "method": "import_account",
                "error": json!({
                    "code": -32603,
                    "message": "InternalError",
                    "data": json!({
                        "server_error": "UnknownKeyDerivation(3)",
                        "details": "Unknown key version version: 3",
                    })
                }),
                "jsonrpc": "2.0",
                "id": 1,
            })
            .to_string(),
        );
    }

    #[test_with_logger]
    fn test_e2e_import_account_legacy(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
                "first_block_index": "200",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        // Catches if a change results in changed accounts_ids, which should always be
        // made to be backward compatible.
        assert_eq!(
            account_id,
            "f9957a9d050ef8dff9d8ef6f66daa608081e631b2d918988311613343827b779"
        );
        assert_eq!(
            *account_obj.get("first_block_index").unwrap(),
            serde_json::json!("200")
        );
    }

    #[test_with_logger]
    fn test_e2e_import_delete_import(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
                "first_block_index": "200",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        // Catches if a change results in changed accounts_ids, which should always be
        // made to be backward compatible.
        assert_eq!(
            account_id,
            "f9957a9d050ef8dff9d8ef6f66daa608081e631b2d918988311613343827b779"
        );

        // Delete Account
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
        assert_eq!(result["removed"].as_bool().unwrap(), true);

        // Import it again - should succeed.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
                "first_block_index": "200",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU");
    }

    #[test_with_logger]
    fn test_export_account_secrets(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "2",
                "name": "Alice Main Account",
                "first_block_index": "200",
            }
        });
        let res = dispatch(&client, body, &logger);
        let account_obj = res["result"]["account"].clone();
        let account_id = account_obj["account_id"].clone();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "export_account_secrets",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let secrets = result.get("account_secrets").unwrap();
        let phrase = secrets["mnemonic"].as_str().unwrap();
        assert_eq!(secrets["account_id"], serde_json::json!(account_id));
        assert_eq!(secrets["key_derivation_version"], serde_json::json!("2"));

        // Test that the mnemonic serializes correctly back to an AccountKey object
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English).unwrap();
        let account_key = Slip10Key::from(mnemonic.clone())
            .try_into_account_key(
                &"".to_string(),
                &"".to_string(),
                &hex::decode("".to_string()).expect("invalid spki"),
            )
            .unwrap();

        assert_eq!(
            serde_json::json!(json_rpc::account_key::AccountKey::try_from(&account_key).unwrap()),
            secrets["account_key"]
        );
    }

    #[test_with_logger]
    fn test_export_legacy_account_secrets(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        let entropy = "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b";
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": entropy,
                "name": "Alice Main Account",
                "first_block_index": "200",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "export_account_secrets",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let secrets = result.get("account_secrets").unwrap();

        assert_eq!(secrets["account_id"], serde_json::json!(account_id));
        assert_eq!(secrets["entropy"], serde_json::json!(entropy));
        assert_eq!(secrets["key_derivation_version"], serde_json::json!("1"));

        // Test that the account_key serializes correctly back to an AccountKey object
        let mut entropy_slice = [0u8; 32];
        entropy_slice[0..32].copy_from_slice(&hex::decode(&entropy).unwrap().as_slice());
        let account_key = AccountKey::from(&RootIdentity::from(&RootEntropy::from(&entropy_slice)));
        assert_eq!(
            serde_json::json!(json_rpc::account_key::AccountKey::try_from(&account_key).unwrap()),
            secrets["account_key"]
        );
    }

    #[test_with_logger]
    fn test_e2e_import_account_fog(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Import an account with fog info.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account",
            "params": {
                "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
                "key_derivation_version": "2",
                "name": "Alice Main Account",
                "first_block_index": "200",
                "fog_report_url": "fog://fog-report.example.com",
                "fog_report_id": "",
                "fog_authority_spki": "30820222300d06092a864886f70d01010105000382020f003082020a0282020100c853a8724bc211cf5370ed4dbec8947c5573bed0ec47ae14211454977b41336061f0a040f77dbf529f3a46d8095676ec971b940ab4c9642578760779840a3f9b3b893b2f65006c544e9c16586d33649769b7c1c94552d7efa081a56ad612dec932812676ebec091f2aed69123604f4888a125e04ff85f5a727c286664378581cf34c7ee13eb01cc4faf3308ed3c07a9415f98e5fbfe073e6c357967244e46ba6ebbe391d8154e6e4a1c80524b1a6733eca46e37bfdd62d75816988a79aac6bdb62a06b1237a8ff5e5c848d01bbff684248cf06d92f301623c893eb0fba0f3faee2d197ea57ac428f89d6c000f76d58d5aacc3d70204781aca45bc02b1456b454231d2f2ed4ca6614e5242c7d7af0fe61e9af6ecfa76674ffbc29b858091cbfb4011538f0e894ce45d21d7fac04ba2ff57e9ff6db21e2afd9468ad785c262ec59d4a1a801c5ec2f95fc107dc9cb5f7869d70aa84450b8c350c2fa48bddef20752a1e43676b246c7f59f8f1f4aee43c1a15f36f7a36a9ec708320ea42089991551f2656ec62ea38233946b85616ff182cf17cd227e596329b546ea04d13b053be4cf3338de777b50bc6eca7a6185cf7a5022bc9be3749b1bb43e10ecc88a0c580f2b7373138ee49c7bafd8be6a64048887230480b0c85a045255494e04a9a81646369ce7a10e08da6fae27333ec0c16c8a74d93779a9e055395078d0b07286f9930203010001"
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "mpcKQqPcgbB2oPneTAuLiZu9ZHp9qNkQo9k6949dupe89HruwmEgvcyVRFFNQccsurgMaZBykWAR1tGwbZqw4FGckqJsAcs2Fc1912Bf84S2am1kLKiRdQWfWUm6rQ8LCw75k14htjiD4u1PfYxwEvXWHXPK2R7PpzfWv5xc8129J5DykCC6wRDUZiqDcesjf7zi91frhfWvX3E6QPnc6kKZj4mfZQPjFVkHdcXWAuQoaJc");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            account_id,
            "e260179ba2bed78ed47266a55106a7365f96329203cd95edfc0915f08b7947ce"
        );

        // Export account secrets and check fog info.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "export_account_secrets",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let secrets = result.get("account_secrets").unwrap();
        let account_key = secrets.get("account_key").unwrap();

        assert_eq!(
            *account_key.get("fog_report_url").unwrap(),
            serde_json::json!("fog://fog-report.example.com")
        );
    }

    #[test_with_logger]
    fn test_e2e_get_balance(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
            42 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

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
        let balance = result.get("balance").unwrap();
        assert_eq!(
            balance
                .get("unspent_pmob")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            (42 * MOB).to_string()
        );
    }

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
        assert_eq!(status.get("total_unspent_pmob").unwrap(), "0");
        assert_eq!(status.get("total_pending_pmob").unwrap(), "0");
        assert_eq!(status.get("total_spent_pmob").unwrap(), "0");
        assert_eq!(status.get("total_orphaned_pmob").unwrap(), "0");
        assert_eq!(status.get("total_secreted_pmob").unwrap(), "0");
        assert_eq!(
            status.get("account_ids").unwrap().as_array().unwrap().len(),
            1
        );
        assert_eq!(
            status
                .get("account_map")
                .unwrap()
                .as_object()
                .unwrap()
                .len(),
            1
        );
    }

    #[test_with_logger]
    fn test_account_status(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
            42 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

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
        let balance = result.get("balance").unwrap();
        assert_eq!(
            balance
                .get("unspent_pmob")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            (42 * MOB).to_string()
        );
        let _account = result.get("account").unwrap();
    }

    #[test_with_logger]
    fn test_build_and_submit_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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

        wait_for_sync(&client, &ledger_db, &network_state, &logger);

        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        // Add a block with significantly more MOB
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            14,
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
                "value_pmob": "42000000000000", // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();
        let tx = tx_proposal.get("tx").unwrap();
        let tx_prefix = tx.get("prefix").unwrap();

        // Assert the fee is correct in both places
        let prefix_fee = tx_prefix.get("fee").unwrap().as_str().unwrap();
        let fee = tx_proposal.get("fee").unwrap();
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we
        // are not losing precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee, &MINIMUM_FEE.to_string());
        assert_eq!(fee, prefix_fee);

        // Transaction builder attempts to use as many inputs as we have txos
        let inputs = tx_proposal.get("input_list").unwrap().as_array().unwrap();
        assert_eq!(inputs.len(), 2);
        let prefix_inputs = tx_prefix.get("inputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_inputs.len(), inputs.len());

        // One destination
        let outlays = tx_proposal.get("outlay_list").unwrap().as_array().unwrap();
        assert_eq!(outlays.len(), 1);

        // Map outlay -> tx_out, should have one entry for one outlay
        let outlay_index_to_tx_out_index = tx_proposal
            .get("outlay_index_to_tx_out_index")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_index_to_tx_out_index.len(), 1);

        // Two outputs in the prefix, one for change
        let prefix_outputs = tx_prefix.get("outputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_outputs.len(), 2);

        // One outlay confirmation number for our one outlay (no receipt for change)
        let outlay_confirmation_numbers = tx_proposal
            .get("outlay_confirmation_numbers")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_confirmation_numbers.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 50 default
        // tombstone)
        let prefix_tombstone = tx_prefix.get("tombstone_block").unwrap();
        assert_eq!(prefix_tombstone, "64");

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            15,
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
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let pending = balance_status
            .get("pending_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let spent = balance_status.get("spent_pmob").unwrap().as_str().unwrap();
        let secreted = balance_status
            .get("secreted_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let orphaned = balance_status
            .get("orphaned_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, &(100000000000100 - MINIMUM_FEE).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");
    }

    #[test_with_logger]
    fn test_build_then_submit_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        // Create a tx proposal to ourselves
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value_pmob": "42",
            }
        });
        // We will fail because we cannot afford the fee
        dispatch_expect_error(
            &client,
            body,
            &logger,
            json!({
                "method": "build_transaction",
                "error": json!({
                    "code": -32603,
                    "message": "InternalError",
                    "data": json!({
                        "server_error": format!("TransactionBuilder(WalletDb(InsufficientFundsUnderMaxSpendable(\"Max spendable value in wallet: 100, but target value: {}\")))", 42 + MINIMUM_FEE),
                        "details": format!("Error building transaction: Wallet DB Error: Insufficient funds from Txos under max_spendable_value: Max spendable value in wallet: 100, but target value: {}", 42 + MINIMUM_FEE),
                    })
                }),
                "jsonrpc": "2.0",
                "id": 1,
            }).to_string(),
        );

        // Add a block with significantly more MOB
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            14,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Create a tx proposal to ourselves
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value_pmob": "42000000000000", // 42.0 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();
        let tx = tx_proposal.get("tx").unwrap();
        let tx_prefix = tx.get("prefix").unwrap();

        // Assert the fee is correct in both places
        let prefix_fee = tx_prefix.get("fee").unwrap().as_str().unwrap();
        let fee = tx_proposal.get("fee").unwrap();
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we
        // are not losing precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee, &MINIMUM_FEE.to_string());
        assert_eq!(fee, prefix_fee);

        // Transaction builder attempts to use as many inputs as we have txos
        let inputs = tx_proposal.get("input_list").unwrap().as_array().unwrap();
        assert_eq!(inputs.len(), 2);
        let prefix_inputs = tx_prefix.get("inputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_inputs.len(), inputs.len());

        // One destination
        let outlays = tx_proposal.get("outlay_list").unwrap().as_array().unwrap();
        assert_eq!(outlays.len(), 1);

        // Map outlay -> tx_out, should have one entry for one outlay
        let outlay_index_to_tx_out_index = tx_proposal
            .get("outlay_index_to_tx_out_index")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_index_to_tx_out_index.len(), 1);

        // Two outputs in the prefix, one for change
        let prefix_outputs = tx_prefix.get("outputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_outputs.len(), 2);

        // One outlay confirmation number for our one outlay (no receipt for change)
        let outlay_confirmation_numbers = tx_proposal
            .get("outlay_confirmation_numbers")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_confirmation_numbers.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 50 default
        // tombstone)
        let prefix_tombstone = tx_prefix.get("tombstone_block").unwrap();
        assert_eq!(prefix_tombstone, "64");

        // Get current balance
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);
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
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, "100000000000100");

        // Submit the tx_proposal
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
        let result = res.get("result").unwrap();
        let transaction_id = result
            .get("transaction_log")
            .unwrap()
            .get("transaction_log_id")
            .unwrap()
            .as_str()
            .unwrap();
        // Note - we cannot test here that the transaction ID is consistent, because
        // there is randomness in the transaction creation.

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            15,
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
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let pending = balance_status
            .get("pending_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let spent = balance_status.get("spent_pmob").unwrap().as_str().unwrap();
        let secreted = balance_status
            .get("secreted_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        let orphaned = balance_status
            .get("orphaned_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, "99999600000100");
        assert_eq!(pending, "0");
        assert_eq!(spent, "100000000000100");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

        // Get the transaction_id and verify it contains what we expect
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_log",
            "params": {
                "transaction_log_id": transaction_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log = result.get("transaction_log").unwrap();
        assert_eq!(
            transaction_log.get("direction").unwrap().as_str().unwrap(),
            "tx_direction_sent"
        );
        assert_eq!(
            transaction_log.get("value_pmob").unwrap().as_str().unwrap(),
            "42000000000000"
        );
        assert_eq!(
            transaction_log.get("output_txos").unwrap()[0]
                .get("recipient_address_id")
                .unwrap()
                .as_str()
                .unwrap(),
            b58_public_address
        );
        transaction_log.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            transaction_log.get("fee_pmob").unwrap().as_str().unwrap(),
            &MINIMUM_FEE.to_string()
        );
        assert_eq!(
            transaction_log.get("status").unwrap().as_str().unwrap(),
            "tx_status_succeeded"
        );
        assert_eq!(
            transaction_log
                .get("submitted_block_index")
                .unwrap()
                .as_str()
                .unwrap(),
            "14"
        );
        assert_eq!(
            transaction_log
                .get("transaction_log_id")
                .unwrap()
                .as_str()
                .unwrap(),
            transaction_id
        );

        // Get All Transaction Logs
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log_ids = result
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();
        // We have a transaction log for each of the received, as well as the sent.
        assert_eq!(transaction_log_ids.len(), 5);

        // Check the contents of the transaction log associated txos
        let transaction_log_map = result.get("transaction_log_map").unwrap();
        let transaction_log = transaction_log_map.get(transaction_id).unwrap();
        assert_eq!(
            transaction_log
                .get("output_txos")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            transaction_log
                .get("input_txos")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            transaction_log
                .get("change_txos")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );

        assert_eq!(
            transaction_log.get("status").unwrap().as_str().unwrap(),
            "tx_status_succeeded"
        );

        // Get all Transaction Logs for a given Block

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_transaction_logs_ordered_by_block",
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log_map = result
            .get("transaction_log_map")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(transaction_log_map.len(), 5);
    }

    #[test_with_logger]
    fn test_multiple_outlay_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add some accounts.
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
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let alice_public_address = b58_decode_public_address(b58_public_address).unwrap();

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
        let account_obj = result.get("account").unwrap();
        let bob_account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let bob_b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Charlie Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let charlie_account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let charlie_b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();

        // Add some money to Alice's account.
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            13,
        );

        // Create a two-output tx proposal to Bob and Charlie.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": alice_account_id,
                "addresses_and_values": [
                    [bob_b58_public_address, "42000000000000"], // 42.0 MOB
                    [charlie_b58_public_address, "43000000000000"], // 43.0 MOB
                ]
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();

        let tx_proposal = result.get("tx_proposal").unwrap();
        let tx = tx_proposal.get("tx").unwrap();
        let tx_prefix = tx.get("prefix").unwrap();

        // Assert the fee is correct in both places
        let prefix_fee = tx_prefix.get("fee").unwrap().as_str().unwrap();
        let fee = tx_proposal.get("fee").unwrap();
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we
        // are not losing precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee, &MINIMUM_FEE.to_string());
        assert_eq!(fee, prefix_fee);

        // Two destinations.
        let outlays = tx_proposal.get("outlay_list").unwrap().as_array().unwrap();
        assert_eq!(outlays.len(), 2);

        // Map outlay -> tx_out, should have one entry for one outlay
        let outlay_index_to_tx_out_index = tx_proposal
            .get("outlay_index_to_tx_out_index")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_index_to_tx_out_index.len(), 2);

        // Three outputs in the prefix, one for change
        let prefix_outputs = tx_prefix.get("outputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_outputs.len(), 3);

        // Two outlay confirmation numbers for our two outlays (no receipt for change)
        let outlay_confirmation_numbers = tx_proposal
            .get("outlay_confirmation_numbers")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_confirmation_numbers.len(), 2);

        // Get balances before submitting.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let alice_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(alice_unspent, "100000000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": bob_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let bob_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(bob_unspent, "0");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": charlie_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let charlie_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(charlie_unspent, "0");

        // Submit the tx_proposal
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_id = result
            .get("transaction_log")
            .unwrap()
            .get("transaction_log_id")
            .unwrap()
            .as_str()
            .unwrap();

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        assert_eq!(ledger_db.num_blocks().unwrap(), 14);

        // Wait for accounts to sync.
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            14,
        );
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(bob_account_id.to_string()),
            14,
        );
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(charlie_account_id.to_string()),
            14,
        );

        // Get balances after submission
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": alice_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(unspent, &(15 * MOB - MINIMUM_FEE as i64).to_string());

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": bob_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let bob_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(bob_unspent, "42000000000000");

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": charlie_account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let charlie_unspent = balance_status
            .get("unspent_pmob")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(charlie_unspent, "43000000000000");

        // Get the transaction log and verify it contains what we expect
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_log",
            "params": {
                "transaction_log_id": transaction_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let transaction_log = result.get("transaction_log").unwrap();
        assert_eq!(
            transaction_log.get("direction").unwrap().as_str().unwrap(),
            "tx_direction_sent"
        );
        assert_eq!(
            transaction_log.get("value_pmob").unwrap().as_str().unwrap(),
            "85000000000000"
        );

        let mut output_addresses: Vec<String> = transaction_log
            .get("output_txos")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                t.get("recipient_address_id")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .into()
            })
            .collect();
        output_addresses.sort();
        let mut target_addresses = vec![bob_b58_public_address, charlie_b58_public_address];
        target_addresses.sort();
        assert_eq!(output_addresses, target_addresses);

        transaction_log.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            transaction_log.get("fee_pmob").unwrap().as_str().unwrap(),
            &MINIMUM_FEE.to_string()
        );
        assert_eq!(
            transaction_log.get("status").unwrap().as_str().unwrap(),
            "tx_status_succeeded"
        );
        assert_eq!(
            transaction_log
                .get("submitted_block_index")
                .unwrap()
                .as_str()
                .unwrap(),
            "13"
        );
        assert_eq!(
            transaction_log
                .get("transaction_log_id")
                .unwrap()
                .as_str()
                .unwrap(),
            transaction_id
        );
    }

    #[test_with_logger]
    fn test_paginate_transactions(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add some transactions.
        for _ in 0..10 {
            add_block_to_ledger_db(
                &mut ledger_db,
                &vec![public_address.clone()],
                100,
                &vec![KeyImage::from(rng.next_u64())],
                &mut rng,
            );
        }

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        assert_eq!(ledger_db.num_blocks().unwrap(), 22);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            22,
        );

        // Check that we can paginate txo output.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_txos_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let txos_all = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos_all.len(), 10);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txos_for_account",
            "params": {
                "account_id": account_id,
                "offset": "2",
                "limit": "5",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let txos_page = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos_page.len(), 5);
        assert_eq!(txos_all[2..7].len(), 5);
        assert_eq!(txos_page[..], txos_all[2..7]);

        // Check that we can paginate transaction log output.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_logs_all = result
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(tx_logs_all.len(), 10);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
                "offset": "3",
                "limit": "6",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_logs_page = result
            .get("transaction_log_ids")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(tx_logs_page.len(), 6);
        assert_eq!(tx_logs_all[3..9].len(), 6);
        assert_eq!(tx_logs_page[..], tx_logs_all[3..9]);
    }

    #[test_with_logger]
    fn test_paginate_assigned_addresses(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();

        // Assign some addresses.
        for _ in 0..10 {
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
        }

        // Check that we can paginate address output.
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_addresses_for_account",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let addresses_all = result.get("public_addresses").unwrap().as_array().unwrap();
        assert_eq!(addresses_all.len(), 12); // Accounts start with 2 addresses, then we created 10.

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_addresses_for_account",
            "params": {
                "account_id": account_id,
                "offset": "1",
                "limit": "4",
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let addresses_page = result.get("public_addresses").unwrap().as_array().unwrap();
        assert_eq!(addresses_page.len(), 4);
        assert_eq!(addresses_page[..], addresses_all[1..5]);
    }

    #[test_with_logger]
    fn test_import_account_with_next_subaddress_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

        // create an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();

        // assign next subaddress for account
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
            &vec![public_address],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

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
        let balance = result.get("balance").unwrap();
        let unspent_pmob = balance.get("unspent_pmob").unwrap().as_str().unwrap();

        assert_eq!("100000000000000", unspent_pmob);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "remove_account",
            "params": {
                "account_id": account_id,
            }
        });
        dispatch(&client, body, &logger);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
            }
        });
        dispatch(&client, body, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );
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
        let balance = result.get("balance").unwrap();
        let unspent_pmob = balance.get("unspent_pmob").unwrap().as_str().unwrap();
        let orphaned_pmob = balance.get("orphaned_pmob").unwrap().as_str().unwrap();
        let spent_pmob = balance.get("spent_pmob").unwrap().as_str().unwrap();

        assert_eq!("0", unspent_pmob);
        assert_eq!("100000000000000", orphaned_pmob);
        assert_eq!("0", spent_pmob);

        // assign next subaddress for account
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
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance = result.get("balance").unwrap();
        let unspent_pmob = balance.get("unspent_pmob").unwrap().as_str().unwrap();
        let orphaned_pmob = balance.get("orphaned_pmob").unwrap().as_str().unwrap();

        assert_eq!("100000000000000", unspent_pmob);
        assert_eq!("0", orphaned_pmob);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "remove_account",
            "params": {
                "account_id": account_id,
            }
        });
        dispatch(&client, body, &logger);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "import_account_from_legacy_root_entropy",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
                "next_subaddress_index": "3",
            }
        });
        dispatch(&client, body, &logger);

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

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
        let balance = result.get("balance").unwrap();
        let unspent_pmob = balance.get("unspent_pmob").unwrap().as_str().unwrap();
        let orphaned_pmob = balance.get("orphaned_pmob").unwrap().as_str().unwrap();

        assert_eq!("100000000000000", unspent_pmob);
        assert_eq!("0", orphaned_pmob);
    }

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
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
                .unwrap(),
            0
        );
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address_1],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_account_sync(
            &ledger_db,
            &wallet_db,
            &AccountID(account_id_1.to_string()),
            13,
        );
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
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

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_account_sync(
            &ledger_db,
            &wallet_db,
            &AccountID(account_id_2.to_string()),
            14,
        );
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
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
        assert_eq!(result["removed"].as_bool().unwrap(), true,);
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
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

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_account_sync(
            &ledger_db,
            &wallet_db,
            &AccountID(account_id_3.to_string()),
            15,
        );
        assert_eq!(
            txos::table
                .select(count(txos::txo_id_hex))
                .first::<i64>(&wallet_db.get_conn().unwrap())
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

    #[test_with_logger]
    fn test_create_assigned_subaddress(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
        let account_id = result
            .get("account")
            .unwrap()
            .get("account_id")
            .unwrap()
            .as_str()
            .unwrap();

        // Create a subaddress
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "comment": "For Bob",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let b58_public_address = result
            .get("address")
            .unwrap()
            .get("public_address")
            .unwrap()
            .as_str()
            .unwrap();
        let from_bob_public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add a block to the ledger with a transaction "From Bob"
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![from_bob_public_address],
            42000000000000,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_txos_for_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let txos = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos.len(), 1);
        let txo_map = result.get("txo_map").unwrap().as_object().unwrap();
        let txo = &txo_map.get(txos[0].as_str().unwrap()).unwrap();
        let status_map = txo
            .get("account_status_map")
            .unwrap()
            .as_object()
            .unwrap()
            .get(account_id)
            .unwrap();
        let txo_status = status_map.get("txo_status").unwrap().as_str().unwrap();
        assert_eq!(txo_status, TXO_STATUS_UNSPENT);
        let txo_type = status_map.get("txo_type").unwrap().as_str().unwrap();
        assert_eq!(txo_type, TXO_TYPE_RECEIVED);
        let value = txo.get("value_pmob").unwrap().as_str().unwrap();
        assert_eq!(value, "42000000000000");
    }

    #[test_with_logger]
    fn test_get_address_for_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
        let account_id = result
            .get("account")
            .unwrap()
            .get("account_id")
            .unwrap()
            .as_str()
            .unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_for_account",
            "params": {
                "account_id": account_id,
                "index": 2,
            }
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();
        let code = error.get("code").unwrap();
        assert_eq!(code, -32603);

        // Create a subaddress
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "comment": "test",
            }
        });
        dispatch(&client, body, &logger);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_for_account",
            "params": {
                "account_id": account_id,
                "index": 2,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let address = result.get("address").unwrap();
        let subaddress_index = address.get("subaddress_index").unwrap().as_str().unwrap();

        assert_eq!(subaddress_index, "2");
    }

    #[test_with_logger]
    fn test_verify_address(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "verify_address",
            "params": {
                "address": "NOTVALIDB58",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"]["verified"].as_bool().unwrap();
        assert!(!result);

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
        let b58_public_address = res["result"]["account"]["main_address"].as_str().unwrap();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "verify_address",
            "params": {
                "address": b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"]["verified"].as_bool().unwrap();
        assert!(result);
    }

    #[test_with_logger]
    fn test_balance_for_address(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let res = dispatch(&client, body, &logger);
        let account_id = res["result"]["account"]["account_id"].as_str().unwrap();
        let b58_public_address = res["result"]["account"]["main_address"].as_str().unwrap();

        let alice_public_address = b58_decode_public_address(&b58_public_address)
            .expect("Could not b58_decode public address");
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address],
            42 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
            "method": "get_balance_for_address",
            "params": {
                "address": b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let balance = res["result"]["balance"].clone();
        assert_eq!(
            balance["unspent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            42 * MOB as u64
        );
        assert_eq!(
            balance["pending_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );
        assert_eq!(
            balance["spent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );
        assert_eq!(
            balance["secreted_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );
        assert_eq!(
            balance["orphaned_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            0
        );

        // Create a subaddress
        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "comment": "For Bob",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let from_bob_b58_public_address = result
            .get("address")
            .unwrap()
            .get("public_address")
            .unwrap()
            .as_str()
            .unwrap();
        let from_bob_public_address =
            b58_decode_public_address(from_bob_b58_public_address).unwrap();

        // Add a block to the ledger with a transaction "From Bob"
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![from_bob_public_address],
            64 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            14,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
            "method": "get_balance_for_address",
            "params": {
                "address": from_bob_b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let balance = res["result"]["balance"].clone();
        assert_eq!(
            balance["unspent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            64 * MOB as u64
        );
    }

    /// This test is intended to make sure that when a subaddress is assigned
    /// that it correctly generates and checks the key image against the ledger
    /// db to see if the previously orphaned txo has been spent or not
    #[test_with_logger]
    fn test_mark_orphaned_txo_as_spent(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            500000000000000, // 500.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            14,
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
        assert_eq!(result["removed"].as_bool().unwrap(), true,);

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

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            14,
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
        assert_eq!(result["removed"].as_bool().unwrap(), true,);

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

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            15,
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
        assert_eq!(result["removed"].as_bool().unwrap(), true,);

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

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            15,
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
    fn test_get_all_txos(logger: Logger) {
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
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_txos_for_account",
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
        assert_eq!(txo_status, TXO_STATUS_UNSPENT);
        let txo_type = account_status_map
            .get("txo_type")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_type, TXO_TYPE_RECEIVED);
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
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            13,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_txos_for_account",
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
        assert_eq!(txo_status, TXO_STATUS_UNSPENT);
        let txo_type = account_status_map
            .get("txo_type")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_type, TXO_TYPE_RECEIVED);
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

        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            14,
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

    #[test_with_logger]
    fn test_receipts(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            13,
        );

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
        let bob_b58_public_address = bob_account_obj
            .get("main_address")
            .unwrap()
            .as_str()
            .unwrap();

        // Construct a transaction proposal from Alice to Bob
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "build_transaction",
            "params": {
                "account_id": alice_account_id,
                "recipient_public_address": bob_b58_public_address,
                "value_pmob": "42000000000000", // 42 MOB
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let tx_proposal = result.get("tx_proposal").unwrap();

        // Get the receipts from the tx_proposal
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_receiver_receipts",
            "params": {
                "tx_proposal": tx_proposal
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let receipts = result["receiver_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 1);
        let receipt = &receipts[0];

        // Bob checks status (should be pending before the block is added to the ledger)
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check_receiver_receipt_status",
            "params": {
                "address": bob_b58_public_address,
                "receiver_receipt": receipt,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let status = result["receipt_transaction_status"].as_str().unwrap();
        assert_eq!(status, "TransactionPending");

        // Add the block to the ledger with the tx proposal
        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            14,
        );
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(bob_account_id.to_string()),
            14,
        );

        // Bob checks status (should be successful after added to the ledger)
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check_receiver_receipt_status",
            "params": {
                "address": bob_b58_public_address,
                "receiver_receipt": receipt,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let status = result["receipt_transaction_status"].as_str().unwrap();
        assert_eq!(status, "TransactionSuccess");
    }

    #[test_with_logger]
    fn test_gift_codes(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, network_state) = setup(&mut rng, logger.clone());

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
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            13,
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
        let json_tx_proposal: json_rpc::tx_proposal::TxProposal =
            serde_json::from_value(tx_proposal.clone()).unwrap();
        let payments_tx_proposal =
            mc_mobilecoind::payments::TxProposal::try_from(&json_tx_proposal).unwrap();

        // The MockBlockchainConnection does not write to the ledger_db
        add_block_with_tx_proposal(&mut ledger_db, payments_tx_proposal);
        wait_for_sync(&client, &ledger_db, &network_state, &logger);
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(alice_account_id.to_string()),
            14,
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
        wait_for_account_sync(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(bob_account_id.to_string()),
            14,
        );

        // Get all the gift codes in the wallet
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_all_gift_codes",
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
            "method": "get_all_gift_codes",
        });
        let res = dispatch(&client, body, &logger);
        let result = res["result"]["gift_codes"].as_array().unwrap();
        assert_eq!(result.len(), 0);
    }
}
