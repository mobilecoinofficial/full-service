// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    db,
    db::account::AccountID,
    json_rpc,
    json_rpc::{
        account_secrets::AccountSecrets,
        api_v1::wallet_api::{wallet_api_inner_v1, JsonCommandRequestV1},
        balance::Balance,
        json_rpc_request::{help_str_v2, JsonCommandRequest, JsonCommandRequestV2},
        json_rpc_response::{
            format_error, JsonCommandResponse, JsonCommandResponseV2, JsonRPCResponse,
        },
        wallet_status::WalletStatus,
    },
    service::{
        account::AccountService, balance::BalanceService, transaction::TransactionService,
        transaction_log::TransactionLogService, WalletService,
    },
};
use mc_common::logger::global_log;
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde_json::Map;
use std::{convert::TryFrom, iter::FromIterator};

/// State managed by rocket.
pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    /// The Wallet Service implementation.
    pub service: WalletService<T, FPR>,
}

/// The route for the Full Service Wallet API.
#[post("/wallet", format = "json", data = "<command>")]
fn wallet_api(
    state: rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
    command: Json<JsonCommandRequest>,
) -> Result<Json<JsonCommandResponse>, String> {
    let req: JsonCommandRequest = command.0.clone();

    if let Some(version) = command.0.api_version.clone() {
        wallet_api_inner_v2(
            &state.service,
            Json(JsonCommandRequestV2::try_from(&req).map_err(|e| e)?),
        )
        .map(|res| {
            Json(JsonCommandResponse {
                method: res.0.method,
                result: res.0.result,
                error: res.0.error,
                jsonrpc: Some("2.0".to_string()),
                id: command.0.id,
                api_version: Some(version),
            })
        })
    } else {
        wallet_api_inner_v1(
            &state.service,
            Json(JsonCommandRequestV1::try_from(&req).map_err(|e| e)?),
        )
        .map(|res| {
            let json_response: serde_json::Value = serde_json::json!(res.0);
            Json(JsonCommandResponse {
                method: Some(json_response.get("method").unwrap().to_string()),
                result: Some(json_response.get("result").unwrap().clone()),
                error: None,
                jsonrpc: None,
                id: None,
                api_version: None,
            })
        })
    }
}

