// Copyright (c) 2020 MobileCoin Inc.

use crate::error::WalletAPIError;
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
    list_accounts,
    get_account {
        id: String,
    },
    update_account_name {
        id: String,
        name: String,
    },
    delete_account {
        id: String,
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
    list_accounts {
        accounts: Vec<String>,
    },
    get_account {
        name: String,
        balance: String,
    },
    update_account_name {
        success: bool,
    },
    delete_account {
        success: bool,
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
            let (entropy, public_address, account_id) = state.service.create_account(name, fb)?;
            JsonCommandResponse::create_account {
                public_address,
                entropy,
                account_id,
            }
        }
        JsonCommandRequest::list_accounts => JsonCommandResponse::list_accounts {
            accounts: state.service.list_accounts()?,
        },
        JsonCommandRequest::get_account { id } => {
            JsonCommandResponse::get_account {
                name: state.service.get_account(&id)?,
                balance: "0".to_string(), // FIXME once implemented
            }
        }
        JsonCommandRequest::update_account_name { id, name } => {
            state.service.update_account_name(&id, name)?;
            JsonCommandResponse::update_account_name { success: true }
        }
        JsonCommandRequest::delete_account { id } => {
            state.service.delete_account(&id)?;
            JsonCommandResponse::delete_account { success: true }
        }
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
    use crate::test_utils::{get_test_ledger, WalletDbTestContext};
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{log, test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};
    use rocket::{
        http::{ContentType, Status},
        local::Client,
    };
    use rocket_contrib::json::JsonValue;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

    fn get_free_port() -> u16 {
        static PORT_NR: AtomicUsize = AtomicUsize::new(0);
        PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
    }

    fn setup(logger: Logger) -> Client {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance();
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let service = WalletService::new(wallet_db, ledger_db, None, logger);

        let rocket_config: rocket::Config =
            rocket::Config::build(rocket::config::Environment::Development)
                .port(get_free_port())
                .unwrap();
        let rocket = rocket(rocket_config, State { service });
        Client::new(rocket).expect("valid rocket instance")
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
        let client = setup(logger.clone());

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
                "id": *account_id,
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
                "id": *account_id,
                "name": "Eve Main Account",
            }
        });
        let result = dispatch(&client, body, &logger);
        assert_eq!(result.get("success").unwrap(), true);

        let body = json!({
            "method": "get_account",
            "params": {
                "id": *account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let name = result.get("name").unwrap();
        assert_eq!("Eve Main Account", name.as_str().unwrap());

        // Delete Account
        let body = json!({
            "method": "delete_account",
            "params": {
                "id": *account_id,
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
    fn test_create_account_with_first_block(logger: Logger) {
        let client = setup(logger.clone());

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
}
