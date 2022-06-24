// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::{
        db::{
            account::AccountID,
            models::{TXO_STATUS_UNSPENT, TXO_TYPE_RECEIVED},
        },
        json_rpc,
        json_rpc::api_test_utils::{
            dispatch, dispatch_expect_error, dispatch_with_header,
            dispatch_with_header_expect_error, setup, setup_with_api_key,
        },
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_proposal, manually_sync_account,
            manually_sync_view_only_account, MOB,
        },
        util::b58::b58_decode_public_address,
    };
    use bip39::{Language, Mnemonic};
    use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
    use mc_account_keys_slip10::Slip10Key;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_ledger_db::Ledger;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};
    use rocket::http::{Header, Status};
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
        assert_eq!(account_obj.get("fog_enabled").unwrap(), false);

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
    fn test_e2e_create_account_with_fog(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());
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

        let res = dispatch(&client, body, &logger);
        assert_eq!(res.get("jsonrpc").unwrap(), "2.0");

        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        assert!(account_obj.get("account_id").is_some());
        assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
        assert_eq!(account_obj.get("recovery_mode").unwrap(), false);
        assert!(account_obj.get("main_address").is_some());
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "1");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), true);
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
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), false);
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
                "fog_report_url": "fog://fog-report.example.com",
                "fog_report_id": "",
                "fog_authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "2kD4vRp3DaBdRrNLNhJ5BKf5FsZxcAijoMt5pxjJpbk5jQRubngUXnd92vuXWkFyezuLgjCiKu4JHjpjNCnmzf1gAdW6PbqXsecQtp8Qr8uoeeDKrd1a5PtA6apXuDVtnrKsDCcHiJqdeSt3bRsPBvkBP4JqpGyAeKFsC7s2LQwuZ88BxFe2kyeZp5G3zENfvLaMripxTKkWGDopok2LCyA9NiCDf1vwjA5opLU7eqaRfh9");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
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
                "fog_report_url": "fog://fog-report.example.com",
                "fog_report_id": "",
                "fog_authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "d3FhtyUQDYJFpEmzoXmRtF9VA5FTLycgQBKf1JEJJj8K6UXCuwzGD2uVYw1cxzZpbSivZLSxf9nZpMgUnuRxSpJA9qCDpDZd2qtc7j2N2x4758dQ91jrSCxzyuR1aJR7zgdcgdF2KwSShUhQ5n7M9uebf2HqiCWt8vttqESJ7aRNDwiW8TVmeKWviWunzYG46c8vo4DeZYK4wFfLNdwmeSn9HXKkQVpNgzsMz87cKpHRnzn");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
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
    fn test_e2e_view_only_account_flow(logger: Logger) {
        // create normal account
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, db_ctx, _network_state) = setup(&mut rng, logger.clone());
        let wallet_db = db_ctx.get_db_instance(logger.clone());

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
        let account_id = account_obj.get("account_id").unwrap();
        let main_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let main_account_address = b58_decode_public_address(main_address).unwrap();

        // add some funds to that account
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![main_account_address],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        manually_sync_account(
            &ledger_db,
            &db_ctx.get_db_instance(logger.clone()),
            &AccountID(account_id.as_str().unwrap().to_string()),
            &logger,
        );

        // confirm that the regular account has the correct balance
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_account",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status["unspent_pmob"].as_str().unwrap();
        assert_eq!(unspent, "100000000000000");

        // export view only import package
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "export_view_only_account_package",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        assert_eq!(res.get("jsonrpc").unwrap(), "2.0");
        let result = res.get("result").unwrap();
        let request = result.get("json_rpc_request").unwrap();

        // remove regular account (can't have both view only and regular in same wallet)
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        assert_eq!(result["removed"].as_bool().unwrap(), true);

        // import vo account
        let body = json!(request);
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account = result.get("view_only_account").unwrap();
        let vo_account_id = account.get("account_id").unwrap();
        assert_eq!(vo_account_id, account_id);

        // sync the view only account
        manually_sync_view_only_account(
            &ledger_db,
            &wallet_db,
            vo_account_id.as_str().unwrap(),
            &logger,
        );

        // check balance for view only account
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_balance_for_view_only_account",
            "params": {
                "account_id": account_id,
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let balance_status = result.get("balance").unwrap();
        let unspent = balance_status["unspent_pmob"].as_str().unwrap();
        assert_eq!(unspent, "100000000000000");

        // test get
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_view_only_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account = result.get("view_only_account").unwrap();
        let vo_account_id = account.get("account_id").unwrap();
        assert_eq!(vo_account_id, account_id);

        // test update name
        let name = "Look at these coins";
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "update_view_only_account_name",
            "params": {
                "account_id": account_id,
                "name": name,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account = result.get("view_only_account").unwrap();
        let account_name = account.get("name").unwrap();
        assert_eq!(name, account_name);

        // create new subaddress request
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "create_new_subaddresses_request",
            "params": {
                "account_id": account_id,
                "num_subaddresses_to_generate": "2",
            },
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let next_index = result
            .get("next_subaddress_index")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(next_index, "2");

        // test creating unsigned tx
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "build_unsigned_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": main_address,
                "value_pmob": "50000000000000",
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let _tx = result.get("unsigned_tx").unwrap();

        // test remove
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "remove_view_only_account",
            "params": {
                "account_id": account_id,
            }
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let removed = result.get("removed").unwrap().as_bool().unwrap();
        assert!(removed);

        // test get-all
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_all_view_only_accounts",
        });
        let res = dispatch(&client, body, &logger);
        let result = res.get("result").unwrap();
        let account_ids = result.get("account_ids").unwrap().as_array().unwrap();
        assert_eq!(account_ids.len(), 0);
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
        assert_eq!(
            balance
                .get("max_spendable_pmob")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            (42 * MOB - Mob::MINIMUM_FEE).to_string()
        );
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
            "method": "get_addresses_for_account",
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
        let balance = result.get("balance").unwrap();
        let unspent_pmob = balance.get("unspent_pmob").unwrap().as_str().unwrap();
        let orphaned_pmob = balance.get("orphaned_pmob").unwrap().as_str().unwrap();

        assert_eq!("100000000000000", unspent_pmob);
        assert_eq!("0", orphaned_pmob);
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
        let balance = res["result"]["balance"].clone();
        assert_eq!(
            balance["unspent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            42 * MOB
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
        let balance = res["result"]["balance"].clone();
        assert_eq!(
            balance["unspent_pmob"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .expect("Could not parse u64"),
            64 * MOB
        );
    }
}
