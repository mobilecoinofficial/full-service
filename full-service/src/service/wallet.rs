// Copyright (c) 2020 MobileCoin Inc.

use crate::{
    db::account::AccountID,
    service::{
        decorated_types::{
            JsonAccount, JsonAddress, JsonBalanceResponse, JsonBlock, JsonBlockContents, JsonProof,
            JsonSubmitResponse, JsonTransactionLog, JsonTxo, JsonWalletStatus,
        },
        password_manager::PasswordService,
        wallet_impl::WalletService,
    },
};
use mc_connection::{BlockchainConnection, ThickClient, UserTxConnection};
use mc_fog_report_connection::FogPubkeyResolver;
use mc_fog_report_connection::GrpcFogPubkeyResolver;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxProposal};

use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::iter::FromIterator;
use strum::{EnumMessage, IntoEnumIterator};
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
    set_password {
        password: String,
    },
    unlock {
        password: String,
    },
    change_password {
        old_password: String,
        new_password: String,
    },
    get_locked_status,
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
        tx_proposal: JsonTxProposal,
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
    set_password {
        success: bool,
    },
    unlock {
        success: bool,
    },
    change_password {
        success: bool,
    },
    get_locked_status {
        status: String,
        details: String,
    },
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
        tx_proposal: JsonTxProposal,
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

// Helper method to format displaydoc errors in json.
fn format_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> String {
    json!({"error": format!("{:?}", e), "details": e.to_string()}).to_string()
}

