// Copyright (c) 2020 MobileCoin Inc.

use crate::error::WalletAPIError;
use crate::service_decorated_types::{
    JsonAccount, JsonAddress, JsonBalanceResponse, JsonListTxosResponse, JsonSubmitResponse,
    JsonTransactionResponse, JsonTxo,
};
use crate::service_impl::WalletService;
use mc_connection::ThickClient;
use mc_connection::UserTxConnection;
use mc_fog_report_connection::FogPubkeyResolver;
use mc_fog_report_connection::GrpcFogPubkeyResolver;
use mc_mobilecoind_json::data_types::JsonTxProposal;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct WalletState<
    T: UserTxConnection + 'static,
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
    get_txo {
        account_id: String,
        txo_id: String,
    },
    get_balance {
        account_id: String,
    },
    create_address {
        account_id: String,
        comment: Option<String>,
    },
    list_addresses {
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
    },
    list_transactions {
        account_id: String,
    },
    get_transaction {
        transaction_id: String,
    },
    // get_transaction_object {
    //     transaction_id: String,
    //     transaction_contents: JsonTx,
    // }
}
#[derive(Deserialize, Serialize, Debug)]
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
        accounts: Vec<JsonAccount>,
    },
    get_account {
        account: JsonAccount,
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
    get_txo {
        txo: JsonTxo,
    },
    get_balance {
        status: JsonBalanceResponse,
    },
    create_address {
        address: JsonAddress,
    },
    list_addresses {
        addresses: Vec<JsonAddress>,
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
    list_transactions {
        transactions: Vec<JsonTransactionResponse>,
    },
    get_transaction {
        transaction: JsonTransactionResponse,
    },
}

#[get("/wallet")]
fn wallet_help() -> Result<String, String> {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequest::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    Ok(help_str.to_string())
}

#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<WalletState<ThickClient, GrpcFogPubkeyResolver>>,
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
            account: state.service.get_account(&account_id)?,
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
        JsonCommandRequest::get_txo { account_id, txo_id } => {
            let result = state.service.get_txo(&account_id, &txo_id)?;
            JsonCommandResponse::get_txo { txo: result }
        }
        JsonCommandRequest::get_balance { account_id } => JsonCommandResponse::get_balance {
            status: state.service.get_balance(&account_id)?,
        },
        JsonCommandRequest::create_address {
            account_id,
            comment,
        } => JsonCommandResponse::create_address {
            address: state
                .service
                .create_assigned_subaddress(&account_id, comment.as_deref())?,
        },
        JsonCommandRequest::list_addresses { account_id } => JsonCommandResponse::list_addresses {
            addresses: state.service.list_assigned_subaddresses(&account_id)?,
        },
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
            let transaction_details = state.service.send_transaction(
                &account_id,
                &recipient_public_address,
                value,
                input_txo_ids.as_ref(),
                fee,
                tombstone_block,
                max_spendable_value,
                comment,
            )?;
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
            let tx_proposal = state.service.build_transaction(
                &account_id,
                &recipient_public_address,
                value,
                input_txo_ids.as_ref(),
                fee,
                tombstone_block,
                max_spendable_value,
            )?;
            JsonCommandResponse::build_transaction {
                tx_proposal: tx_proposal.into(),
            }
        }
        JsonCommandRequest::submit_transaction {
            tx_proposal,
            comment,
        } => JsonCommandResponse::submit_transaction {
            transaction: state.service.submit_transaction(tx_proposal, comment)?,
        },
        JsonCommandRequest::list_transactions { account_id } => {
            JsonCommandResponse::list_transactions {
                transactions: state.service.list_transactions(&account_id)?,
            }
        }
        JsonCommandRequest::get_transaction { transaction_id } => {
            JsonCommandResponse::get_transaction {
                transaction: state.service.get_transaction(&transaction_id)?,
            }
        }
    };
    Ok(Json(result))
}

pub fn rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient, GrpcFogPubkeyResolver>,
) -> rocket::Rocket {
    // FIXME: Note that if state has different type parameters, it throws an error that you are
    // requesting unmanaged state. This is an issue in tests, where we want to use mock
    // connections. For now, I am simply not testing the endpoints like submit_transaction,
    // and I am not building test transactions with a fog recipients.
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api, wallet_help])
        .manage(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_impl::b58_decode;
    use crate::test_utils::{
        add_block_to_ledger_db, get_test_ledger, setup_grpc_peer_manager, WalletDbTestContext,
    };
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
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
    use std::time::Duration;

    fn get_free_port() -> u16 {
        static PORT_NR: AtomicUsize = AtomicUsize::new(0);
        PORT_NR.fetch_add(1, SeqCst) as u16 + 30300
    }

    fn setup(mut rng: &mut StdRng, logger: Logger) -> (Client, LedgerDB) {
        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let peer_manager = setup_grpc_peer_manager(logger.clone());

        let service = WalletService::new(
            wallet_db,
            ledger_db.clone(),
            peer_manager,
            None,
            None,
            logger,
        );

        let rocket_config: rocket::Config =
            rocket::Config::build(rocket::config::Environment::Development)
                .port(get_free_port())
                .unwrap();
        let rocket = rocket(rocket_config, WalletState { service });
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
        assert_eq!(accounts[0].get("account_id").unwrap(), &account_id.clone());

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
        assert_eq!(result.get("success").unwrap(), true);

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
        std::thread::sleep(Duration::from_secs(2));

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
        let balance_status = result.get("status").unwrap();
        let unspent = balance_status.get("unspent").unwrap().as_str().unwrap();
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
        let account_id = result.get("account_id").unwrap().as_str().unwrap();
        let b58_public_address = result.get("public_address").unwrap().as_str().unwrap();
        let public_address = b58_decode(b58_public_address);

        // Add a block with a txo for this address (note that value is smaller than MINIMUM_FEE)
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![public_address.clone()],
            100,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(2));

        // Create a tx proposal to ourselves
        let body = json!({
            "method": "build_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": b58_public_address,
                "value": "42",
            }
        });
        dispatch_expect_error(
            &client,
            body,
            &logger,
            "WalletService(TransactionBuilder(InsufficientFunds(\"Cannot make change for value 100\")))".to_string(),
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
        std::thread::sleep(Duration::from_secs(2));

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
        // FIXME: Note, minimum fee does not fit into i32 - need to make sure we are not losing
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
        let (client, mut ledger_db) = setup(&mut rng, logger.clone());

        // Add an account
        let body = json!({
            "method": "create_account",
            "params": {
                "name": "Alice Main Account",
            }
        });
        let result = dispatch(&client, body, &logger);
        let account_id = result.get("account_id").unwrap().as_str().unwrap();

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
            .get("public_address_b58")
            .unwrap()
            .as_str()
            .unwrap();
        let from_bob_public_address = b58_decode(b58_public_address);

        // Add a block to the ledger with a transaction "From Bob"
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![from_bob_public_address],
            42000000000000,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo
        std::thread::sleep(Duration::from_secs(2));

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
        assert_eq!(value, "42000000000000");
    }
}
