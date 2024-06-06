// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::{
        db::{account::AccountID, txo::TxoStatus},
        json_rpc::v2::{
            api::test_utils::{dispatch, setup},
            models::public_address::PublicAddress,
        },
        test_utils::{add_block_to_ledger_db, manually_sync_account},
        util::b58::b58_decode_public_address,
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_ledger_db::Ledger;
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};

    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

    #[test_with_logger]
    fn test_import_account_with_next_subaddress_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

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
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();

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
        let b58_public_address = address.get("public_address_b58").unwrap().as_str().unwrap();
        let public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add a block to fund account at the new subaddress.
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100000000000000, // 100.0 MOB
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.to_string()),
            &logger,
        );
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

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

        assert_eq!("100000000000000", unspent);

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
        let spent = balance_mob["spent"].as_str().unwrap();
        let orphaned = balance_mob["orphaned"].as_str().unwrap();

        assert_eq!("0", unspent);
        assert_eq!("100000000000000", orphaned);
        assert_eq!("0", spent);

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
        let orphaned = balance_mob["orphaned"].as_str().unwrap();

        assert_eq!("100000000000000", unspent);
        assert_eq!("0", orphaned);

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
        let orphaned = balance_mob["orphaned"].as_str().unwrap();

        assert_eq!("100000000000000", unspent);
        assert_eq!("0", orphaned);
    }

    #[test_with_logger]
    fn test_paginate_assigned_addresses(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

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
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();

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
        let (client, mut _ledger_db, _db_ctx, _network_state) =
            setup(&mut rng, None, logger.clone());

        // Create Account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
                "fog_info": {
                    "report_url": "fog://fog-report.example.com",
                    "report_id": "",
                    "authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0Ifnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
                }
            },
        });

        let creation_res = dispatch(&client, body, &logger);
        let creation_result = creation_res.get("result").unwrap();
        let account_obj = creation_result.get("account").unwrap();
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
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
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

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
            .get("id")
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
            .get("public_address_b58")
            .unwrap()
            .as_str()
            .unwrap();
        let from_bob_public_address = b58_decode_public_address(b58_public_address).unwrap();

        // Add a block to the ledger with a transaction "From Bob"
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![from_bob_public_address],
            42000000000000,
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
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

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
            .get("id")
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
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

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
    fn test_get_address_details(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, None, logger.clone());

        // Add an account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_details",
            "params": {
                "address": "NOTVALIDB58",
            }
        });
        let res = dispatch(&client, body, &logger);
        let error = res.get("error").unwrap();
        let data = error.get("data").unwrap();
        assert_eq!("B58(PrintableWrapper(B58(\"provided string contained invalid character 'O' at byte 1\")))", data.get("server_error").unwrap().as_str().unwrap());

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
        let public_address = b58_decode_public_address(b58_public_address).unwrap();
        let public_address_json = PublicAddress::from(&public_address);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_address_details",
            "params": {
                "address": b58_public_address,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let details = result.get("details").unwrap();
        let public_address_details: PublicAddress =
            serde_json::from_value(details.clone()).unwrap();

        assert_eq!(public_address_details, public_address_json);
    }
}
