// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::{
        db::{account::AccountID, txo::TxoStatus},
        json_rpc,
        json_rpc::v2::api::test_utils::{dispatch, setup},
        test_utils::{add_block_to_ledger_db, manually_sync_account, MOB},
        util::b58::b58_decode_public_address,
    };
    use bip39::{Language, Mnemonic};
    use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
    use mc_account_keys_slip10::Slip10Key;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;

    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    use std::convert::TryFrom;

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
            serde_json::json!(json_rpc::v2::models::account_key::AccountKey::try_from(
                &account_key
            )
            .unwrap()),
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
            serde_json::json!(json_rpc::v2::models::account_key::AccountKey::try_from(
                &account_key
            )
            .unwrap()),
            secrets["account_key"]
        );
    }

    #[test_with_logger]
    fn test_account_status(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
            42 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
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
        assert_eq!(unspent, (42 * MOB).to_string());
        let _account = result.get("account").unwrap();
    }

    #[test_with_logger]
    fn test_e2e_get_balance(logger: Logger) {
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
            42 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
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
        assert_eq!(unspent, (42 * MOB).to_string());
        // assert_eq!(
        //     balance
        //         .get("max_spendable_pmob")
        //         .unwrap()
        //         .as_str()
        //         .unwrap()
        //         .to_string(),
        //     (42 * MOB - Mob::MINIMUM_FEE).to_string()
        // );
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
            "method": "get_addresses",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let addresses_all = result.get("public_addresses").unwrap().as_array().unwrap();
        assert_eq!(addresses_all.len(), 13); // Accounts start with 3 addresses, then we created 10.

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_addresses",
            "params": {
                "account_id": account_id,
                "offset": 1,
                "limit": 4,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let addresses_page = result.get("public_addresses").unwrap().as_array().unwrap();
        assert_eq!(addresses_page.len(), 4);
        assert_eq!(addresses_page[..], addresses_all[1..5]);
    }

    #[test_with_logger]
    fn test_next_subaddress_fails_with_fog(logger: Logger) {
        use crate::db::WalletDbError::SubaddressesNotSupportedForFOGEnabledAccounts as subaddress_error;

        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Create Account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
                "fog_report_url": "fog://fog-report.example.com",
                "fog_report_id": "",
                "fog_authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
            },
        });

        let creation_res = dispatch(&client, body, &logger);
        let creation_result = creation_res.get("result").unwrap();
        let account_obj = creation_result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(creation_res.get("jsonrpc").unwrap(), "2.0");

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
        let error = res.get("error").unwrap();
        let data = error.get("data").unwrap();
        let details = data.get("details").unwrap();
        assert!(details.to_string().contains(&subaddress_error.to_string()));
    }

    #[test_with_logger]
    fn test_create_assigned_subaddress(logger: Logger) {
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

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_txos",
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
        let txo_status = txo.get("status").unwrap().as_str().unwrap();
        assert_eq!(txo_status, TxoStatus::Unspent.to_string());
        let value = txo.get("value").unwrap();
        let token_id = txo.get("token_id").unwrap();
        assert_eq!(value, "42000000000000");
        assert_eq!(token_id, "0");
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
            "method": "get_address_for_account_at_index",
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
            "method": "get_address_for_account_at_index",
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
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
            42 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        //
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
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
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        let pending = balance_mob["pending"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();
        let secreted = balance_mob["secreted"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        assert_eq!(unspent, (42 * MOB).to_string());
        assert_eq!(pending, "0");
        assert_eq!(spent, "0");
        assert_eq!(secreted, "0");
        assert_eq!(orphaned, "0");

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
            64 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        //
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
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
        let result = res.get("result").unwrap();
        let balance_per_token = result.get("balance_per_token").unwrap();
        let balance_mob = balance_per_token.get(Mob::ID.to_string()).unwrap();
        let unspent = balance_mob["unspent"].as_str().unwrap();
        assert_eq!(unspent, (64 * MOB).to_string());
    }
}
