// Copyright (c) 2020-2021 MobileCoin Inc.

//! Entrypoint for Wallet API.

use crate::{
    db::{self, account::AccountID, transaction_log::TransactionID, txo::TxoID},
    json_rpc,
    json_rpc::{
        account_secrets::AccountSecrets,
        address::Address,
        balance::{Balance, ViewOnlyBalance},
        block::{Block, BlockContents},
        confirmation_number::Confirmation,
        gift_code::GiftCode,
        json_rpc_request::{help_str, JsonCommandRequest, JsonRPCRequest},
        json_rpc_response::{
            format_error, format_invalid_request_error, JsonCommandResponse, JsonRPCError,
            JsonRPCResponse,
        },
        network_status::NetworkStatus,
        receiver_receipt::ReceiverReceipt,
        tx_proposal::TxProposal,
        txo::Txo,
        view_only_txo::ViewOnlyTxo,
        wallet_status::WalletStatus,
    },
    service,
    service::{
        account::AccountService,
        address::AddressService,
        balance::BalanceService,
        confirmation_number::ConfirmationService,
        gift_code::{EncodedGiftCode, GiftCodeService},
        ledger::LedgerService,
        payment_request::PaymentRequestService,
        receipt::ReceiptService,
        transaction::TransactionService,
        transaction_log::TransactionLogService,
        txo::TxoService,
        view_only_account::ViewOnlyAccountService,
        view_only_txo::ViewOnlyTxoService,
        WalletService,
    },
    util::{
        b58::{
            b58_decode_payment_request, b58_encode_public_address, b58_printable_wrapper_type,
            PrintableWrapperType,
        },
        encoding_helpers::hex_to_ristretto,
    },
};
use mc_common::logger::global_log;
use mc_connection::{
    BlockchainConnection, HardcodedCredentialsProvider, ThickClient, UserTxConnection,
};
use mc_fog_report_validation::{FogPubkeyResolver, FogResolver};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use mc_validator_connection::ValidatorConnection;
use rocket::{
    self, get, http::Status, outcome::Outcome, post, request::FromRequest, routes, Request, State,
};
use rocket_contrib::json::Json;
use serde_json::Map;
use std::{collections::HashMap, convert::TryFrom, iter::FromIterator};

/// State managed by rocket.
pub struct WalletState<
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
> {
    /// The Wallet Service implementation.
    pub service: WalletService<T, FPR>,
}

pub const API_KEY_HEADER: &str = "X-API-KEY";

pub struct APIKeyState(pub String);

/// Ensures check for a pre-shared symmetric API key for the JsonRPC loop on the
/// Mobilecoin wallet.
pub struct ApiKeyGuard {}

#[derive(Debug)]
pub enum ApiKeyError {
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKeyGuard {
    type Error = ApiKeyError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> Outcome<Self, (rocket::http::Status, Self::Error), ()> {
        let client_key = req.headers().get_one(API_KEY_HEADER).unwrap_or_default();
        let local_key = &req
            .guard::<State<APIKeyState>>()
            .expect("api key state config is bad. see main.rs")
            .0;
        if local_key == client_key {
            Outcome::Success(ApiKeyGuard {})
        } else {
            Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid))
        }
    }
}

fn generic_wallet_api<T, FPR>(
    _api_key_guard: ApiKeyGuard,
    state: rocket::State<WalletState<T, FPR>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    let req: JsonRPCRequest = command.0.clone();

    let mut response = JsonRPCResponse {
        method: Some(command.0.method),
        result: None,
        error: None,
        jsonrpc: "2.0".to_string(),
        id: command.0.id,
    };

    let request = match JsonCommandRequest::try_from(&req) {
        Ok(request) => request,
        Err(error) => {
            response.error = Some(format_invalid_request_error(error));
            return Ok(Json(response));
        }
    };

    match wallet_api_inner(&state.service, request) {
        Ok(command_response) => {
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            response.error = Some(rpc_error);
        }
    };

    Ok(Json(response))
}