/// The Wallet API inner method, which handles switching on the method enum.
///
/// Note that this is structured this way so that the routes can be defined to
/// take explicit Rocket state, and then pass the service to the inner method.
/// This allows us to properly construct state with Mock Connection Objects in
/// tests. This also allows us to version the overall API easily.
pub fn wallet_api_inner_v2<T, FPR>(
    service: &WalletService<T, FPR>,
    command: Json<JsonCommandRequestV2>,
) -> Result<Json<JsonRPCResponse>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    global_log::trace!("Running command {:?}", command);

    let result: JsonCommandResponseV2 = match command.0 {
        JsonCommandRequestV2::create_account {
            name,
            first_block_index,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let account: db::models::Account =
                service.create_account(name, fb).map_err(format_error)?;

            JsonCommandResponseV2::create_account {
                account: json_rpc::account::Account::try_from(&account)
                    .map_err(|e| format!("Could not get RPC Account from DB Account {:?}", e))?,
            }
        }
        JsonCommandRequestV2::import_account_by_entropy {
            entropy,
            name,
            first_block_index,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            JsonCommandResponseV2::import_account_by_entropy {
                account: json_rpc::account::Account::try_from(
                    &service
                        .import_account_entropy(entropy, name, fb)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::import_account_by_account_key {
            account_key,
            name,
            first_block_index,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            JsonCommandResponseV2::import_account_by_account_key {
                account: json_rpc::account::Account::try_from(
                    &service
                        .import_account_key(
                            mc_account_keys::AccountKey::try_from(&account_key)
                                .map_err(format_error)?,
                            name,
                            fb,
                        )
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::export_account_secrets { account_id } => {
            let account = service
                .get_account(&AccountID(account_id))
                .map_err(format_error)?;
            JsonCommandResponseV2::export_account_secrets {
                account_secrets: AccountSecrets::try_from(&account).map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::get_all_accounts => {
            let accounts = service.list_accounts().map_err(format_error)?;
            let json_accounts: Vec<(String, serde_json::Value)> = accounts
                .iter()
                .map(|a| {
                    json_rpc::account::Account::try_from(a).and_then(|v| {
                        serde_json::to_value(v)
                            .map(|v| (a.account_id_hex.clone(), v))
                            .map_err(format_error)
                    })
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, String>>()?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(json_accounts);
            JsonCommandResponseV2::get_all_accounts {
                account_ids: accounts.iter().map(|a| a.account_id_hex.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequestV2::get_account { account_id } => JsonCommandResponseV2::get_account {
            account: json_rpc::account::Account::try_from(
                &service
                    .get_account(&AccountID(account_id))
                    .map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
        JsonCommandRequestV2::update_account_name { account_id, name } => {
            JsonCommandResponseV2::update_account_name {
                account: json_rpc::account::Account::try_from(
                    &service
                        .update_account_name(&AccountID(account_id), name)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::delete_account { account_id } => {
            JsonCommandResponseV2::delete_account {
                account: json_rpc::account::Account::try_from(
                    &service
                        .delete_account(&AccountID(account_id))
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::get_balance_for_account { account_id } => {
            JsonCommandResponseV2::get_balance_for_account {
                balance: Balance::from(
                    &service
                        .get_balance_for_account(&AccountID(account_id))
                        .map_err(format_error)?,
                ),
            }
        }
        JsonCommandRequestV2::get_wallet_status => JsonCommandResponseV2::get_wallet_status {
            wallet_status: WalletStatus::try_from(
                &service.get_wallet_status().map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
        JsonCommandRequestV2::get_account_status { account_id } => {
            let account = json_rpc::account::Account::try_from(
                &service
                    .get_account(&AccountID(account_id.clone()))
                    .map_err(format_error)?,
            )
            .map_err(format_error)?;
            let balance = Balance::from(
                &service
                    .get_balance_for_account(&AccountID(account_id))
                    .map_err(format_error)?,
            );
            JsonCommandResponseV2::get_account_status { account, balance }
        }
        JsonCommandRequestV2::build_and_submit_transaction {
            account_id,
            recipient_public_address,
            value,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
            comment,
        } => {
            let (transaction_log, associated_txos) = service
                .build_and_submit(
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
            JsonCommandResponseV2::build_and_submit_transaction {
                transaction_log: json_rpc::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                ),
            }
        }
        JsonCommandRequestV2::build_transaction {
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
            JsonCommandResponseV2::build_transaction {
                tx_proposal: json_rpc::tx_proposal::TxProposal::from(&tx_proposal),
            }
        }
        JsonCommandRequestV2::submit_transaction {
            tx_proposal,
            comment,
            account_id,
        } => {
            let result: Option<json_rpc::transaction_log::TransactionLog> = service
                .submit_transaction(
                    mc_mobilecoind::payments::TxProposal::try_from(&tx_proposal)
                        .map_err(format_error)?,
                    comment,
                    account_id,
                )
                .map_err(format_error)?
                .map(|(transaction_log, associated_txos)| {
                    json_rpc::transaction_log::TransactionLog::new(
                        &transaction_log,
                        &associated_txos,
                    )
                });
            JsonCommandResponseV2::submit_transaction {
                transaction_log: result,
            }
        }
        JsonCommandRequestV2::get_all_transaction_logs_for_account { account_id } => {
            let transaction_logs_and_txos = service
                .list_transaction_logs(&AccountID(account_id))
                .map_err(format_error)?;
            let transaction_log_map: Map<String, serde_json::Value> = Map::from_iter(
                transaction_logs_and_txos
                    .iter()
                    .map(|(t, a)| {
                        (
                            t.transaction_id_hex.clone(),
                            serde_json::json!(json_rpc::transaction_log::TransactionLog::new(t, a)),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponseV2::get_all_transaction_logs_for_account {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _a)| t.transaction_id_hex.to_string())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequestV2::get_transaction_log { transaction_log_id } => {
            let (transaction_log, associated_txos) = service
                .get_transaction_log(&transaction_log_id)
                .map_err(format_error)?;
            JsonCommandResponseV2::get_transaction_log {
                transaction_log: json_rpc::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                ),
            }
        }
        JsonCommandRequestV2::get_all_transaction_logs_for_block { block_index } => {
            let transaction_logs_and_txos = service
                .get_all_transaction_logs_for_block(
                    block_index.parse::<u64>().map_err(format_error)?,
                )
                .map_err(format_error)?;
            let transaction_log_map: Map<String, serde_json::Value> = Map::from_iter(
                transaction_logs_and_txos
                    .iter()
                    .map(|(t, a)| {
                        (
                            t.transaction_id_hex.clone(),
                            serde_json::json!(json_rpc::transaction_log::TransactionLog::new(t, a)),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponseV2::get_all_transaction_logs_for_block {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _a)| t.transaction_id_hex.to_string())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequestV2::get_all_transaction_logs_ordered_by_block => {
            let transaction_logs_and_txos = service
                .get_all_transaction_logs_ordered_by_block()
                .map_err(format_error)?;
            let transaction_log_map: Map<String, serde_json::Value> = Map::from_iter(
                transaction_logs_and_txos
                    .iter()
                    .map(|(t, a)| {
                        (
                            t.transaction_id_hex.clone(),
                            serde_json::json!(json_rpc::transaction_log::TransactionLog::new(t, a)),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponseV2::get_all_transaction_logs_ordered_by_block {
                transaction_log_map,
            }
        }
    };
    let response = Json(JsonRPCResponse::from(result));
    Ok(response)
}

#[get("/wallet")]
fn wallet_help_v2() -> Result<String, String> {
    Ok(help_str_v2())
}

/// Returns an instance of a Rocker server.
pub fn rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount("/", routes![wallet_api, wallet_help_v2])
        .manage(state)
}
