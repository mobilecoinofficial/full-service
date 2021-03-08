// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    db,
    db::{account::AccountID, txo::TxoID},
    json_rpc,
    json_rpc::{
        account_secrets::AccountSecrets,
        address::Address,
        balance::Balance,
        block::{Block, BlockContents},
        gift_code::GiftCode,
        json_rpc_request::{help_str_v2, JsonCommandRequest, JsonCommandRequestV2},
        json_rpc_response::{
            format_error, JsonCommandResponse, JsonCommandResponseV2, JsonRPCResponse,
        },
        proof::Proof,
        receiver_receipt::ReceiverReceipt,
        tx_proposal::TxProposal,
        txo::Txo,
        wallet_status::WalletStatus,
    },
    service,
    service::{
        account::AccountService,
        address::AddressService,
        balance::BalanceService,
        gift_code::{EncodedGiftCode, GiftCodeService},
        ledger::LedgerService,
        proof::ProofService,
        receipt::ReceiptService,
        transaction::TransactionService,
        transaction_log::TransactionLogService,
        txo::TxoService,
        WalletService,
    },
};
use mc_common::logger::global_log;
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
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
        })
    })
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
        JsonCommandRequestV2::import_account {
            entropy,
            name,
            first_block_index,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            JsonCommandResponseV2::import_account {
                account: json_rpc::account::Account::try_from(
                    &service
                        .import_account_entropy(entropy, name, fb)
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
                success: service
                    .delete_account(&AccountID(account_id))
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
        JsonCommandRequestV2::assign_address_for_account {
            account_id,
            metadata,
        } => JsonCommandResponseV2::assign_address_for_account {
            address: Address::from(
                &service
                    .assign_address_for_account(&AccountID(account_id), metadata.as_deref())
                    .map_err(format_error)?,
            ),
        },
        JsonCommandRequestV2::get_all_addresses_for_account { account_id } => {
            let addresses = service
                .get_all_addresses_for_account(&AccountID(account_id))
                .map_err(format_error)?;
            let address_map: Map<String, serde_json::Value> = Map::from_iter(
                addresses
                    .iter()
                    .map(|a| {
                        (
                            a.assigned_subaddress_b58.clone(),
                            serde_json::to_value(&(Address::from(a)))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponseV2::get_all_addresses_for_account {
                public_addresses: addresses
                    .iter()
                    .map(|a| a.assigned_subaddress_b58.clone())
                    .collect(),
                address_map,
            }
        }
        JsonCommandRequestV2::build_and_submit_transaction {
            account_id,
            recipient_public_address,
            value_pmob,
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
                    value_pmob,
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
            value_pmob,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
        } => {
            let tx_proposal = service
                .build_transaction(
                    &account_id,
                    &recipient_public_address,
                    value_pmob,
                    input_txo_ids.as_ref(),
                    fee,
                    tombstone_block,
                    max_spendable_value,
                )
                .map_err(format_error)?;
            JsonCommandResponseV2::build_transaction {
                tx_proposal: TxProposal::from(&tx_proposal),
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
        JsonCommandRequestV2::verify_address { address } => JsonCommandResponseV2::verify_address {
            verified: service.verify_address(&address).map_err(format_error)?,
        },
        JsonCommandRequestV2::get_balance_for_address { address } => {
            JsonCommandResponseV2::get_balance_for_address {
                balance: Balance::from(
                    &service
                        .get_balance_for_address(&address)
                        .map_err(format_error)?,
                ),
            }
        }
        JsonCommandRequestV2::get_all_txos_for_account { account_id } => {
            let txos = service
                .list_txos(&AccountID(account_id))
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo.txo_id_hex.clone(),
                            serde_json::to_value(Txo::from(t)).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponseV2::get_all_txos_for_account {
                txo_ids: txos.iter().map(|t| t.txo.txo_id_hex.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequestV2::get_txo { txo_id } => {
            let result = service.get_txo(&TxoID(txo_id)).map_err(format_error)?;
            JsonCommandResponseV2::get_txo {
                txo: Txo::from(&result),
            }
        }
        JsonCommandRequestV2::get_all_txos_for_address { address } => {
            let txos = service
                .get_all_txos_for_address(&address)
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo.txo_id_hex.clone(),
                            serde_json::to_value(Txo::from(t)).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponseV2::get_all_txos_for_address {
                txo_ids: txos.iter().map(|t| t.txo.txo_id_hex.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequestV2::get_proofs { transaction_log_id } => {
            JsonCommandResponseV2::get_proofs {
                proofs: service
                    .get_proofs(&transaction_log_id)
                    .map_err(format_error)?
                    .iter()
                    .map(Proof::from)
                    .collect(),
            }
        }
        JsonCommandRequestV2::verify_proof {
            account_id,
            txo_id,
            proof,
        } => {
            let result = service
                .verify_proof(&AccountID(account_id), &TxoID(txo_id), &proof)
                .map_err(format_error)?;
            JsonCommandResponseV2::verify_proof { verified: result }
        }
        JsonCommandRequestV2::get_mc_protocol_transaction { transaction_log_id } => {
            let tx = service
                .get_transaction_object(&transaction_log_id)
                .map_err(format_error)?;
            let proto_tx = mc_api::external::Tx::from(&tx);
            let json_tx = JsonTx::from(&proto_tx);
            JsonCommandResponseV2::get_mc_protocol_transaction {
                transaction: json_tx,
            }
        }
        JsonCommandRequestV2::get_mc_protocol_txo { txo_id } => {
            let tx_out = service.get_txo_object(&txo_id).map_err(format_error)?;
            let proto_txo = mc_api::external::TxOut::from(&tx_out);
            let json_txo = JsonTxOut::from(&proto_txo);
            JsonCommandResponseV2::get_mc_protocol_txo { txo: json_txo }
        }
        JsonCommandRequestV2::get_block { block_index } => {
            let (block, block_contents) = service
                .get_block_object(block_index.parse::<u64>().map_err(format_error)?)
                .map_err(format_error)?;
            JsonCommandResponseV2::get_block {
                block: Block::new(&block),
                block_contents: BlockContents::new(&block_contents),
            }
        }
        JsonCommandRequestV2::check_receiver_receipts_status {
            account_id,
            receiver_receipts,
            expected_value,
        } => {
            let receipts: Vec<service::receipt::ReceiverReceipt> = receiver_receipts
                .iter()
                .map(service::receipt::ReceiverReceipt::try_from)
                .collect::<Result<Vec<service::receipt::ReceiverReceipt>, String>>()
                .map_err(format_error)?;
            let status = service
                .check_receiver_receipts_status(
                    &AccountID(account_id),
                    &receipts,
                    expected_value.parse::<u64>().map_err(format_error)?,
                )
                .map_err(format_error)?;
            JsonCommandResponseV2::check_receiver_receipts_status {
                receipts_transaction_status: status,
            }
        }
        JsonCommandRequestV2::create_receiver_receipts { tx_proposal } => {
            let receipts = service
                .create_receiver_receipts(
                    &mc_mobilecoind::payments::TxProposal::try_from(&tx_proposal)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?;
            let json_receipts: Vec<ReceiverReceipt> = receipts
                .iter()
                .map(ReceiverReceipt::try_from)
                .collect::<Result<Vec<ReceiverReceipt>, String>>()
                .map_err(format_error)?;
            JsonCommandResponseV2::create_receiver_receipts {
                receiver_receipts: json_receipts,
            }
        }
        JsonCommandRequestV2::build_gift_code {
            account_id,
            value_pmob,
            memo,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
        } => {
            let (tx_proposal, gift_code_b58, gift_code) = service
                .build_gift_code(
                    &AccountID(account_id),
                    value_pmob.parse::<u64>().map_err(format_error)?,
                    memo,
                    input_txo_ids.as_ref(),
                    fee.map(|f| f.parse::<u64>())
                        .transpose()
                        .map_err(format_error)?,
                    tombstone_block
                        .map(|t| t.parse::<u64>())
                        .transpose()
                        .map_err(format_error)?,
                    max_spendable_value
                        .map(|m| m.parse::<u64>())
                        .transpose()
                        .map_err(format_error)?,
                )
                .map_err(format_error)?;
            JsonCommandResponseV2::build_gift_code {
                tx_proposal: TxProposal::try_from(&tx_proposal).map_err(format_error)?,
                gift_code_b58: gift_code_b58.to_string(),
                gift_code: GiftCode::from(&gift_code),
            }
        }
        JsonCommandRequestV2::get_gift_code { gift_code_b58 } => {
            JsonCommandResponseV2::get_gift_code {
                gift_code: GiftCode::from(
                    &service
                        .get_gift_code(&EncodedGiftCode(gift_code_b58))
                        .map_err(format_error)?,
                ),
            }
        }
        JsonCommandRequestV2::get_all_gift_codes {} => JsonCommandResponseV2::get_all_gift_codes {
            gift_codes: service
                .list_gift_codes()
                .map_err(format_error)?
                .iter()
                .map(GiftCode::from)
                .collect(),
        },
        JsonCommandRequestV2::check_gift_code_status { gift_code_b58 } => {
            JsonCommandResponseV2::check_gift_code_status {
                gift_code_status: service
                    .check_gift_code_status(&EncodedGiftCode(gift_code_b58))
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequestV2::claim_gift_code {
            gift_code_b58,
            account_id,
            address,
        } => {
            let (transaction_log, gift_code) = service
                .claim_gift_code(
                    &EncodedGiftCode(gift_code_b58.clone()),
                    &AccountID(account_id),
                    address,
                )
                .map_err(format_error)?;
            JsonCommandResponseV2::claim_gift_code {
                transaction_log_id: transaction_log.transaction_id_hex,
                gift_code: GiftCode::from(&gift_code),
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