/// The route for the Full Service Wallet API.
#[post("/wallet", format = "json", data = "<command>")]
pub fn consensus_backed_wallet_api(
    _api_key_guard: ApiKeyGuard,
    state: rocket::State<WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse>, String> {
    generic_wallet_api(_api_key_guard, state, command)
}

#[post("/wallet", format = "json", data = "<command>")]
pub fn validator_backed_wallet_api(
    _api_key_guard: ApiKeyGuard,
    state: rocket::State<WalletState<ValidatorConnection, FogResolver>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse>, String> {
    generic_wallet_api(_api_key_guard, state, command)
}

/// The Wallet API inner method, which handles switching on the method enum.
///
/// Note that this is structured this way so that the routes can be defined to
/// take explicit Rocket state, and then pass the service to the inner method.
/// This allows us to properly construct state with Mock Connection Objects in
/// tests. This also allows us to version the overall API easily.
pub fn wallet_api_inner<T, FPR>(
    service: &WalletService<T, FPR>,
    command: JsonCommandRequest,
) -> Result<JsonCommandResponse, JsonRPCError>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    global_log::trace!("Running command {:?}", command);

    let response = match command {
        JsonCommandRequest::assign_address_for_account {
            account_id,
            metadata,
        } => JsonCommandResponse::assign_address_for_account {
            address: Address::from(
                &service
                    .assign_address_for_account(&AccountID(account_id), metadata.as_deref())
                    .map_err(format_error)?,
            ),
        },
        JsonCommandRequest::build_and_submit_transaction {
            account_id,
            addresses_and_values,
            recipient_public_address,
            value_pmob,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
            comment,
        } => {
            // The user can specify either a single address and a single value, or a list of
            // addresses and values.
            let mut addresses_and_values = addresses_and_values.unwrap_or_default();
            if let (Some(a), Some(v)) = (recipient_public_address, value_pmob) {
                addresses_and_values.push((a, v));
            }
            let (transaction_log, associated_txos, tx_proposal) = service
                .build_and_submit(
                    &account_id,
                    &addresses_and_values,
                    input_txo_ids.as_ref(),
                    fee,
                    tombstone_block,
                    max_spendable_value,
                    comment,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_and_submit_transaction {
                transaction_log: json_rpc::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                ),
                tx_proposal: TxProposal::from(&tx_proposal),
            }
        }
        JsonCommandRequest::build_gift_code {
            account_id,
            value_pmob,
            memo,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
        } => {
            let (tx_proposal, gift_code_b58) = service
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
            JsonCommandResponse::build_gift_code {
                tx_proposal: TxProposal::try_from(&tx_proposal).map_err(format_error)?,
                gift_code_b58: gift_code_b58.to_string(),
            }
        }
        JsonCommandRequest::build_split_txo_transaction {
            txo_id,
            output_values,
            destination_subaddress_index,
            fee,
            tombstone_block,
        } => {
            let tx_proposal = service
                .split_txo(
                    &TxoID(txo_id),
                    &output_values,
                    destination_subaddress_index
                        .map(|f| f.parse::<i64>())
                        .transpose()
                        .map_err(format_error)?,
                    fee,
                    tombstone_block,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_split_txo_transaction {
                tx_proposal: TxProposal::from(&tx_proposal),
                transaction_log_id: TransactionID::from(&tx_proposal.tx).to_string(),
            }
        }
        JsonCommandRequest::build_transaction {
            account_id,
            addresses_and_values,
            recipient_public_address,
            value_pmob,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
            log_tx_proposal,
        } => {
            // The user can specify a list of addresses and values,
            // or a single address and a single value (deprecated).
            let mut addresses_and_values = addresses_and_values.unwrap_or_default();
            if let (Some(a), Some(v)) = (recipient_public_address, value_pmob) {
                addresses_and_values.push((a, v));
            }
            let tx_proposal = service
                .build_transaction(
                    &account_id,
                    &addresses_and_values,
                    input_txo_ids.as_ref(),
                    fee,
                    tombstone_block,
                    max_spendable_value,
                    log_tx_proposal,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_transaction {
                tx_proposal: TxProposal::from(&tx_proposal),
                transaction_log_id: TransactionID::from(&tx_proposal.tx).to_string(),
            }
        }
        JsonCommandRequest::check_b58_type { b58_code } => {
            let b58_type = b58_printable_wrapper_type(b58_code.clone()).map_err(format_error)?;
            let mut b58_data = HashMap::new();
            match b58_type {
                PrintableWrapperType::PublicAddress => {
                    b58_data.insert("public_address_b58".to_string(), b58_code);
                }
                PrintableWrapperType::TransferPayload => {}
                PrintableWrapperType::PaymentRequest => {
                    let payment_request =
                        b58_decode_payment_request(b58_code).map_err(format_error)?;
                    let public_address_b58 =
                        b58_encode_public_address(&payment_request.public_address)
                            .map_err(format_error)?;
                    b58_data.insert("public_address_b58".to_string(), public_address_b58);
                    b58_data.insert("value".to_string(), payment_request.value.to_string());
                    b58_data.insert("memo".to_string(), payment_request.memo);
                }
            }
            JsonCommandResponse::check_b58_type {
                b58_type,
                data: b58_data,
            }
        }
        JsonCommandRequest::check_gift_code_status { gift_code_b58 } => {
            let (status, value, memo) = service
                .check_gift_code_status(&EncodedGiftCode(gift_code_b58))
                .map_err(format_error)?;
            JsonCommandResponse::check_gift_code_status {
                gift_code_status: status,
                gift_code_value: value,
                gift_code_memo: memo,
            }
        }
        JsonCommandRequest::check_receiver_receipt_status {
            address,
            receiver_receipt,
        } => {
            let receipt = service::receipt::ReceiverReceipt::try_from(&receiver_receipt)
                .map_err(format_error)?;
            let (status, txo) = service
                .check_receipt_status(&address, &receipt)
                .map_err(format_error)?;
            JsonCommandResponse::check_receiver_receipt_status {
                receipt_transaction_status: status,
                txo: txo.as_ref().map(Txo::from),
            }
        }
        JsonCommandRequest::claim_gift_code {
            gift_code_b58,
            account_id,
            address,
        } => {
            let tx = service
                .claim_gift_code(
                    &EncodedGiftCode(gift_code_b58),
                    &AccountID(account_id),
                    address,
                )
                .map_err(format_error)?;
            JsonCommandResponse::claim_gift_code {
                txo_id: TxoID::from(&tx.prefix.outputs[0]).to_string(),
            }
        }
        JsonCommandRequest::create_account {
            name,
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
        } => {
            let account: db::models::Account = service
                .create_account(
                    name,
                    fog_report_url.unwrap_or_default(),
                    fog_report_id.unwrap_or_default(),
                    fog_authority_spki.unwrap_or_default(),
                )
                .map_err(format_error)?;

            JsonCommandResponse::create_account {
                account: json_rpc::account::Account::try_from(&account).map_err(|e| {
                    format_error(format!("Could not get RPC Account from DB Account {:?}", e))
                })?,
            }
        }
        JsonCommandRequest::create_payment_request {
            account_id,
            subaddress_index,
            amount_pmob,
            memo,
        } => JsonCommandResponse::create_payment_request {
            payment_request_b58: service
                .create_payment_request(account_id, subaddress_index, amount_pmob, memo)
                .map_err(format_error)?,
        },
        JsonCommandRequest::create_receiver_receipts { tx_proposal } => {
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
            JsonCommandResponse::create_receiver_receipts {
                receiver_receipts: json_receipts,
            }
        }
        JsonCommandRequest::export_account_secrets { account_id } => {
            let account = service
                .get_account(&AccountID(account_id))
                .map_err(format_error)?;
            JsonCommandResponse::export_account_secrets {
                account_secrets: AccountSecrets::try_from(&account).map_err(format_error)?,
            }
        }
        JsonCommandRequest::export_view_only_account_secrets { account_id } => {
            let account = service
                .get_view_only_account(&account_id)
                .map_err(format_error)?;
            JsonCommandResponse::export_view_only_account_secrets {
                view_only_account_secrets:
                    json_rpc::view_only_account::ViewOnlyAccountSecrets::try_from(&account)
                        .map_err(format_error)?,
            }
        }

        JsonCommandRequest::get_account { account_id } => JsonCommandResponse::get_account {
            account: json_rpc::account::Account::try_from(
                &service
                    .get_account(&AccountID(account_id))
                    .map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
        JsonCommandRequest::get_account_status { account_id } => {
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
            JsonCommandResponse::get_account_status { account, balance }
        }
        JsonCommandRequest::get_address_for_account { account_id, index } => {
            let assigned_subaddress = service
                .get_address_for_account(&AccountID(account_id), index)
                .map_err(format_error)?;
            JsonCommandResponse::get_address_for_account {
                address: Address::from(&assigned_subaddress),
            }
        }
        JsonCommandRequest::get_addresses_for_account {
            account_id,
            offset,
            limit,
        } => {
            let o = offset.parse::<i64>().map_err(format_error)?;
            let l = limit.parse::<i64>().map_err(format_error)?;

            if l > 1000 {
                return Err(format_error("limit must not exceed 1000"));
            }

            let addresses = service
                .get_addresses_for_account(&AccountID(account_id), Some(o), Some(l))
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

            JsonCommandResponse::get_addresses_for_account {
                public_addresses: addresses
                    .iter()
                    .map(|a| a.assigned_subaddress_b58.clone())
                    .collect(),
                address_map,
            }
        }
        JsonCommandRequest::get_all_accounts => {
            let accounts = service.list_accounts().map_err(format_error)?;
            let json_accounts: Vec<(String, serde_json::Value)> = accounts
                .iter()
                .map(|a| {
                    json_rpc::account::Account::try_from(a)
                        .map_err(format_error)
                        .and_then(|v| {
                            serde_json::to_value(v)
                                .map(|v| (a.account_id_hex.clone(), v))
                                .map_err(format_error)
                        })
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, JsonRPCError>>()?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(json_accounts);
            JsonCommandResponse::get_all_accounts {
                account_ids: accounts.iter().map(|a| a.account_id_hex.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequest::get_all_addresses_for_account { account_id } => {
            let addresses = service
                .get_addresses_for_account(&AccountID(account_id), None, None)
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

            JsonCommandResponse::get_addresses_for_account {
                public_addresses: addresses
                    .iter()
                    .map(|a| a.assigned_subaddress_b58.clone())
                    .collect(),
                address_map,
            }
        }
        JsonCommandRequest::get_all_gift_codes {} => JsonCommandResponse::get_all_gift_codes {
            gift_codes: service
                .list_gift_codes()
                .map_err(format_error)?
                .iter()
                .map(GiftCode::from)
                .collect(),
        },
        JsonCommandRequest::get_all_transaction_logs_for_account { account_id } => {
            let transaction_logs_and_txos = service
                .list_transaction_logs(&AccountID(account_id), None, None)
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

            JsonCommandResponse::get_all_transaction_logs_for_account {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _a)| t.transaction_id_hex.to_string())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_all_transaction_logs_for_block { block_index } => {
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

            JsonCommandResponse::get_all_transaction_logs_for_block {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _a)| t.transaction_id_hex.to_string())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_all_transaction_logs_ordered_by_block => {
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

            JsonCommandResponse::get_all_transaction_logs_ordered_by_block {
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_all_txos_for_account { account_id } => {
            let txos = service
                .list_txos(&AccountID(account_id), None, None)
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo_id_hex.clone(),
                            serde_json::to_value(Txo::from(t)).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_all_txos_for_account {
                txo_ids: txos.iter().map(|t| t.txo_id_hex.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_all_txos_for_address { address } => {
            let txos = service
                .get_all_txos_for_address(&address)
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo_id_hex.clone(),
                            serde_json::to_value(Txo::from(t)).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_all_txos_for_address {
                txo_ids: txos.iter().map(|t| t.txo_id_hex.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_all_view_only_accounts => {
            let accounts = service.list_view_only_accounts().map_err(format_error)?;
            let json_accounts: Vec<(String, serde_json::Value)> = accounts
                .iter()
                .map(|a| {
                    json_rpc::view_only_account::ViewOnlyAccount::try_from(a)
                        .map_err(format_error)
                        .and_then(|v| {
                            serde_json::to_value(v)
                                .map(|v| (a.account_id_hex.clone(), v))
                                .map_err(format_error)
                        })
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, JsonRPCError>>()?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(json_accounts);
            JsonCommandResponse::get_all_view_only_accounts {
                account_ids: accounts.iter().map(|a| a.account_id_hex.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequest::get_balance_for_account { account_id } => {
            JsonCommandResponse::get_balance_for_account {
                balance: Balance::from(
                    &service
                        .get_balance_for_account(&AccountID(account_id))
                        .map_err(format_error)?,
                ),
            }
        }
        JsonCommandRequest::get_balance_for_address { address } => {
            JsonCommandResponse::get_balance_for_address {
                balance: Balance::from(
                    &service
                        .get_balance_for_address(&address)
                        .map_err(format_error)?,
                ),
            }
        }
        JsonCommandRequest::get_balance_for_view_only_account { account_id } => {
            JsonCommandResponse::get_balance_for_view_only_account {
                balance: ViewOnlyBalance::from(
                    &service
                        .get_balance_for_view_only_account(&account_id)
                        .map_err(format_error)?,
                ),
            }
        }
        JsonCommandRequest::get_block { block_index } => {
            let (block, block_contents) = service
                .get_block_object(block_index.parse::<u64>().map_err(format_error)?)
                .map_err(format_error)?;
            JsonCommandResponse::get_block {
                block: Block::new(&block),
                block_contents: BlockContents::new(&block_contents),
            }
        }
        JsonCommandRequest::get_confirmations { transaction_log_id } => {
            JsonCommandResponse::get_confirmations {
                confirmations: service
                    .get_confirmations(&transaction_log_id)
                    .map_err(format_error)?
                    .iter()
                    .map(Confirmation::from)
                    .collect(),
            }
        }
        JsonCommandRequest::get_gift_code { gift_code_b58 } => JsonCommandResponse::get_gift_code {
            gift_code: GiftCode::from(
                &service
                    .get_gift_code(&EncodedGiftCode(gift_code_b58))
                    .map_err(format_error)?,
            ),
        },
        JsonCommandRequest::get_mc_protocol_transaction { transaction_log_id } => {
            let tx = service
                .get_transaction_object(&transaction_log_id)
                .map_err(format_error)?;
            let proto_tx = mc_api::external::Tx::from(&tx);
            let json_tx = JsonTx::from(&proto_tx);
            JsonCommandResponse::get_mc_protocol_transaction {
                transaction: json_tx,
            }
        }
        JsonCommandRequest::get_mc_protocol_txo { txo_id } => {
            let tx_out = service.get_txo_object(&txo_id).map_err(format_error)?;
            let proto_txo = mc_api::external::TxOut::from(&tx_out);
            let json_txo = JsonTxOut::from(&proto_txo);
            JsonCommandResponse::get_mc_protocol_txo { txo: json_txo }
        }
        JsonCommandRequest::get_network_status => JsonCommandResponse::get_network_status {
            network_status: NetworkStatus::try_from(
                &service.get_network_status().map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
        JsonCommandRequest::get_transaction_log { transaction_log_id } => {
            let (transaction_log, associated_txos) = service
                .get_transaction_log(&transaction_log_id)
                .map_err(format_error)?;
            JsonCommandResponse::get_transaction_log {
                transaction_log: json_rpc::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                ),
            }
        }
        JsonCommandRequest::get_transaction_logs_for_account {
            account_id,
            offset,
            limit,
        } => {
            let o = offset.parse::<i64>().map_err(format_error)?;
            let l = limit.parse::<i64>().map_err(format_error)?;

            if l > 1000 {
                return Err(format_error("limit must not exceed 1000"));
            }

            let transaction_logs_and_txos = service
                .list_transaction_logs(&AccountID(account_id), Some(o), Some(l))
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

            JsonCommandResponse::get_transaction_logs_for_account {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _a)| t.transaction_id_hex.to_string())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_txo { txo_id } => {
            let result = service.get_txo(&TxoID(txo_id)).map_err(format_error)?;
            JsonCommandResponse::get_txo {
                txo: Txo::from(&result),
            }
        }
        JsonCommandRequest::get_txos_for_account {
            account_id,
            offset,
            limit,
        } => {
            let o = offset
                .parse::<u64>()
                .map_err(format_invalid_request_error)?;
            let l = limit.parse::<u64>().map_err(format_invalid_request_error)?;

            if l > 1000 {
                return Err(format_error("limit must not exceed 1000"));
            }

            let txos = service
                .list_txos(&AccountID(account_id), Some(o), Some(l))
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo_id_hex.clone(),
                            serde_json::to_value(Txo::from(t)).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_txos_for_account {
                txo_ids: txos.iter().map(|t| t.txo_id_hex.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_txos_for_view_only_account {
            account_id,
            offset,
            limit,
        } => {
            let o = offset
                .parse::<i64>()
                .map_err(format_invalid_request_error)?;
            let l = limit.parse::<i64>().map_err(format_invalid_request_error)?;

            if l > 1000 {
                return Err(format_error("limit must not exceed 1000"));
            }

            let txos = service
                .list_view_only_txos(&account_id, Some(o), Some(l))
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|t| {
                        (
                            t.txo_id_hex.clone(),
                            serde_json::to_value(ViewOnlyTxo::from(t))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_txos_for_account {
                txo_ids: txos.iter().map(|t| t.txo_id_hex.clone()).collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_wallet_status => JsonCommandResponse::get_wallet_status {
            wallet_status: WalletStatus::try_from(
                &service.get_wallet_status().map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
        JsonCommandRequest::get_view_only_account { account_id } => {
            JsonCommandResponse::get_view_only_account {
                view_only_account: json_rpc::view_only_account::ViewOnlyAccount::try_from(
                    &service
                        .get_view_only_account(&account_id)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequest::import_account {
            mnemonic,
            key_derivation_version,
            name,
            first_block_index,
            next_subaddress_index,
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;
            let ns = next_subaddress_index
                .map(|ns| ns.parse::<u64>())
                .transpose()
                .map_err(format_error)?;
            let kdv = key_derivation_version.parse::<u8>().map_err(format_error)?;

            JsonCommandResponse::import_account {
                account: json_rpc::account::Account::try_from(
                    &service
                        .import_account(
                            mnemonic,
                            kdv,
                            name,
                            fb,
                            ns,
                            fog_report_url.unwrap_or_default(),
                            fog_report_id.unwrap_or_default(),
                            fog_authority_spki.unwrap_or_default(),
                        )
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequest::import_account_from_legacy_root_entropy {
            entropy,
            name,
            first_block_index,
            next_subaddress_index,
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;
            let ns = next_subaddress_index
                .map(|ns| ns.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            JsonCommandResponse::import_account {
                account: json_rpc::account::Account::try_from(
                    &service
                        .import_account_from_legacy_root_entropy(
                            entropy,
                            name,
                            fb,
                            ns,
                            fog_report_url.unwrap_or_default(),
                            fog_report_id.unwrap_or_default(),
                            fog_authority_spki.unwrap_or_default(),
                        )
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequest::import_view_only_account {
            view_private_key,
            name,
            first_block_index,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<i64>())
                .transpose()
                .map_err(format_error)?;

            let n = name.unwrap_or_default();

            let decoded_key = hex_to_ristretto(&view_private_key).map_err(format_error)?;

            JsonCommandResponse::import_view_only_account {
                view_only_account: json_rpc::view_only_account::ViewOnlyAccount::try_from(
                    &service
                        .import_view_only_account(decoded_key, &n, fb)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequest::remove_account { account_id } => JsonCommandResponse::remove_account {
            removed: service
                .remove_account(&AccountID(account_id))
                .map_err(format_error)?,
        },
        JsonCommandRequest::remove_gift_code { gift_code_b58 } => {
            JsonCommandResponse::remove_gift_code {
                removed: service
                    .remove_gift_code(&EncodedGiftCode(gift_code_b58))
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequest::remove_view_only_account { account_id } => {
            JsonCommandResponse::remove_view_only_account {
                removed: service
                    .remove_view_only_account(&account_id)
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequest::submit_gift_code {
            from_account_id,
            gift_code_b58,
            tx_proposal,
        } => {
            let gift_code = service
                .submit_gift_code(
                    &AccountID(from_account_id),
                    &EncodedGiftCode(gift_code_b58),
                    &mc_mobilecoind::payments::TxProposal::try_from(&tx_proposal)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?;
            JsonCommandResponse::submit_gift_code {
                gift_code: GiftCode::from(&gift_code),
            }
        }
        JsonCommandRequest::submit_transaction {
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
            JsonCommandResponse::submit_transaction {
                transaction_log: result,
            }
        }
        JsonCommandRequest::update_account_name { account_id, name } => {
            JsonCommandResponse::update_account_name {
                account: json_rpc::account::Account::try_from(
                    &service
                        .update_account_name(&AccountID(account_id), name)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequest::update_view_only_account_name { account_id, name } => {
            JsonCommandResponse::update_view_only_account_name {
                view_only_account: json_rpc::view_only_account::ViewOnlyAccount::try_from(
                    &service
                        .update_view_only_account_name(&account_id, &name)
                        .map_err(format_error)?,
                )
                .map_err(format_error)?,
            }
        }
        JsonCommandRequest::validate_confirmation {
            account_id,
            txo_id,
            confirmation,
        } => {
            let result = service
                .validate_confirmation(&AccountID(account_id), &TxoID(txo_id), &confirmation)
                .map_err(format_error)?;
            JsonCommandResponse::validate_confirmation { validated: result }
        }
        JsonCommandRequest::verify_address { address } => JsonCommandResponse::verify_address {
            verified: service.verify_address(&address).map_err(format_error)?,
        },
        JsonCommandRequest::version => JsonCommandResponse::version {
            string: env!("CARGO_PKG_VERSION").to_string(),
            number: (
                env!("CARGO_PKG_VERSION_MAJOR").to_string(),
                env!("CARGO_PKG_VERSION_MINOR").to_string(),
                env!("CARGO_PKG_VERSION_PATCH").to_string(),
                env!("CARGO_PKG_VERSION_PRE").to_string(),
            ),
            commit: env!("VERGEN_GIT_SHA").to_string(),
        },
    };

    Ok(response)
}

#[get("/wallet")]
fn wallet_help() -> Result<String, String> {
    Ok(help_str())
}

#[get("/health")]
fn health() -> Result<(), ()> {
    Ok(())
}

/// Returns an instance of a Rocket server.
pub fn consensus_backed_rocket(
    rocket_config: rocket::Config,
    state: WalletState<ThickClient<HardcodedCredentialsProvider>, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![consensus_backed_wallet_api, wallet_help, health],
        )
        .manage(state)
}

pub fn validator_backed_rocket(
    rocket_config: rocket::Config,
    state: WalletState<ValidatorConnection, FogResolver>,
) -> rocket::Rocket {
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![validator_backed_wallet_api, wallet_help, health],
        )
        .manage(state)
}
