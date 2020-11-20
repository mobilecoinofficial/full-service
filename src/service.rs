// Copyright (c) 2020 MobileCoin Inc.

use crate::error::WalletAPIError;
use crate::service_impl::create_account;
use rocket::{post, routes};
use rocket_contrib::databases::diesel;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[database("wallet_db")]
struct DbConn(diesel::SqliteConnection);

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequest {
    create_account {
        name: String,
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
    conn: DbConn,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, WalletAPIError> {
    let result = match command.0 {
        JsonCommandRequest::create_account { name, first_block } => {
            let fb = if let Some(fb) = first_block {
                Some(fb.parse::<u64>()?)
            } else {
                None
            };
            let (entropy, public_address, account_id) = create_account(&conn, name, fb)?;
            JsonCommandResponse::create_account {
                public_address,
                entropy,
                account_id,
            }
        }
    };
    Ok(Json(result))
}

pub fn rocket(rocket_config: rocket::Config /*state: State*/) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api,])
        //.manage(state) FIXME: add the connection to the grpc client as state
        .attach(DbConn::fairing())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_common::logger::{log, test_with_logger, Logger};
    use rocket::config::Value;
    use rocket::{
        http::{ContentType, Status},
        local::Client,
    };
    use rocket_contrib::json::JsonValue;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

    fn get_free_port() -> u16 {
        static PORT_NR: AtomicUsize = AtomicUsize::new(0);
        PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
    }

    fn setup() -> Client {
        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();

        // FIXME: It would be great to make a db for basically every test, so that we
        //        can run the tests in parallel. Right now, we're just using the test.db
        // Example rocket tests with DB: https://github.com/SergioBenitez/Rocket/blob/v0.4/examples/todo/src/tests.rs
        database_config.insert("url", Value::from("./src/db/test.db"));
        databases.insert("wallet_db", Value::from(database_config));

        let rocket_config: rocket::Config =
            rocket::Config::build(rocket::config::Environment::Development)
                .port(get_free_port())
                .extra("databases", databases)
                .unwrap();
        let rocket = rocket(
            rocket_config,
            // State {
            //     sql_conn: connection,
            // },
        );
        Client::new(rocket).expect("valid rocket instance")
    }

    #[test_with_logger]
    fn test_create_account(logger: Logger) {
        let client = setup();

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
        let client = setup();

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
