use crate::data_types::*;
use crate::{create_post, publish_post};
use rocket::{post, routes};
use rocket_contrib::databases::diesel;
use rocket_contrib::json::Json;

#[database("posts_db")]
struct DbConn(diesel::SqliteConnection);

/// Connection to the database and the grpc client
// pub struct State {
//     pub sql_conn: Arc<SqliteConnection>,
// }

// #[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "method", content = "params")]
// #[allow(non_camel_case_types)]
// pub enum JsonCommandRequest {
//     check_balance { subaddress_index: u32 },
// }
// #[derive(Deserialize, Serialize)]
// #[serde(tag = "method", content = "result")]
// #[allow(non_camel_case_types)]
// pub enum JsonCommandResponse {
//     check_balance { balance: u32 },
// }

// #[post("/wallet", format = "json", data = "<command>")]
// fn wallet_api(
//     _state: rocket::State<State>,
//     command: Json<JsonCommandRequest>,
// ) -> Result<Json<JsonCommandResponse>, String> {
//     let result = JsonCommandResponse::check_balance { balance: 0 };
//     match command.0 {
//         JsonCommandRequest::check_balance { subaddress_index } => println!("{}", subaddress_index),
//     }
//     Ok(Json(result))
// }

pub fn rocket(rocket_config: rocket::Config /*state: State*/) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_create_account,])
        //.manage(state)
        .attach(DbConn::fairing())
}

#[post("/wallet/create-account", format = "json", data = "<request>")]
fn wallet_create_account(
    conn: DbConn,
    request: Json<WalletCreateAccountRequest>,
) -> Result<Json<WalletCreateAccountResponse>, String> {
    let post_id = create_post(&conn, &request.params.title, &request.params.body);
    let _update_int = publish_post(&conn, post_id);
    Ok(Json(WalletCreateAccountResponse {
        public_address: "Hello!".to_string(),
        entropy: "World!".to_string(),
        account_id: "I'm here!".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use mc_common::logger::{log, test_with_logger, Logger};
    use rocket::config::Value;
    use rocket::{
        http::{ContentType, Status},
        local::Client,
    };
    use rocket_contrib::json::JsonValue;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

    // FIXME: example rocket tests with DB: https://github.com/SergioBenitez/Rocket/blob/v0.4/examples/todo/src/tests.rs

    fn get_free_port() -> u16 {
        static PORT_NR: AtomicUsize = AtomicUsize::new(0);
        PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
    }

    fn setup() -> Client {
        dotenv().ok();
        for (k, v) in std::env::vars() {
            println!("\x1b[1;31mvars = {:?}: {:?}\x1b[0m", k, v);
        }

        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();

        // This is the same as the following TOML:
        // posts_db = { url = "./src/db/test.db" }
        database_config.insert("url", Value::from("./src/db/test.db"));
        databases.insert("posts_db", Value::from(database_config));

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
        "title": "Alice Main Account",
        "body": "Body"
        }
        });
        let mut res = client
            .post("/wallet/create-account")
            .header(ContentType::JSON)
            .body(body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        log::info!(logger, "Attempted dispatch got response {:?}", body);
        let res_json: JsonValue = serde_json::from_str(&body).unwrap();
        assert!(res_json.get("public_address").is_some());
        assert!(res_json.get("entropy").is_some());
        assert!(res_json.get("account_id").is_some());
    }
}
