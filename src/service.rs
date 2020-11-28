// Copyright (c) 2020 MobileCoin Inc.

use crate::error::WalletAPIError;
use crate::service_decorated_types::JsonListTxosResponse;
use crate::service_impl::WalletService;
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

pub struct State {
    pub service: WalletService,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequest {
    create_account {
        name: Option<String>,
        first_block: Option<String>,
    },
    import_account {
        entropy: String,
        name: Option<String>,
        first_block: Option<String>,
    },
    list_accounts,
    get_account {
        account_id: String,
    },
    update_account_name {
        account_id: String,
        name: String,
    },
    delete_account {
        account_id: String,
    },
    list_txos {
        account_id: String,
    },
    get_balance {
        account_id: String,
    },
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
pub enum JsonCommandResponse {
    create_account {
        public_address: String,
        entropy: String,
        account_id: String,
    },
    import_account {
        public_address: String,
        account_id: String,
    },
    list_accounts {
        accounts: Vec<String>,
    },
    get_account {
        name: String,
    },
    update_account_name {
        success: bool,
    },
    delete_account {
        success: bool,
    },
    list_txos {
        txos: Vec<JsonListTxosResponse>,
    },
    get_balance {
        balance: String,
    },
}

#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<State>,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, WalletAPIError> {
    let result = match command.0 {
        JsonCommandRequest::create_account { name, first_block } => {
            let fb = if let Some(fb) = first_block {
                Some(fb.parse::<u64>()?)
            } else {
                None
            };
            // FIXME: better way to convert between the json type and enum
            let result = state.service.create_account(name, fb)?;
            JsonCommandResponse::create_account {
                public_address: result.public_address_b58,
                entropy: result.entropy,
                account_id: result.account_id,
            }
        }
        JsonCommandRequest::import_account {
            entropy,
            name,
            first_block,
        } => {
            let fb = if let Some(fb) = first_block {
                Some(fb.parse::<u64>()?)
            } else {
                None
            };
            let result = state.service.import_account(entropy, name, fb)?;
            JsonCommandResponse::import_account {
                public_address: result.public_address_b58,
                account_id: result.account_id,
            }
        }
        JsonCommandRequest::list_accounts => JsonCommandResponse::list_accounts {
            accounts: state.service.list_accounts()?,
        },
        JsonCommandRequest::get_account { account_id } => JsonCommandResponse::get_account {
            name: state.service.get_account(&account_id)?,
        },
        JsonCommandRequest::update_account_name { account_id, name } => {
            state.service.update_account_name(&account_id, name)?;
            JsonCommandResponse::update_account_name { success: true }
        }
        JsonCommandRequest::delete_account { account_id } => {
            state.service.delete_account(&account_id)?;
            JsonCommandResponse::delete_account { success: true }
        }
        JsonCommandRequest::list_txos { account_id } => JsonCommandResponse::list_txos {
            txos: state.service.list_txos(&account_id)?,
        },
        JsonCommandRequest::get_balance { account_id } => JsonCommandResponse::get_balance {
            balance: state.service.get_balance(&account_id)?.to_string(),
        },
    };
    Ok(Json(result))
}

pub fn rocket(rocket_config: rocket::Config, state: State) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api,])
        .manage(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{add_block_to_ledger_db, get_test_ledger, WalletDbTestContext};
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{log, test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_ledger_db::LedgerDB;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use rocket::{
        http::{ContentType, Status},
        local::Client,
    };
    use rocket_contrib::json::JsonValue;
    use std::convert::TryFrom;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
    use std::time::Duration;

    fn get_free_port() -> u16 {
        static PORT_NR: AtomicUsize = AtomicUsize::new(0);
        PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
    }

    // FIXME: this will probably live in db or service_impl once we're decoding public addresses
    fn b58_decode(b58_public_address: &str) -> PublicAddress {
        let wrapper = mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(
            b58_public_address.to_string(),
        )
        .unwrap();
        let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
            let payment_request = wrapper.get_payment_request();
            payment_request.get_public_address()
        } else if wrapper.has_public_address() {
            wrapper.get_public_address()
        } else {
            panic!("No public address in wrapper");
        };
        PublicAddress::try_from(pubaddr_proto).unwrap()
    }

    fn setup(mut rng: &mut StdRng, logger: Logger) -> (Client, LedgerDB) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let service = WalletService::new(wallet_db, ledger_db.clone(), None, logger);

        let rocket_config: rocket::Config =
            rocket::Config::build(rocket::config::Environment::Development)
                .port(get_free_port())
                .unwrap();
        let rocket = rocket(rocket_config, State { service });
        (
            Client::new(rocket).expect("valid rocket instance"),
            ledger_db,
        )
    }

    fn dispatch(client: &Client, body: JsonValue, logger: &Logger) -> serde_json::Value {
        let mut res = client
            .post("/wallet")
            .header(ContentType::JSON)
            .body(body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        log::info!(logger, "Attempted dispatch got response {:?}", body);

        let res: JsonValue = serde_json::from_str(&body).unwrap();
        res.get("result").unwrap().clone()
    }

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db) = setup(&mut rng, logger.clone());

        // Create Account
        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("public_address").is_some());
        assert!(result.get("entropy").is_some());
        assert!(result.get("account_id").is_some());
        let account_id = result.get("account_id").unwrap();

        // Read Accounts via List, Get
        let body = json!({
            "method": "list_accounts",
        });
        let result = dispatch(&client, body, &logger);
        let accounts = result.get("accounts").unwrap().as_array().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0], account_id.clone());

        let body = json!({
            "method": "get_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let name = result.get("name").unwrap();
        assert_eq!("Alice Main Account", name.as_str().unwrap());
        // FIXME: assert balance

        // Update Account
        let body = json!({
            "method": "update_account_name",
            "params": {
                "account_id": *account_id,
                "name": "Eve Main Account",
            }
        });
        let result = dispatch(&client, body, &logger);
        assert_eq!(result.get("success").unwrap(), true);

        let body = json!({
            "method": "get_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let name = result.get("name").unwrap();
        assert_eq!("Eve Main Account", name.as_str().unwrap());

        // Delete Account
        let body = json!({
            "method": "delete_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        assert_eq!(result.get("success").unwrap(), true);

        let body = json!({
            "method": "list_accounts",
        });
        let result = dispatch(&client, body, &logger);
        let accounts = result.get("accounts").unwrap().as_array().unwrap();
        assert_eq!(accounts.len(), 0);
    }

    #[test_with_logger]
    fn test_import_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db) = setup(&mut rng, logger.clone());

        let body = json!({
            "method": "import_account",
            "params": {
                "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
                "name": "Alice Main Account",
                "first_block": "200",
            }
        });
        let result = dispatch(&client, body, &logger);
        let public_address = result.get("public_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU");
        let account_id = result.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            account_id,
            "da150710b5fbc21432edf721b530d379fcefbf50cfca93155c47fe20bb219e48"
        );
    }

    #[test_with_logger]
    fn test_create_account_with_first_block(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db) = setup(&mut rng, logger.clone());

        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
                "first_block": "200",
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("public_address").is_some());
        assert!(result.get("entropy").is_some());
        assert!(result.get("account_id").is_some());
    }

    #[test_with_logger]
    fn test_list_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
                "first_block": "0",
            }
        });
        let result = dispatch(&client, body, &logger);
        let account_id = result.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = result.get("public_address").unwrap().as_str().unwrap();
        let public_address = b58_decode(b58_public_address);

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(3));

        let body = json!({
            "method": "list_txos",
            "params": {
                "account_id": account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let txos = result.get("txos").unwrap().as_array().unwrap();
        assert_eq!(txos.len(), 1);
        let txo = &txos[0];
        let txo_status = txo.get("txo_status").unwrap().as_str().unwrap();
        assert_eq!(txo_status, "unspent");
        let txo_type = txo.get("txo_type").unwrap().as_str().unwrap();
        assert_eq!(txo_type, "received");
        let value = txo.get("value").unwrap().as_str().unwrap();
        assert_eq!(value, "100");

        // Check the overall balance for the account
        let body = json!({
            "method": "get_balance",
            "params": {
                "account_id": account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let balance = result.get("balance").unwrap().as_str().unwrap();
        assert_eq!(balance, "100");
    }
}
