use crate::{
    db::{
        account::AccountID,
        transaction_log::TransactionId,
        txo::{TxoID, TxoStatus},
    },
    json_rpc::{
        self,
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::{
            format_error, format_invalid_request_error, JsonRPCError, JsonRPCResponse,
        },
        v1::{
            api::{request::JsonCommandRequest, response::JsonCommandResponse},
            models::{
                account::Account,
                account_secrets::AccountSecrets,
                address::Address,
                balance::Balance,
                block::{Block, BlockContents},
                confirmation_number::Confirmation,
                gift_code::GiftCode,
                network_status::NetworkStatus,
                receiver_receipt::ReceiverReceipt,
                transaction_log::TransactionLog,
                tx_proposal::TxProposal,
                txo::Txo,
                wallet_status::WalletStatus,
            },
        },
        v2::models::amount::Amount,
        wallet::{ApiKeyGuard, WalletState},
    },
    service::{
        self,
        account::AccountService,
        address::AddressService,
        balance::BalanceService,
        confirmation_number::ConfirmationService,
        gift_code::{EncodedGiftCode, GiftCodeService},
        ledger::LedgerService,
        payment_request::PaymentRequestService,
        receipt::ReceiptService,
        transaction::{TransactionMemo, TransactionService},
        transaction_log::TransactionLogService,
        txo::TxoService,
        WalletService,
    },
    util::b58::{
        b58_decode_payment_request, b58_encode_public_address, b58_printable_wrapper_type,
        PrintableWrapperType,
    },
};
use mc_common::logger::global_log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use mc_transaction_core::{tokens::Mob, Amount as CoreAmount, Token};
use rocket::{self, serde::json::Json};
use serde_json::Map;
use std::{collections::HashMap, convert::TryFrom, iter::FromIterator};

