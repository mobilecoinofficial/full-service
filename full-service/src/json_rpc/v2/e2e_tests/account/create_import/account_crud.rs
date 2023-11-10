// Copyright (c) 2020-2022 MobileCoin Inc.

//! End-to-end tests for the Full Service Wallet API.

#[cfg(test)]
mod e2e_account {
    use crate::json_rpc::v2::api::test_utils::{dispatch, setup};

    use mc_common::logger::{test_with_logger, Logger};

    use rand::{rngs::StdRng, SeedableRng};
    use serde_json::json;

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
        assert!(account_obj.get("id").is_some());
        assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
        assert!(account_obj.get("main_address").is_some());
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
        assert_eq!(account_obj.get("recovery_mode").unwrap(), false);
        assert_eq!(account_obj.get("fog_enabled").unwrap(), false);

        let account_id = account_obj.get("id").unwrap();

        // Read Accounts via Get All
        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_accounts",
            "params": {}
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
                .get("id")
                .unwrap(),
            &account_id.clone()
        );

        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_account_status",
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
            "method": "get_account_status",
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
        assert!(result["removed"].as_bool().unwrap(),);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "get_accounts",
            "params": {}
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
                "fog_info": {
                    "report_url": "fog://fog-report.example.com",
                    "report_id": "",
                    "authority_spki": "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0Ifnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
                }
            },
        });

        let res = dispatch(&client, body, &logger);
        assert_eq!(res.get("jsonrpc").unwrap(), "2.0");

        let result = res.get("result").unwrap();
        let account_obj = result.get("account").unwrap();
        assert!(account_obj.get("id").is_some());
        assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
        assert_eq!(account_obj.get("recovery_mode").unwrap(), false);
        assert!(account_obj.get("main_address").is_some());
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "1");
        assert_eq!(account_obj.get("fog_enabled").unwrap(), true);
    }
}
