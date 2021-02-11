// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{
    db::account::AccountID,
    service::{
        decorated_types::{
            JsonAccount, JsonAddress, JsonBalanceResponse, JsonBlock, JsonBlockContents, JsonProof,
            JsonSubmitResponse, JsonTransactionLog, JsonTxo, JsonWalletStatus,
            StringifiedJsonTxProposal,
        },
        wallet_impl::WalletService,
    },
};
use mc_connection::{BlockchainConnection, ThickClient, UserTxConnection};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxProposal};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::iter::FromIterator;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    pub service: WalletService<T, FPR>,
}

#[derive(Deserialize, Serialize, EnumIter, Debug)]
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
    get_all_accounts,
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
    get_all_txos_by_account {
        account_id: String,
    },
    get_txo {
        txo_id: String,
    },
    get_wallet_status,
    get_balance {
        account_id: String,
    },
    create_address {
        account_id: String,
        comment: Option<String>,
    },
    get_all_addresses_by_account {
        account_id: String,
    },
    send_transaction {
        account_id: String,
        recipient_public_address: String,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    },
    build_transaction {
        account_id: String,
        recipient_public_address: String,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    },
    submit_transaction {
        tx_proposal: StringifiedJsonTxProposal,
        comment: Option<String>,
        account_id: Option<String>,
    },
    get_all_transactions_by_account {
        account_id: String,
    },
    get_transaction {
        transaction_log_id: String,
    },
    get_transaction_object {
        transaction_log_id: String,
    },
    get_txo_object {
        txo_id: String,
    },
    get_block_object {
        block_index: String,
    },
    get_proofs {
        transaction_log_id: String,
    },
    verify_proof {
        account_id: String,
        txo_id: String,
        proof: String,
    },
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
pub enum JsonCommandResponse {
    create_account {
        entropy: String,
        account: JsonAccount,
    },
    import_account {
        account: JsonAccount,
    },
    get_all_accounts {
        account_ids: Vec<String>,
        account_map: Map<String, serde_json::Value>,
    },
    get_account {
        account: JsonAccount,
    },
    update_account_name {
        account: JsonAccount,
    },
    delete_account {
        success: bool,
    },
    get_all_txos_by_account {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_txo {
        txo: JsonTxo,
    },
    get_wallet_status {
        status: JsonWalletStatus,
    },
    get_balance {
        status: JsonBalanceResponse,
    },
    create_address {
        address: JsonAddress,
    },
    get_all_addresses_by_account {
        address_ids: Vec<String>,
        address_map: Map<String, serde_json::Value>,
    },
    send_transaction {
        transaction: JsonSubmitResponse,
    },
    build_transaction {
        tx_proposal: StringifiedJsonTxProposal,
    },
    submit_transaction {
        transaction: JsonSubmitResponse,
    },
    get_all_transactions_by_account {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_transaction {
        transaction: JsonTransactionLog,
    },
    get_transaction_object {
        transaction: JsonTx,
    },
    get_txo_object {
        txo: JsonTxOut,
    },
    get_block_object {
        block: JsonBlock,
        block_contents: JsonBlockContents,
    },
    get_proofs {
        proofs: Vec<JsonProof>,
    },
    verify_proof {
        verified: bool,
    },
}

// The Wallet API inner method, which handles switching on the method enum.
//
// Note that this is structured this way so that the routes can be defined to
// take explicit Rocket state, and then pass the service to the inner method.
// This allows us to properly construct state with Mock Connection Objects in
// tests.
fn wallet_api_inner<T, FPR>(
    service: &WalletService<T, FPR>,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    let result = match command.0 {
        JsonCommandRequest::create_account { name, first_block } => {
            let fb = first_block
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;

            let result = service
                .create_account(name, fb)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::create_account {
                entropy: result.entropy,
                account: result.account,
            }
        }
        JsonCommandRequest::import_account {
            entropy,
            name,
            first_block,
        } => {
            let fb = first_block
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            let result = service
                .import_account(entropy, name, fb)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::import_account { account: result }
        }
        JsonCommandRequest::get_all_accounts => {
            let accounts = service
                .list_accounts()
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(
                accounts
                    .iter()
                    .map(|a| {
                        (
                            a.account_id.clone(),
                            serde_json::to_value(a.clone()).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );
            JsonCommandResponse::get_all_accounts {
                account_ids: accounts.iter().map(|a| a.account_id.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequest::get_account { account_id } => JsonCommandResponse::get_account {
            account: service
                .get_account(&AccountID(account_id))
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
        },
        JsonCommandRequest::update_account_name { account_id, name } => {
            let account = service
                .update_account_name(&account_id, name)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::update_account_name { account }
        }
        JsonCommandRequest::delete_account { account_id } => {
            service
                .delete_account(&account_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::delete_account { success: true }
        }
        JsonCommandRequest::get_all_txos_by_account { account_id } => {
            let txos = service
                .list_txos(&account_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo_id.clone(),
                            serde_json::to_value(t.clone()).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_all_txos_by_account {
                txo_ids: txos.iter().map(|t| t.txo_id.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_txo { txo_id } => {
            let result = service
                .get_txo(&txo_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::get_txo { txo: result }
        }
        JsonCommandRequest::get_wallet_status => {
            let result = service
                .get_wallet_status()
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::get_wallet_status { status: result }
        }
        JsonCommandRequest::get_balance { account_id } => JsonCommandResponse::get_balance {
            status: service
                .get_balance(&account_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
        },
        JsonCommandRequest::create_address {
            account_id,
            comment,
        } => JsonCommandResponse::create_address {
            address: service
                .create_assigned_subaddress(&account_id, comment.as_deref())
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
        },
        JsonCommandRequest::get_all_addresses_by_account { account_id } => {
            let addresses = service
                .list_assigned_subaddresses(&account_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            let address_map: Map<String, serde_json::Value> = Map::from_iter(
                addresses
                    .iter()
                    .map(|a| {
                        (
                            a.address_id.clone(),
                            serde_json::to_value(&(*a).clone()).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_all_addresses_by_account {
                address_ids: addresses.iter().map(|a| a.address_id.clone()).collect(),
                address_map,
            }
        }
        JsonCommandRequest::send_transaction {
            account_id,
            recipient_public_address,
            value,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
            comment,
        } => {
            let transaction_details = service
                .send_transaction(
                    &account_id,
                    &recipient_public_address,
                    value,
                    input_txo_ids.as_ref(),
                    fee,
                    tombstone_block,
                    max_spendable_value,
                    comment,
                )
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::send_transaction {
                transaction: transaction_details,
            }
        }
        JsonCommandRequest::build_transaction {
            account_id,
            recipient_public_address,
            value,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
        } => {
            let tx_proposal = service
                .build_transaction(
                    &account_id,
                    &recipient_public_address,
                    value,
                    input_txo_ids.as_ref(),
                    fee,
                    tombstone_block,
                    max_spendable_value,
                )
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::build_transaction {
                tx_proposal: StringifiedJsonTxProposal::from(&tx_proposal),
            }
        }
        JsonCommandRequest::submit_transaction {
            tx_proposal,
            comment,
            account_id,
        } => JsonCommandResponse::submit_transaction {
            transaction: service
                .submit_transaction(JsonTxProposal::from(&tx_proposal), comment, account_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
        },
        JsonCommandRequest::get_all_transactions_by_account { account_id } => {
            let transactions = service
                .list_transactions(&account_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            let transaction_log_map: Map<String, serde_json::Value> = Map::from_iter(
                transactions
                    .iter()
                    .map(|t| {
                        (
                            t.transaction_log_id.clone(),
                            serde_json::to_value(&(*t).clone()).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_all_transactions_by_account {
                transaction_log_ids: transactions
                    .iter()
                    .map(|t| t.transaction_log_id.clone())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_transaction { transaction_log_id } => {
            JsonCommandResponse::get_transaction {
                transaction: service
                    .get_transaction(&transaction_log_id)
                    .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
            }
        }
        JsonCommandRequest::get_transaction_object { transaction_log_id } => {
            JsonCommandResponse::get_transaction_object {
                transaction: service
                    .get_transaction_object(&transaction_log_id)
                    .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
            }
        }
        JsonCommandRequest::get_txo_object { txo_id } => JsonCommandResponse::get_txo_object {
            txo: service
                .get_txo_object(&txo_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
        },
        JsonCommandRequest::get_block_object { block_index } => {
            let (block, block_contents) = service
                .get_block_object(
                    block_index
                        .parse::<u64>()
                        .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
                )
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::get_block_object {
                block,
                block_contents,
            }
        }
        JsonCommandRequest::get_proofs { transaction_log_id } => JsonCommandResponse::get_proofs {
            proofs: service
                .get_proofs(&transaction_log_id)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?,
        },
        JsonCommandRequest::verify_proof {
            account_id,
            txo_id,
            proof,
        } => {
            let result = service
                .verify_proof(&account_id, &txo_id, &proof)
                .map_err(|e| format!("{{\"error\": \"{:?}\"}}", e))?;
            JsonCommandResponse::verify_proof { verified: result }
        }
    };
    Ok(Json(result))
}

#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<WalletState<ThickClient, FogResolver>>,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, String> {
    wallet_api_inner(&state.service, command)
}

#[get("/wallet")]
fn wallet_help() -> Result<String, String> {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequest::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    Ok(help_str)
}

pub fn rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api, wallet_help])
        .manage(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            b58_decode,
            models::{TXO_RECEIVED, TXO_UNSPENT},
        },
        test_utils::{
            add_block_to_ledger_db, get_resolver_factory, get_test_ledger,
            setup_peer_manager_and_network_state, WalletDbTestContext,
        },
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{log, test_with_logger, Logger};
    use mc_connection_test_utils::MockBlockchainConnection;
    use mc_crypto_rand::rand_core::RngCore;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::LedgerDB;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use rocket::{
        http::{ContentType, Status},
        local::Client,
    };
    use rocket_contrib::json::JsonValue;
    use std::{
        sync::atomic::{AtomicUsize, Ordering::SeqCst},
        time::Duration,
    };

    fn get_free_port() -> u16 {
        static PORT_NR: AtomicUsize = AtomicUsize::new(0);
        PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
    }

    pub struct TestWalletState {
        pub service: WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
    }

    #[post("/wallet", format = "json", data = "<command>")]
    fn test_wallet_api(
        state: rocket::State<TestWalletState>,
        command: Json<JsonCommandRequest>,
    ) -> Result<Json<JsonCommandResponse>, String> {
        wallet_api_inner(&state.service, command)
    }

    fn test_rocket(rocket_config: rocket::Config, state: TestWalletState) -> rocket::Rocket {
        rocket::custom(rocket_config)
            .mount("/", routes![test_wallet_api, wallet_help])
            .manage(state)
    }

    fn setup(mut rng: &mut StdRng, logger: Logger) -> (Client, LedgerDB) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let (peer_manager, network_state) =
            setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

        let service: WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> =
            WalletService::new(
                wallet_db,
                ledger_db.clone(),
                peer_manager,
                network_state,
                get_resolver_factory(&mut rng).unwrap(),
                None,
                logger,
            );

        let rocket_config: rocket::Config =
            rocket::Config::build(rocket::config::Environment::Development)
                .port(get_free_port())
                .unwrap();
        let rocket = test_rocket(rocket_config, TestWalletState { service });
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

    fn dispatch_expect_error(
        client: &Client,
        body: JsonValue,
        logger: &Logger,
        expected_err: String,
    ) {
        let mut res = client
            .post("/wallet")
            .header(ContentType::JSON)
            .body(body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        log::info!(logger, "Attempted dispatch got response {:?}", body);
        assert_eq!(body, expected_err);
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
        assert!(result.get("entropy").is_some());
        let account_obj = result.get("account").unwrap();
        assert!(account_obj.get("account_id").is_some());
        assert_eq!(account_obj.get("name").unwrap(), "Alice Main Account");
        assert_eq!(account_obj.get("network_height").unwrap(), "12");
        assert_eq!(account_obj.get("local_height").unwrap(), "12");
        assert_eq!(account_obj.get("account_height").unwrap(), "0");
        assert_eq!(account_obj.get("is_synced").unwrap(), false);
        assert_eq!(account_obj.get("available_pmob").unwrap(), "0");
        assert_eq!(account_obj.get("pending_pmob").unwrap(), "0");
        assert!(account_obj.get("main_address").is_some());
        assert_eq!(account_obj.get("next_subaddress_index").unwrap(), "2");
        assert_eq!(account_obj.get("recovery_mode").unwrap(), false);

        let account_id = account_obj.get("account_id").unwrap();

        // Read Accounts via List, Get
        let body = json!({
            "method": "get_all_accounts",
        });
        let result = dispatch(&client, body, &logger);
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
            "method": "get_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let name = result.get("account").unwrap().get("name").unwrap();
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
        assert_eq!(
            result.get("account").unwrap().get("name").unwrap(),
            "Eve Main Account"
        );

        let body = json!({
            "method": "get_account",
            "params": {
                "account_id": *account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let name = result.get("account").unwrap().get("name").unwrap();
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
            "method": "get_all_accounts",
        });
        let result = dispatch(&client, body, &logger);
        let accounts = result.get("account_ids").unwrap().as_array().unwrap();
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
        let account_obj = result.get("account").unwrap();
        let public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        assert_eq!(public_address, "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU");
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        assert_eq!(
            account_id,
            "f9957a9d050ef8dff9d8ef6f66daa608081e631b2d918988311613343827b779"
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
        let account_obj = result.get("account").unwrap();
        assert!(account_obj.get("main_address").is_some());
        assert!(result.get("entropy").is_some());
        assert!(account_obj.get("account_id").is_some());
    }

    #[test_with_logger]
    fn test_wallet_status(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db) = setup(&mut rng, logger.clone());

        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let _result = dispatch(&client, body, &logger);

        let body = json!({
            "method": "get_wallet_status",
        });
        let result = dispatch(&client, body, &logger);
        let status = result.get("status").unwrap();
        assert_eq!(status.get("network_height").unwrap(), "12");
        assert_eq!(status.get("local_height").unwrap(), "12");
        assert_eq!(status.get("is_synced_all").unwrap(), false);
        assert_eq!(status.get("total_available_pmob").unwrap(), "0");
        assert_eq!(status.get("total_pending_pmob").unwrap(), "0");
        assert_eq!(
            status.get("account_ids").unwrap().as_array().unwrap().len(),
            1
        );
        assert_eq!(
            status
                .get("account_map")
                .unwrap()
                .as_object()
                .unwrap()
                .len(),
            1
        );
    }

    #[test_with_logger]
    fn test_get_all_txos(logger: Logger) {
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
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address = b58_decode(b58_public_address).unwrap();

        // Add a block with a txo for this address
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo - sometimes fails at 2s
        std::thread::sleep(Duration::from_secs(4));

        let body = json!({
            "method": "get_all_txos_by_account",
            "params": {
                "account_id": account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let txos = result.get("txo_ids").unwrap().as_array().unwrap();
        assert_eq!(txos.len(), 1);
        let txo_map = result.get("txo_map").unwrap().as_object().unwrap();
        let txo = txo_map.get(txos[0].as_str().unwrap()).unwrap();
        let account_status_map = txo
            .get("account_status_map")
            .unwrap()
            .as_object()
            .unwrap()
            .get(account_id)
            .unwrap();
        let txo_status = account_status_map
            .get("txo_status")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_status, TXO_UNSPENT);
        let txo_type = account_status_map
            .get("txo_type")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(txo_type, TXO_RECEIVED);
        let value = txo.get("value_pmob").unwrap().as_str().unwrap();
        assert_eq!(value, "100");

        // Check the overall balance for the account
        let body = json!({
            "method": "get_balance",
            "params": {
                "account_id": account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let balance_status = result.get("status").unwrap();
        let unspent = balance_status.get(TXO_UNSPENT).unwrap().as_str().unwrap();
        assert_eq!(unspent, "100");
    }

    #[test_with_logger]
    fn test_build_transaction(logger: Logger) {
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
        let account_obj = result.get("account").unwrap();
        let account_id = account_obj.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = account_obj.get("main_address").unwrap().as_str().unwrap();
        let public_address = b58_decode(b58_public_address).unwrap();

        // Add a block with a txo for this address (note that value is smaller than
        // MINIMUM_FEE)
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            100,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(4));

        // Create a tx proposal to ourselves
        let body = json!({
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value": "42",
            }
        });
        // We will fail because we cannot afford the fee, which is 100000000000 pMOB
        // (.01 MOB)
        dispatch_expect_error(
            &client,
            body,
            &logger,
            "{\"error\": \"TransactionBuilder(WalletDb(InsufficientFundsUnderMaxSpendable(\"Max spendable value in wallet: 100, but target value: 10000000042\")))\"}".to_string(),
        );

        // Add a block with significantly more MOB
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address],
            100000000000000, // 100.0 MOB
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(4));

        // Create a tx proposal to ourselves
        let body = json!({
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value": "42000000000000", // 42.0 MOB
            }
        });
        let result = dispatch(&client, body, &logger);
        let tx_proposal = result.get("tx_proposal").unwrap();
        let tx = tx_proposal.get("tx").unwrap();
        let tx_prefix = tx.get("prefix").unwrap();

        // Assert the fee is correct in both places
        let prefix_fee = tx_prefix.get("fee").unwrap().as_str().unwrap();
        let fee = tx_proposal.get("fee").unwrap();
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we
        // are not losing precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee, "10000000000");
        assert_eq!(fee, prefix_fee);

        // Transaction builder attempts to use as many inputs as we have txos
        let inputs = tx_proposal.get("input_list").unwrap().as_array().unwrap();
        assert_eq!(inputs.len(), 2);
        let prefix_inputs = tx_prefix.get("inputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_inputs.len(), inputs.len());

        // One destination
        let outlays = tx_proposal.get("outlay_list").unwrap().as_array().unwrap();
        assert_eq!(outlays.len(), 1);

        // Map outlay -> tx_out, should have one entry for one outlay
        let outlay_index_to_tx_out_index = tx_proposal
            .get("outlay_index_to_tx_out_index")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_index_to_tx_out_index.len(), 1);

        // Two outputs in the prefix, one for change
        let prefix_outputs = tx_prefix.get("outputs").unwrap().as_array().unwrap();
        assert_eq!(prefix_outputs.len(), 2);

        // One outlay confirmation number for our one outlay (no receipt for change)
        let outlay_confirmation_numbers = tx_proposal
            .get("outlay_confirmation_numbers")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(outlay_confirmation_numbers.len(), 1);

        // Tombstone block = ledger height (12 to start + 2 new blocks + 50 default
        // tombstone)
        let prefix_tombstone = tx_prefix.get("tombstone_block").unwrap();
        assert_eq!(prefix_tombstone, "64");
    }

    #[test_with_logger]
    fn test_create_assigned_subaddress(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let result = dispatch(&client, body, &logger);
        let account_id = result
            .get("account")
            .unwrap()
            .get("account_id")
            .unwrap()
            .as_str()
            .unwrap();

        // Create a subaddress
        let body = json!({
            "method": "create_address",
            "params": {
                "account_id": account_id,
                "comment": "For Bob",
            }
        });
        let result = dispatch(&client, body, &logger);
        let b58_public_address = result
            .get("address")
            .unwrap()
            .get("public_address")
            .unwrap()
            .as_str()
            .unwrap();
        let from_bob_public_address = b58_decode(b58_public_address).unwrap();

        // Add a block to the ledger with a transaction "From Bob"
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![from_bob_public_address],
            42000000000000,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo - sometimes fails at 2s
        std::thread::sleep(Duration::from_secs(4));

        let body = json!({
            "method": "get_all_txos_by_account",
            "params": {
                "account_id": account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
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
        assert_eq!(txo_status, TXO_UNSPENT);
        let txo_type = status_map.get("txo_type").unwrap().as_str().unwrap();
        assert_eq!(txo_type, TXO_RECEIVED);
        let value = txo.get("value_pmob").unwrap().as_str().unwrap();
        assert_eq!(value, "42000000000000");
    }
}