pub async fn generic_wallet_api<T, FPR>(
    _api_key_guard: ApiKeyGuard,
    state: &rocket::State<WalletState<T, FPR>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse>>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    let req: JsonRPCRequest = command.0.clone();

    let mut response: JsonRPCResponse<JsonCommandResponse> = JsonRPCResponse {
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

    match wallet_api_inner(&state.service, request).await {
        Ok(command_response) => {
            global_log::info!("Command executed successfully");
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            global_log::info!("Command failed with error: {:?}", rpc_error);
            response.error = Some(rpc_error);
        }
    };

    Ok(Json(response))
}

/// The Wallet API inner method, which handles switching on the method enum.
///
/// Note that this is structured this way so that the routes can be defined to
/// take explicit Rocket state, and then pass the service to the inner method.
/// This allows us to properly construct state with Mock Connection Objects in
/// tests. This also allows us to version the overall API easily.
pub async fn wallet_api_inner<T, FPR>(
    service: &WalletService<T, FPR>,
    command: JsonCommandRequest,
) -> Result<JsonCommandResponse, JsonRPCError>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    global_log::info!("Running command {:?}", command);

    if service.resync_in_progress().map_err(format_error)? {
        let wallet_status = service.get_wallet_status().map_err(format_error)?;

        let percent_complete = (wallet_status.min_synced_block_index as f64
            / wallet_status.local_block_height as f64
            * 100.0) as u64;

        return Err(format_error(&format!(
            "Resync in progress, please wait until it is completed to perform API calls... ({percent_complete}% complete)"
        )));
    }

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
            // The user can specify either a single address and a single value,
            // or a list of addresses and values.
            let mut addresses_and_values = addresses_and_values.unwrap_or_default();
            if let (Some(a), Some(v)) = (recipient_public_address, value_pmob) {
                addresses_and_values.push((a, v));
            }

            let addresses_and_amounts: Vec<(String, Amount)> = addresses_and_values
                .into_iter()
                .map(|(a, v)| {
                    (
                        a,
                        Amount {
                            value: v.into(),
                            token_id: Mob::ID.to_string().into(),
                        },
                    )
                })
                .collect();

            let (transaction_log, associated_txos, _, tx_proposal) = service
                .build_sign_and_submit_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee,
                    Some(Mob::ID.to_string()),
                    tombstone_block,
                    max_spendable_value,
                    comment,
                    TransactionMemo::RTH {
                        subaddress_index: None,
                    },
                    None,
                )
                .await
                .map_err(format_error)?;

            JsonCommandResponse::build_and_submit_transaction {
                transaction_log: json_rpc::v1::models::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                ),
                tx_proposal: TxProposal::try_from(&tx_proposal).map_err(format_error)?,
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
                .await
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
                    Some(Mob::ID.to_string()),
                    tombstone_block,
                )
                .await
                .map_err(format_error)?;
            JsonCommandResponse::build_split_txo_transaction {
                tx_proposal: TxProposal::try_from(&tx_proposal).map_err(format_error)?,
                transaction_log_id: TransactionId::from(&tx_proposal.tx).to_string(),
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
            log_tx_proposal: _,
        } => {
            // The user can specify either a single address and a single value,
            // or a list of addresses and values.
            let mut addresses_and_values = addresses_and_values.unwrap_or_default();
            if let (Some(a), Some(v)) = (recipient_public_address, value_pmob) {
                addresses_and_values.push((a, v));
            }

            let addresses_and_amounts: Vec<(String, Amount)> = addresses_and_values
                .into_iter()
                .map(|(a, v)| {
                    (
                        a,
                        Amount {
                            value: v.into(),
                            token_id: Mob::ID.to_string().into(),
                        },
                    )
                })
                .collect();

            let tx_proposal = service
                .build_and_sign_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee,
                    Some(Mob::ID.to_string()),
                    tombstone_block,
                    max_spendable_value,
                    TransactionMemo::RTH {
                        subaddress_index: None,
                    },
                    None,
                )
                .await
                .map_err(format_error)?;

            JsonCommandResponse::build_transaction {
                tx_proposal: TxProposal::try_from(&tx_proposal).map_err(format_error)?,
                transaction_log_id: TransactionId::from(&tx_proposal.tx).to_string(),
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
            let (status, txo_and_status) = service
                .check_receipt_status(&address, &receipt)
                .map_err(format_error)?;
            JsonCommandResponse::check_receiver_receipt_status {
                receipt_transaction_status: status,
                txo: txo_and_status.as_ref().map(Txo::from),
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
            fog_report_id: _, // Deprecated
            fog_authority_spki,
        } => {
            let account = service
                .create_account(
                    name,
                    fog_report_url.unwrap_or_default(),
                    fog_authority_spki.unwrap_or_default(),
                )
                .map_err(format_error)?;
            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;

            JsonCommandResponse::create_account {
                account: Account::new(&account, next_subaddress_index).map_err(format_error)?,
            }
        }
        JsonCommandRequest::create_payment_request {
            account_id,
            subaddress_index,
            amount_pmob,
            memo,
        } => JsonCommandResponse::create_payment_request {
            payment_request_b58: service
                .create_payment_request(
                    account_id,
                    subaddress_index,
                    CoreAmount::new(amount_pmob.parse::<u64>().map_err(format_error)?, Mob::ID),
                    memo,
                )
                .map_err(format_error)?,
        },
        JsonCommandRequest::create_receiver_receipts { tx_proposal } => {
            let receipts = service
                .create_receiver_receipts(
                    &service::models::tx_proposal::TxProposal::try_from(&tx_proposal)
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
        JsonCommandRequest::get_account { account_id } => {
            let account_id = AccountID(account_id);
            let account = service.get_account(&account_id).map_err(format_error)?;
            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&account_id)
                .map_err(format_error)?;

            JsonCommandResponse::get_account {
                account: Account::new(&account, next_subaddress_index).map_err(format_error)?,
            }
        }
        JsonCommandRequest::get_account_status { account_id } => {
            let account_id = AccountID(account_id);
            let account = &service.get_account(&account_id).map_err(format_error)?;
            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&account_id)
                .map_err(format_error)?;

            let balance_map = service
                .get_balance_for_account(&account_id)
                .map_err(format_error)?;
            let balance_mob = balance_map.get(&Mob::ID).unwrap_or_default();

            let network_status = service.get_network_status().map_err(format_error)?;

            let balance = Balance::new(
                balance_mob,
                account.next_block_index as u64,
                &network_status,
            );

            let account = Account::new(account, next_subaddress_index).map_err(format_error)?;
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
            let (o, l) = page_helper(offset, limit)?;
            let addresses = service
                .get_addresses(Some(account_id), Some(o), Some(l))
                .map_err(format_error)?;
            let address_map: Map<String, serde_json::Value> = Map::from_iter(
                addresses
                    .iter()
                    .map(|a| {
                        (
                            a.public_address_b58.clone(),
                            serde_json::to_value(Address::from(a))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_addresses_for_account {
                public_addresses: addresses
                    .iter()
                    .map(|a| a.public_address_b58.clone())
                    .collect(),
                address_map,
            }
        }
        JsonCommandRequest::get_all_accounts => {
            let accounts = service.list_accounts(None, None).map_err(format_error)?;
            let json_accounts: Vec<(String, serde_json::Value)> = accounts
                .iter()
                .map(|a| {
                    let next_subaddress_index = service
                        .get_next_subaddress_index_for_account(&AccountID(a.id.clone()))
                        .map_err(format_error)?;
                    let account_json =
                        Account::new(a, next_subaddress_index).map_err(format_error)?;

                    serde_json::to_value(account_json)
                        .map(|v| (a.id.clone(), v))
                        .map_err(format_error)
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, JsonRPCError>>()?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(json_accounts);
            JsonCommandResponse::get_all_accounts {
                account_ids: accounts.iter().map(|a| a.id.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequest::get_all_gift_codes {} => JsonCommandResponse::get_all_gift_codes {
            gift_codes: service
                .list_gift_codes(None, None)
                .map_err(format_error)?
                .iter()
                .map(GiftCode::from)
                .collect(),
        },
        JsonCommandRequest::get_all_transaction_logs_for_block { block_index } => {
            let block_index = block_index.parse::<u64>().map_err(format_error)?;
            let transaction_logs_and_txos = service
                .list_transaction_logs(None, None, None, Some(block_index), Some(block_index))
                .map_err(format_error)?;

            let mut transaction_log_map: Map<String, serde_json::Value> = Map::new();

            let received_txos = service
                .list_txos(
                    None,
                    None,
                    None,
                    Some(*Mob::ID),
                    Some(block_index),
                    Some(block_index),
                    None,
                    None,
                )
                .map_err(format_error)?;

            let received_tx_logs: Vec<TransactionLog> = received_txos
                .iter()
                .map(|txo_info| {
                    let subaddress_b58 = match (
                        txo_info.txo.subaddress_index,
                        txo_info.txo.account_id.as_ref(),
                    ) {
                        (Some(subaddress_index), Some(account_id)) => service
                            .get_address_for_account(
                                &AccountID(account_id.clone()),
                                subaddress_index,
                            )
                            .map(|assigned_sub| assigned_sub.public_address_b58)
                            .ok(),
                        _ => None,
                    };

                    TransactionLog::new_from_received_txo(&txo_info.txo, subaddress_b58)
                })
                .collect::<Result<Vec<TransactionLog>, _>>()
                .map_err(format_error)?;

            let mut transaction_log_ids = Vec::new();

            for received_tx_log in received_tx_logs.iter() {
                let tx_log_json = serde_json::to_value(received_tx_log).map_err(format_error)?;
                transaction_log_map.insert(received_tx_log.transaction_log_id.clone(), tx_log_json);
                transaction_log_ids.push(received_tx_log.transaction_log_id.clone());
            }

            for (tx_log, associated_txos, _status) in &transaction_logs_and_txos {
                let tx_log_json =
                    serde_json::json!(json_rpc::v1::models::transaction_log::TransactionLog::new(
                        tx_log,
                        associated_txos
                    ));
                transaction_log_map.insert(tx_log.id.clone(), tx_log_json);
                transaction_log_ids.push(tx_log.id.clone());
            }

            JsonCommandResponse::get_all_transaction_logs_for_block {
                transaction_log_ids,
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_all_transaction_logs_ordered_by_block => {
            let transaction_logs_and_txos = service
                .list_transaction_logs(None, None, None, None, None)
                .map_err(format_error)?;

            let mut transaction_log_map: Map<String, serde_json::Value> = Map::new();

            let received_txos = service
                .list_txos(None, None, None, Some(*Mob::ID), None, None, None, None)
                .map_err(format_error)?;

            let received_tx_logs: Vec<TransactionLog> = received_txos
                .iter()
                .map(|txo_info| {
                    let subaddress_b58 = match (
                        txo_info.txo.subaddress_index,
                        txo_info.txo.account_id.as_ref(),
                    ) {
                        (Some(subaddress_index), Some(account_id)) => service
                            .get_address_for_account(
                                &AccountID(account_id.clone()),
                                subaddress_index,
                            )
                            .map(|assigned_sub| assigned_sub.public_address_b58)
                            .ok(),
                        _ => None,
                    };

                    TransactionLog::new_from_received_txo(&txo_info.txo, subaddress_b58)
                })
                .collect::<Result<Vec<TransactionLog>, _>>()
                .map_err(format_error)?;

            for received_tx_log in received_tx_logs.iter() {
                let tx_log_json = serde_json::to_value(received_tx_log).map_err(format_error)?;
                transaction_log_map.insert(received_tx_log.transaction_log_id.clone(), tx_log_json);
            }

            for (tx_log, associated_txos, _status) in transaction_logs_and_txos {
                let tx_log_json =
                    serde_json::json!(json_rpc::v1::models::transaction_log::TransactionLog::new(
                        &tx_log,
                        &associated_txos
                    ));
                transaction_log_map.insert(tx_log.id.clone(), tx_log_json);
            }

            JsonCommandResponse::get_all_transaction_logs_ordered_by_block {
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_all_txos_for_address { address } => {
            let txos = service
                .list_txos(
                    None,
                    Some(address),
                    None,
                    Some(*Mob::ID),
                    None,
                    None,
                    None,
                    None,
                )
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|txo_info| {
                        (
                            txo_info.txo.id.clone(),
                            serde_json::to_value(Txo::from(txo_info))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_all_txos_for_address {
                txo_ids: txos
                    .iter()
                    .map(|txo_info| txo_info.txo.id.clone())
                    .collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_balance_for_account { account_id } => {
            let account_id = AccountID(account_id);
            let account = &service.get_account(&account_id).map_err(format_error)?;
            let balance_map = service
                .get_balance_for_account(&account_id)
                .map_err(format_error)?;
            let balance_mob = balance_map.get(&Mob::ID).unwrap_or_default();

            let network_status = service.get_network_status().map_err(format_error)?;
            JsonCommandResponse::get_balance_for_account {
                balance: Balance::new(
                    balance_mob,
                    account.next_block_index as u64,
                    &network_status,
                ),
            }
        }
        JsonCommandRequest::get_balance_for_address { address } => {
            let assigned_subaddress = service.get_address(&address).map_err(format_error)?;
            let account_id = AccountID(assigned_subaddress.account_id);
            let account = &service.get_account(&account_id).map_err(format_error)?;

            let balance_map = service
                .get_balance_for_address(&address)
                .map_err(format_error)?;

            let balance_mob = balance_map.get(&Mob::ID).unwrap_or_default();

            JsonCommandResponse::get_balance_for_address {
                balance: Balance::new(
                    balance_mob,
                    account.next_block_index as u64,
                    &service.get_network_status().map_err(format_error)?,
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
            // Check whether the transaction_log_id actually refers to the txo_id of a
            // received transaction.
            let txo_id = TxoID(transaction_log_id.clone());
            let json_tx_log = if let Ok(txo_info) = service.get_txo(&txo_id) {
                // A txo was found, determine which address it was received at, if any.
                let subaddress_b58 =
                    match (&txo_info.txo.subaddress_index, &txo_info.txo.account_id) {
                        (Some(subaddress_index), Some(account_id)) => {
                            let account_id = AccountID(account_id.clone());
                            service
                                .get_address_for_account(&account_id, *subaddress_index)
                                .map(|assigned_sub| assigned_sub.public_address_b58)
                                .ok()
                        }
                        _ => None,
                    };
                TransactionLog::new_from_received_txo(&txo_info.txo, subaddress_b58)
                    .map_err(format_error)?
            } else {
                // Txo ID did not match, check whether this is a real transaction log ID.
                let (transaction_log, associated_txos, _) = service
                    .get_transaction_log(&transaction_log_id)
                    .map_err(format_error)?;
                json_rpc::v1::models::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                )
            };

            JsonCommandResponse::get_transaction_log {
                transaction_log: json_tx_log,
            }
        }
        JsonCommandRequest::get_transaction_logs_for_account {
            account_id,
            offset,
            limit,
            min_block_index,
            max_block_index,
        } => {
            let (o, l) = page_helper(offset, limit)?;

            let min_block_index = min_block_index
                .map(|i| i.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let max_block_index = max_block_index
                .map(|i| i.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let mut transaction_log_map: Map<String, serde_json::Value> = Map::new();
            let mut transaction_log_ids: Vec<String> = Vec::new();

            // Add txo ids for received transactions.
            let received_txos = service
                .list_txos(
                    Some(account_id.clone()),
                    None,
                    None,
                    Some(*Mob::ID),
                    None,
                    None,
                    None,
                    None,
                )
                .map_err(format_error)?;

            let received_tx_logs: Vec<TransactionLog> = received_txos
                .iter()
                .map(|txo_info| {
                    let subaddress_b58 = match txo_info.txo.subaddress_index {
                        Some(subaddress_index) => service
                            .get_address_for_account(
                                &AccountID(account_id.clone()),
                                subaddress_index,
                            )
                            .map(|assigned_sub| assigned_sub.public_address_b58)
                            .ok(),
                        None => None,
                    };

                    TransactionLog::new_from_received_txo(&txo_info.txo, subaddress_b58)
                })
                .collect::<Result<Vec<TransactionLog>, _>>()
                .map_err(format_error)?;

            for received_tx_log in received_tx_logs.iter() {
                let tx_log_json = serde_json::to_value(received_tx_log).map_err(format_error)?;
                transaction_log_map.insert(received_tx_log.transaction_log_id.clone(), tx_log_json);
                transaction_log_ids.push(received_tx_log.transaction_log_id.clone());
            }

            // Add transaction log objects for sent transactions.
            let transaction_logs_and_txos = service
                .list_transaction_logs(
                    Some(account_id),
                    None,
                    None,
                    min_block_index,
                    max_block_index,
                )
                .map_err(format_error)?;

            for (tx_log, associated_txos, _status) in transaction_logs_and_txos {
                let tx_log_json =
                    serde_json::json!(json_rpc::v1::models::transaction_log::TransactionLog::new(
                        &tx_log,
                        &associated_txos
                    ));
                transaction_log_map.insert(tx_log.id.clone(), tx_log_json);
                transaction_log_ids.push(tx_log.id.clone());
            }

            // Handle ordering, offset and limit.
            transaction_log_ids.sort();

            let transaction_log_ids_limitted = if l - o < transaction_log_ids.len() as u64 {
                let mut max = (o + l) as usize;
                if max > transaction_log_ids.len() {
                    max = transaction_log_ids.len();
                }
                transaction_log_ids[o as usize..max].to_vec()
            } else {
                transaction_log_ids.clone()
            };

            JsonCommandResponse::get_transaction_logs_for_account {
                transaction_log_ids: transaction_log_ids_limitted,
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_txo { txo_id } => {
            let txo_info = service.get_txo(&TxoID(txo_id)).map_err(format_error)?;
            JsonCommandResponse::get_txo {
                txo: Txo::from(&txo_info),
            }
        }
        JsonCommandRequest::get_txos_for_account {
            account_id,
            status,
            offset,
            limit,
        } => {
            let status = if let Some(status) = status {
                Some(status.parse::<TxoStatus>().map_err(format_error)?)
            } else {
                None
            };

            let (o, l) = page_helper(offset, limit)?;
            let txos = service
                .list_txos(
                    Some(account_id),
                    None,
                    status,
                    Some(*Mob::ID),
                    None,
                    None,
                    Some(o),
                    Some(l),
                )
                .map_err(format_error)?;
            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos.iter()
                    .map(|txo_info| {
                        (
                            txo_info.txo.id.clone(),
                            serde_json::to_value(Txo::from(txo_info))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_txos_for_account {
                txo_ids: txos
                    .iter()
                    .map(|txo_info| txo_info.txo.id.clone())
                    .collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_wallet_status => {
            let wallet_status = service.get_wallet_status().map_err(format_error)?;

            let account_mapped: Vec<(String, serde_json::Value)> = wallet_status
                .account_map
                .iter()
                .map(|(i, a)| {
                    let next_subaddress_index = service
                        .get_next_subaddress_index_for_account(i)
                        .map_err(|_| {
                            ("Could not get next subaddress index for account").to_string()
                        })?;
                    let account = Account::new(a, next_subaddress_index)?;
                    serde_json::to_value(account)
                        .map(|v| (i.to_string(), v))
                        .map_err(|e| format!("Coult not convert account map:{e:?}"))
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, String>>()
                .map_err(format_error)?;
            let account_map = Map::from_iter(account_mapped);

            let wallet_status =
                WalletStatus::new(&wallet_status, account_map).map_err(format_error)?;

            JsonCommandResponse::get_wallet_status { wallet_status }
        }
        JsonCommandRequest::import_account {
            mnemonic,
            name,
            first_block_index,
            next_subaddress_index,
            fog_report_url,
            fog_report_id: _, // Deprecated
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

            let account = service
                .import_account(
                    mnemonic,
                    name,
                    fb,
                    ns,
                    fog_report_url.unwrap_or_default(),
                    fog_authority_spki.unwrap_or_default(),
                )
                .map_err(format_error)?;

            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;

            let account_json =
                Account::new(&account, next_subaddress_index).map_err(format_error)?;

            JsonCommandResponse::import_account {
                account: account_json,
            }
        }
        JsonCommandRequest::import_account_from_legacy_root_entropy {
            entropy,
            name,
            first_block_index,
            next_subaddress_index,
            fog_report_url,
            fog_report_id: _, // Deprecated
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

            let account = service
                .import_account_from_legacy_root_entropy(
                    entropy,
                    name,
                    fb,
                    ns,
                    fog_report_url.unwrap_or_default(),
                    fog_authority_spki.unwrap_or_default(),
                )
                .map_err(format_error)?;

            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;

            let account_json =
                Account::new(&account, next_subaddress_index).map_err(format_error)?;

            JsonCommandResponse::import_account {
                account: account_json,
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
        JsonCommandRequest::submit_gift_code {
            from_account_id,
            gift_code_b58,
            tx_proposal,
        } => {
            let gift_code = service
                .submit_gift_code(
                    &AccountID(from_account_id),
                    &EncodedGiftCode(gift_code_b58),
                    &service::models::tx_proposal::TxProposal::try_from(&tx_proposal)
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
            let result = service
                .submit_transaction(
                    &service::models::tx_proposal::TxProposal::try_from(&tx_proposal)
                        .map_err(format_error)?,
                    comment,
                    account_id,
                )
                .map_err(format_error)?
                .map(|(tx_log, associated_txos, _value_map)| {
                    TransactionLog::new(&tx_log, &associated_txos)
                });
            JsonCommandResponse::submit_transaction {
                transaction_log: result,
            }
        }
        JsonCommandRequest::update_account_name { account_id, name } => {
            let account_id = AccountID(account_id);
            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&account_id)
                .map_err(format_error)?;
            let account = service
                .update_account_name(&account_id, name)
                .map_err(format_error)?;
            let account_json =
                Account::new(&account, next_subaddress_index).map_err(format_error)?;
            JsonCommandResponse::update_account_name {
                account: account_json,
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
            verified: service.verify_address(&address).is_ok(),
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

fn page_helper(offset: Option<String>, limit: Option<String>) -> Result<(u64, u64), JsonRPCError> {
    let offset = match offset {
        Some(o) => o.parse::<u64>().map_err(format_error)?,
        None => 0, // Default offset is zero, at the start of the records.
    };
    let limit = match limit {
        Some(l) => l.parse::<u64>().map_err(format_error)?,
        None => 100, // Default page size is one hundred records.
    };
    Ok((offset, limit))
}
