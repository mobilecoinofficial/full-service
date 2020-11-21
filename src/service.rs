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
    use crate::test_utils::WalletDbTestContext;
    use mc_common::logger::{log, test_with_logger, Logger};
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
        let db_test_context = WalletDbTestContext::default();
        let walletdb = db_test_context.get_db_instance();
        let service = WalletService::new(walletdb, logger);

        let rocket_config: rocket::Config =
            rocket::Config::build(rocket::config::Environment::Development)
                .port(get_free_port())
                .unwrap();
        let rocket = rocket(rocket_config, State { service });
        Client::new(rocket).expect("valid rocket instance")
    }

    #[test_with_logger]
    fn test_create_account(logger: Logger) {
        let client = setup(logger.clone());

        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let mut res = client
            .post("/wallet")
            .header(ContentType::JSON)
            .body(body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        log::info!(logger, "Attempted dispatch got response {:?}", body);

        let res_json: JsonValue = serde_json::from_str(&body).unwrap();
        let result = res_json.get("result").unwrap();
        assert!(result.get("public_address").is_some());
        assert!(result.get("entropy").is_some());
        assert!(result.get("account_id").is_some());
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
        let mut res = client
            .post("/wallet")
            .header(ContentType::JSON)
            .body(body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        log::info!(logger, "Attempted dispatch got response {:?}", body);

        let res_json: JsonValue = serde_json::from_str(&body).unwrap();
        let result = res_json.get("result").unwrap();
        assert!(result.get("public_address").is_some());
        assert!(result.get("entropy").is_some());
        assert!(result.get("account_id").is_some());
    }
}