// The Wallet API inner method, which handles switching on the method enum.
//
// Note that this is structured this way so that the routes can be defined to take
// explicit Rocket state, and then pass the service to the inner method. This allows
// us to properly construct state with Mock Connection Objects in tests.
fn wallet_api_inner<T, FPR>(
    service: &WalletService<T, FPR>,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    let result = match command.0 {
        JsonCommandRequest::set_password { password } => {
            let success = service.set_password(password).map_err(format_error)?;
            JsonCommandResponse::set_password { success }
        }
        JsonCommandRequest::unlock { password } => {
            let success = service.unlock(password).map_err(format_error)?;
            JsonCommandResponse::unlock { success }
        }
        JsonCommandRequest::change_password {
            old_password,
            new_password,
        } => {
            let success = service
                .change_password(old_password, new_password)
                .map_err(format_error)?;
            JsonCommandResponse::change_password { success }
        }
        JsonCommandRequest::get_locked_status => {
            let status = service.get_locked_status().map_err(format_error)?;
            JsonCommandResponse::get_locked_status {
                status: format!("{:?}", status),
                details: format!(
                    "{} {}",
                    status.get_message().unwrap(),
                    status.get_detailed_message().unwrap()
                ),
            }
        }
        JsonCommandRequest::create_account { name, first_block } => {
            let fb = first_block
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let result = service.create_account(name, fb).map_err(format_error)?;
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
                .map_err(format_error)?;
            let result = service
                .import_account(entropy, name, fb)
                .map_err(format_error)?;
            JsonCommandResponse::import_account { account: result }
        }
        JsonCommandRequest::get_all_accounts => {
            let accounts = service.list_accounts().map_err(format_error)?;
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
                .map_err(format_error)?,
        },
        JsonCommandRequest::update_account_name { account_id, name } => {
            let account = service
                .update_account_name(&account_id, name)
                .map_err(format_error)?;
            JsonCommandResponse::update_account_name { account }
        }
        JsonCommandRequest::delete_account { account_id } => {
            service.delete_account(&account_id).map_err(format_error)?;
            JsonCommandResponse::delete_account { success: true }
        }
        JsonCommandRequest::get_all_txos_by_account { account_id } => {
            let txos = service.list_txos(&account_id).map_err(format_error)?;
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
            let result = service.get_txo(&txo_id).map_err(format_error)?;
            JsonCommandResponse::get_txo { txo: result }
        }
        JsonCommandRequest::get_wallet_status => {
            let result = service.get_wallet_status().map_err(format_error)?;
            JsonCommandResponse::get_wallet_status { status: result }
        }
        JsonCommandRequest::get_balance { account_id } => JsonCommandResponse::get_balance {
            status: service.get_balance(&account_id).map_err(format_error)?,
        },
        JsonCommandRequest::create_address {
            account_id,
            comment,
        } => JsonCommandResponse::create_address {
            address: service
                .create_assigned_subaddress(&account_id, comment.as_deref())
                .map_err(format_error)?,
        },
        JsonCommandRequest::get_all_addresses_by_account { account_id } => {
            let addresses = service
                .list_assigned_subaddresses(&account_id)
                .map_err(format_error)?;
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
                .map_err(format_error)?;
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
                .map_err(format_error)?;
            JsonCommandResponse::build_transaction { tx_proposal }
        }
        JsonCommandRequest::submit_transaction {
            tx_proposal,
            comment,
            account_id,
        } => JsonCommandResponse::submit_transaction {
            transaction: service
                .submit_transaction(tx_proposal, comment, account_id)
                .map_err(format_error)?,
        },
        JsonCommandRequest::get_all_transactions_by_account { account_id } => {
            let transactions = service
                .list_transactions(&account_id)
                .map_err(format_error)?;
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
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequest::get_transaction_object { transaction_log_id } => {
            JsonCommandResponse::get_transaction_object {
                transaction: service
                    .get_transaction_object(&transaction_log_id)
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequest::get_txo_object { txo_id } => JsonCommandResponse::get_txo_object {
            txo: service.get_txo_object(&txo_id).map_err(format_error)?,
        },
        JsonCommandRequest::get_block_object { block_index } => {
            let (block, block_contents) = service
                .get_block_object(block_index.parse::<u64>().map_err(format_error)?)
                .map_err(format_error)?;
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
                .map_err(format_error)?;
            JsonCommandResponse::verify_proof { verified: result }
        }
    };
    Ok(Json(result))
}

#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<WalletState<ThickClient, GrpcFogPubkeyResolver>>,
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
    state: WalletState<ThickClient, GrpcFogPubkeyResolver>,
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
            WalletDb,
        },
        test_utils::{
            add_block_to_ledger_db, get_test_ledger, setup_peer_manager_and_network_state,
            WalletDbTestContext,
        },
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{log, test_with_logger, Logger};
    use mc_connection_test_utils::MockBlockchainConnection;
    use mc_crypto_rand::rand_core::RngCore;
    use mc_fog_report_validation::MockFogPubkeyResolver;
    use mc_ledger_db::{Ledger, LedgerDB};
    use mc_ledger_sync::PollingNetworkState;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
    use rocket::{
        http::{ContentType, Status},
        local::Client,
    };
    use rocket_contrib::json::JsonValue;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering::SeqCst},
            Arc, RwLock,
        },
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

    fn setup(
        mut rng: &mut StdRng,
        logger: Logger,
    ) -> (
        Client,
        LedgerDB,
        WalletDbTestContext,
        Arc<RwLock<PollingNetworkState<MockBlockchainConnection<LedgerDB>>>>,
    ) {
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
                network_state.clone(),
                None,
                None,
                false,
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
            db_test_context,
            network_state,
        )
    }

    fn dispatch(client: &Client, request_body: JsonValue, logger: &Logger) -> serde_json::Value {
        let mut res = client
            .post("/wallet")
            .header(ContentType::JSON)
            .body(request_body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let response_body = res.body().unwrap().into_string().unwrap();
        log::info!(
            logger,
            "Attempted dispatch of {:?} got response {:?}",
            request_body,
            response_body
        );

        let res: JsonValue = serde_json::from_str(&response_body).unwrap();
        res.get("result").unwrap().clone()
    }

    fn dispatch_expect_error(
        client: &Client,
        request_body: JsonValue,
        logger: &Logger,
        expected_err: String,
    ) {
        let mut res = client
            .post("/wallet")
            .header(ContentType::JSON)
            .body(request_body.to_string())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let response_body = res.body().unwrap().into_string().unwrap();
        log::info!(
            logger,
            "Attempted dispatch of {:?} got response {:?}",
            request_body,
            response_body
        );
        let response_json: serde_json::Value = serde_json::from_str(&response_body).unwrap();
        let expected_json: serde_json::Value = serde_json::from_str(&expected_err).unwrap();
        assert_eq!(response_json, expected_json);
    }

    fn wait_for_sync(
        client: &Client,
        ledger_db: &LedgerDB,
        network_state: &Arc<RwLock<PollingNetworkState<MockBlockchainConnection<LedgerDB>>>>,
        logger: &Logger,
    ) {
        let mut count = 0;
        loop {
            // Sleep to let the sync thread process the txos with the new password
            std::thread::sleep(Duration::from_secs(1));
            // Check that syncing is working
            let body = json!({
                "method": "get_wallet_status",
            });
            let result = dispatch(&client, body, &logger);
            let status = result.get("status").unwrap();

            // Have to manually call poll() on network state to get it to update for these tests
            network_state.write().unwrap().poll();

            let is_synced_all = status.get("is_synced_all").unwrap().as_bool().unwrap();
            if is_synced_all {
                let local_height = status
                    .get("local_height")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                assert_eq!(local_height, ledger_db.num_blocks().unwrap());
                assert_eq!(
                    status
                        .get("network_height")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap(),
                    local_height
                );
                break;
            }
            count += 1;
            if count > 10 {
                panic!("Service did not sync after 10 iterations");
            }
        }
    }

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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
            "b266572c325f5f0388e4645cfa945d8527e90a11bf2182c28f62090225e73138"
        );
    }

    #[test_with_logger]
    fn test_create_account_with_first_block(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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
        let (client, _ledger_db, _db_ctx, _network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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
        let (client, mut ledger_db, _db_ctx, network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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

        wait_for_sync(&client, &ledger_db, &network_state, &logger);

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
        let (client, mut ledger_db, _db_ctx, network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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

        // Add a block with a txo for this address (note that value is smaller than MINIMUM_FEE)
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            100,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        wait_for_sync(&client, &ledger_db, &network_state, &logger);

        // Create a tx proposal to ourselves
        let body = json!({
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value": "42",
            }
        });
        // We will fail because we cannot afford the fee, which is 100000000000 pMOB (.01 MOB)
        dispatch_expect_error(
            &client,
            body,
            &logger,
            "{\"error\": \"TransactionBuilder(WalletDb(InsufficientFundsUnderMaxSpendable(\\\"Max spendable value in wallet: 100, but target value: 10000000042\\\")))\", \"details\": \"Error building transaction: Wallet DB Error: Insufficient funds from Txos under max_spendable_value: Max spendable value in wallet: 100, but target value: 10000000042\"}".to_string()
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
        wait_for_sync(&client, &ledger_db, &network_state, &logger);

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
        // FIXME: WS-9 - Note, minimum fee does not fit into i32 - need to make sure we are not losing
        //        precision with the JsonTxProposal treating Fee as number
        assert_eq!(fee.to_string(), "10000000000");
        assert_eq!(fee.to_string(), prefix_fee);

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

        // Tombstone block = ledger height (12 to start + 2 new blocks + 50 default tombstone)
        let prefix_tombstone = tx_prefix.get("tombstone_block").unwrap();
        assert_eq!(prefix_tombstone, "64");
    }

    #[test_with_logger]
    fn test_create_assigned_subaddress(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (client, mut ledger_db, _db_ctx, network_state) = setup(&mut rng, logger.clone());

        // Set password on new DB
        let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
        let body = json!({
            "method": "set_password",
            "params": {
                "password": hex::encode(&password),
            }
        });
        let result = dispatch(&client, body, &logger);
        assert!(result.get("success").unwrap().as_bool().unwrap());

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

        // Sleep to let the sync thread process the txo
        wait_for_sync(&client, &ledger_db, &network_state, &logger);

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

        let body = json!({
            "method": "get_all_addresses_by_account",
            "params": {
                "account_id": account_id,
            }
        });
        let result = dispatch(&client, body, &logger);
        let addresses = result.get("address_ids").unwrap().as_array().unwrap();
        assert_eq!(addresses.len(), 3);
    }

    #[test_with_logger]
    fn test_database_password_management(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let (
            base_url,
            db_name,
            mut ledger_db,
            from_bob_public_address,
            account_id,
            password,
            new_password,
        ) = {
            let (client, mut ledger_db, db_test_context, network_state) =
                setup(&mut rng, logger.clone());

            // Create an account should fail before password is set
            let body = json!({
                "method": "create_account",
                "params": {
                    "name": "Alice Main Account",
                }
            });
            dispatch_expect_error(
                &client,
                body,
                &logger,
                "{\"error\": \"PasswordService(DatabaseLocked)\", \"details\": \"Error with the wallet password service: Cannot perform this action without a set password or while database is locked. Please set_password or unlock first.\"}".to_string()
            );

            // Verify locked status is NeverLocked before a password has been set
            let body = json!({
                "method": "get_locked_status"
            });
            let result = dispatch(&client, body, &logger);
            assert_eq!(
                result.get("status").unwrap().as_str().unwrap(),
                "NeverLocked"
            );

            // Unlocking a never-set database should fail
            let pw_rng: StdRng = SeedableRng::from_seed([20u8; 32]);
            let password: String = pw_rng.sample_iter(&Alphanumeric).take(7).collect();
            log::info!(logger, "Unlocking DB1 with {:?}", password);
            let body = json!({
                "method": "unlock",
                "params": {
                    "password": password,
                }
            });
            dispatch_expect_error(
                &client,
                body,
                &logger,
                "{\"error\": \"Database(SetPassword)\", \"details\": \"Error interacting with the database: Must set password before continuing.\"}".to_string()
            );

            // Once we set the password, we should be able to create an account
            log::info!(logger, "Setting password for DB1 to {:?}", password);
            let body = json!({
                "method": "set_password",
                "params": {
                    "password": password,
                }
            });
            let result = dispatch(&client, body, &logger);
            assert!(result.get("success").unwrap().as_bool().unwrap());

            // Verify locked status is Unlocked once a password has been set
            let body = json!({
                "method": "get_locked_status"
            });
            let result = dispatch(&client, body, &logger);
            assert_eq!(result.get("status").unwrap().as_str().unwrap(), "Unlocked");

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
                .unwrap()
                .to_string();

            // Attempting to change password with the wrong password should fail
            let pw1_rng: StdRng = SeedableRng::from_seed([21u8; 32]);
            let wrong_password: String = pw1_rng.sample_iter(&Alphanumeric).take(7).collect();
            let pw2_rng: StdRng = SeedableRng::from_seed([22u8; 32]);
            let new_password: String = pw2_rng.sample_iter(&Alphanumeric).take(7).collect();
            log::info!(
                logger,
                "Changing password for DB1 from {:?} to {:?}",
                wrong_password,
                new_password
            );
            let body = json!({
                "method": "change_password",
                "params": {
                    "old_password": wrong_password,
                    "new_password": new_password,
                }
            });
            dispatch_expect_error(
                &client,
                body,
                &logger,
                "{\"error\": \"Database(PasswordFailed)\", \"details\": \"Error interacting with the database: Password failed\"}".to_string()
            );

            // The right old password will change the password
            log::info!(
                logger,
                "Changing password for DB1 from {:?} to {:?}",
                password,
                new_password
            );
            let body = json!({
                "method": "change_password",
                "params": {
                    "old_password": password,
                    "new_password": new_password,
                }
            });
            let result = dispatch(&client, body, &logger);
            assert!(result.get("success").unwrap().as_bool().unwrap());

            // DB should still be unlocked after password change
            let body = json!({
                "method": "get_locked_status"
            });
            let result = dispatch(&client, body, &logger);
            assert_eq!(result.get("status").unwrap().as_str().unwrap(), "Unlocked");

            // We should be able to continue interacting with our accounts
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
                &vec![from_bob_public_address.clone()],
                42000000000000,
                &vec![KeyImage::from(rng.next_u64())],
                &mut rng,
            );
            assert_eq!(ledger_db.num_blocks().unwrap(), 13);
            assert_eq!(ledger_db.num_txos().unwrap(), 61);
            // Sleep to let the sync thread process the txo
            wait_for_sync(&client, &ledger_db, &network_state, &logger);

            let body = json!({
                "method": "get_all_txos_by_account",
                "params": {
                    "account_id": account_id,
                }
            });
            let result = dispatch(&client, body, &logger);
            let txos = result.get("txo_ids").unwrap().as_array().unwrap();
            assert_eq!(txos.len(), 1);

            (
                db_test_context.base_url,
                db_test_context.db_name,
                ledger_db,
                from_bob_public_address,
                account_id,
                password,
                new_password,
            )
        };

        let newer_password = {
            // Now, if we open up a new connection to the same DB, it should be in a "locked state"
            // To simulate re-opening the DB, we can use new_from_url
            let wallet_db2 =
                WalletDb::new_from_url(&format!("{}/{}", base_url, db_name), logger.clone())
                    .unwrap();

            let (peer_manager2, network_state2) =
                setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

            let service2: WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> =
                WalletService::new(
                    wallet_db2,
                    ledger_db.clone(),
                    peer_manager2,
                    network_state2.clone(),
                    None,
                    Some(1),
                    false,
                    logger.clone(),
                );

            let rocket_config: rocket::Config =
                rocket::Config::build(rocket::config::Environment::Development)
                    .port(get_free_port())
                    .unwrap();
            let rocket = test_rocket(rocket_config, TestWalletState { service: service2 });
            let client2 = Client::new(rocket).expect("valid rocket instance");

            // Verify database is locked for this new connection to the DB
            let body = json!({
                "method": "get_locked_status"
            });
            let result = dispatch(&client2, body, &logger);
            assert_eq!(result.get("status").unwrap().as_str().unwrap(), "IsLocked");

            let body = json!({
                "method": "create_address",
                "params": {
                    "account_id": account_id,
                    "comment": "For Carol",
                }
            });
            dispatch_expect_error(
                &client2,
                body,
                &logger,
                "{\"error\": \"PasswordService(DatabaseLocked)\", \"details\": \"Error with the wallet password service: Cannot perform this action without a set password or while database is locked. Please set_password or unlock first.\"}".to_string()
            );

            log::info!(logger, "Unlocking DB2 with {:?}", password);
            // Unlocking with the old password should fail
            let body = json!({
                "method": "unlock",
                "params": {
                    "password": password,
                }
            });
            dispatch_expect_error(
                &client2,
                body,
                &logger,
                "{\"error\": \"Database(PasswordFailed)\", \"details\": \"Error interacting with the database: Password failed\"}".to_string()
            );

            log::info!(logger, "Unlocking DB2 with {:?}", new_password);
            // Unlocking with the new password should work
            let body = json!({
                "method": "unlock",
                "params": {
                    "password": new_password,
                }
            });
            let result = dispatch(&client2, body, &logger);
            assert!(result.get("success").unwrap().as_bool().unwrap());

            // Verify locked status is unlocked after unlocking
            let body = json!({
                "method": "get_locked_status"
            });
            let result = dispatch(&client2, body, &logger);
            assert_eq!(result.get("status").unwrap().as_str().unwrap(), "Unlocked");

            let body = json!({
                "method": "create_address",
                "params": {
                    "account_id": account_id,
                    "comment": "For Carol",
                }
            });
            let result = dispatch(&client2, body, &logger);
            let b58_public_address = result
                .get("address")
                .unwrap()
                .get("public_address")
                .unwrap()
                .as_str()
                .unwrap();
            let from_carol_public_address = b58_decode(b58_public_address).unwrap();

            // We should be able to get all of our txos
            let body = json!({
                "method": "get_all_txos_by_account",
                "params": {
                    "account_id": account_id,
                }
            });
            let result = dispatch(&client2, body, &logger);
            let txos = result.get("txo_ids").unwrap().as_array().unwrap();
            assert_eq!(txos.len(), 1);

            // Change password again
            let newer_password = "TestTest";
            log::info!(
                logger,
                "Changing password for DB2 from {:?} to {:?}",
                new_password,
                newer_password
            );
            let body = json!({
                "method": "change_password",
                "params": {
                    "old_password": new_password,
                    "new_password": newer_password,
                }
            });
            let result = dispatch(&client2, body, &logger);
            assert!(result.get("success").unwrap().as_bool().unwrap());

            // After the password is changed, the sync thread should have the latest password, so we
            // should be able to get our Txos. Let's add a block to the ledger with a transaction "From Bob"
            add_block_to_ledger_db(
                &mut ledger_db,
                &vec![
                    from_bob_public_address.clone(),
                    from_carol_public_address.clone(),
                ],
                42000000000000,
                &vec![KeyImage::from(rng.next_u64())],
                &mut rng,
            );
            assert_eq!(ledger_db.num_blocks().unwrap(), 14);
            assert_eq!(ledger_db.num_txos().unwrap(), 63);

            wait_for_sync(&client2, &ledger_db, &network_state2, &logger);

            let body = json!({
                "method": "get_all_txos_by_account",
                "params": {
                    "account_id": account_id,
                }
            });
            let result = dispatch(&client2, body, &logger);
            let txos = result.get("txo_ids").unwrap().as_array().unwrap();
            assert_eq!(txos.len(), 3);

            let body = json!({
                "method": "create_address",
                "params": {
                    "account_id": account_id,
                    "comment": "For Dan",
                }
            });
            let result = dispatch(&client2, body, &logger);
            let _b58_public_address = result
                .get("address")
                .unwrap()
                .get("public_address")
                .unwrap()
                .as_str()
                .unwrap();
            newer_password
        };

        {
            // Finally try unlocking with that password from a new client
            let wallet_db3 =
                WalletDb::new_from_url(&format!("{}/{}", base_url, db_name), logger.clone())
                    .unwrap();

            let (peer_manager3, network_state3) =
                setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

            let service3: WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> =
                WalletService::new(
                    wallet_db3,
                    ledger_db.clone(),
                    peer_manager3,
                    network_state3.clone(),
                    None,
                    Some(1),
                    false,
                    logger.clone(),
                );

            let rocket_config3: rocket::Config =
                rocket::Config::build(rocket::config::Environment::Development)
                    .port(get_free_port())
                    .unwrap();
            let rocket3 = test_rocket(rocket_config3, TestWalletState { service: service3 });
            let client3 = Client::new(rocket3).expect("valid rocket instance");

            // Unlocking with the new password should work
            log::info!(logger, "Unlocking DB3 with password {:?}", newer_password);
            let body = json!({
                "method": "unlock",
                "params": {
                    "password": newer_password,
                }
            });
            let result = dispatch(&client3, body, &logger);
            assert!(result.get("success").unwrap().as_bool().unwrap());

            let body = json!({
                "method": "get_all_txos_by_account",
                "params": {
                    "account_id": account_id,
                }
            });
            let result = dispatch(&client3, body, &logger);
            let txos = result.get("txo_ids").unwrap().as_array().unwrap();
            assert_eq!(txos.len(), 3);
        }
    }
}
