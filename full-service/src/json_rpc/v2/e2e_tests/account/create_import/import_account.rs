// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::api::test_utils::{dispatch, setup},
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
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
        assert_eq!(
            account_id,
            "7872edf0d4094643213aabc92aa0d07379cfb58eda0722b21a44868f22f75b4e"
        );

        assert_eq!(
            *account_obj.get("first_block_index").unwrap(),
            serde_json::json!("200")
        );
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), false);
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
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
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
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), false);
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
                "fog_info": {
                    "report_url": "fog://fog-report.example.com",
                    "report_id": "",
                    "authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
                }
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "2kD4vRp3DaBdRrNLNhJ5BKf5FsZxcAijoMt5pxjJpbk5jQRubngUXnd92vuXWkFyezuLgjCiKu4JHjpjNCnmzf1gAdW6PbqXsecQtp8Qr8uoeeDKrd1a5PtA6apXuDVtnrKsDCcHiJqdeSt3bRsPBvkBP4JqpGyAeKFsC7s2LQwuZ88BxFe2kyeZp5G3zENfvLaMripxTKkWGDopok2LCyA9NiCDf1vwjA5opLU7eqaRfh9");
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
        assert_eq!(
            account_id,
            "0b8a95253a7d57faf8510d8092ab55fb8610a9d691a7fa3bfafbf49945b845a2"
        );

        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "1");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), true);
    }

    #[test_with_logger]
    fn test_e2e_import_account_legacy_fog(logger: Logger) {
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
                "fog_info": {
                    "report_url": "fog://fog-report.example.com",
                    "report_id": "",
                    "authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
                }
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "d3FhtyUQDYJFpEmzoXmRtF9VA5FTLycgQBKf1JEJJj8K6UXCuwzGD2uVYw1cxzZpbSivZLSxf9nZpMgUnuRxSpJA9qCDpDZd2qtc7j2N2x4758dQ91jrSCxzyuR1aJR7zgdcgdF2KwSShUhQ5n7M9uebf2HqiCWt8vttqESJ7aRNDwiW8TVmeKWviWunzYG46c8vo4DeZYK4wFfLNdwmeSn9HXKkQVpNgzsMz87cKpHRnzn");
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
        // Catches if a change results in changed accounts_ids, which should always be
        // made to be backward compatible.
        assert_eq!(
            account_id,
            "9111a17691a1eecb85bbeaa789c69471e7c8b9789e0068de02204f9d7264263d"
        );
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "1");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), true);
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
        let account_id = account_obj.get("id").unwrap().as_str().unwrap();
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
        assert!(result["removed"].as_bool().unwrap());

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
    fn test_import_account_with_next_subaddress_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());

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
        let orphaned = balance_mob["orphaned"].as_str().unwrap();
        let spent = balance_mob["spent"].as_str().unwrap();

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
}
