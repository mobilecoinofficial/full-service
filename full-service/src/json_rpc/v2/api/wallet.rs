use crate::{
    db::{
        self,
        account::AccountID,
        transaction_log::TransactionID,
        txo::{TxoID, TxoStatus},
    },
    json_rpc::{
        self,
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::{
            format_error, format_invalid_request_error, JsonRPCError, JsonRPCResponse,
        },
        v2::{
            api::{request::JsonCommandRequest, response::JsonCommandResponse},
            models::{
                account_secrets::AccountSecrets,
                address::Address,
                balance::Balance,
                block::{Block, BlockContents},
                confirmation_number::Confirmation,
                gift_code::GiftCode,
                network_status::NetworkStatus,
                receiver_receipt::ReceiverReceipt,
                tx_proposal::TxProposal as TxProposalJSON,
                txo::Txo,
                wallet_status::WalletStatus,
            },
        },
        wallet::{ApiKeyGuard, WalletState},
    },
    service,
    service::{
        account::AccountService,
        address::AddressService,
        balance::BalanceService,
        confirmation_number::ConfirmationService,
        gift_code::{EncodedGiftCode, GiftCodeService},
        ledger::LedgerService,
        models::tx_proposal::TxProposal,
        payment_request::PaymentRequestService,
        receipt::ReceiptService,
        transaction::TransactionService,
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
use rocket::{self};
use rocket_contrib::json::Json;
use serde_json::Map;
use std::{collections::HashMap, convert::TryFrom, iter::FromIterator, str::FromStr};

pub fn generic_wallet_api<T, FPR>(
    _api_key_guard: ApiKeyGuard,
    state: rocket::State<WalletState<T, FPR>>,
    command: Json<JsonRPCRequest>,
) -> Result<Json<JsonRPCResponse<JsonCommandResponse>>, String>
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
            addresses_and_amounts,
            recipient_public_address,
            amount,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
            comment,
        } => {
            // The user can specify a list of addresses and values,
            // or a single address and a single value.
            let mut addresses_and_amounts = addresses_and_amounts.unwrap_or_default();
            if let (Some(address), Some(amount)) = (recipient_public_address, amount) {
                addresses_and_amounts.push((address, amount));
            }

            let (transaction_log, associated_txos, value_map, tx_proposal) = service
                .build_and_submit(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    comment,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_and_submit_transaction {
                transaction_log: json_rpc::v2::models::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                    &value_map,
                ),
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
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
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
                gift_code_b58: gift_code_b58.to_string(),
            }
        }
        JsonCommandRequest::build_split_txo_transaction {
            txo_id,
            output_values,
            destination_subaddress_index,
            fee_value,
            fee_token_id,
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
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_split_txo_transaction {
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
                transaction_log_id: TransactionID::from(&tx_proposal.tx).to_string(),
            }
        }
        JsonCommandRequest::build_transaction {
            account_id,
            addresses_and_amounts,
            recipient_public_address,
            amount,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
        } => {
            // The user can specify a list of addresses and values,
            // or a single address and a single value.
            let mut addresses_and_amounts = addresses_and_amounts.unwrap_or_default();
            if let (Some(address), Some(amount)) = (recipient_public_address, amount) {
                addresses_and_amounts.push((address, amount));
            }

            let tx_proposal = service
                .build_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    None,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_transaction {
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
                transaction_log_id: TransactionID::from(&tx_proposal.tx).to_string(),
            }
        }
        JsonCommandRequest::build_unsigned_transaction {
            account_id,
            recipient_public_address,
            amount,
            fee_value,
            fee_token_id,
            tombstone_block,
        } => {
            let mut addresses_and_amounts = Vec::new();
            if let (Some(address), Some(amount)) = (recipient_public_address, amount) {
                addresses_and_amounts.push((address, amount));
            }
            let (unsigned_tx, fog_resolver) = service
                .build_unsigned_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                )
                .map_err(format_error)?;
            JsonCommandResponse::build_unsigned_transaction {
                account_id,
                unsigned_tx,
                fog_resolver,
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
                txo: txo_and_status
                    .as_ref()
                    .map(|(txo, status)| Txo::new(txo, status)),
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
                account: json_rpc::v2::models::account::Account::try_from(&account).map_err(
                    |e| format_error(format!("Could not get RPC Account from DB Account {:?}", e)),
                )?,
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
                    &TxProposal::try_from(&tx_proposal).map_err(format_error)?,
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
        JsonCommandRequest::create_view_only_account_sync_request { account_id } => {
            let unverified_txos = service
                .list_txos(
                    Some(account_id.clone()),
                    None,
                    Some(TxoStatus::Unverified),
                    None,
                    None,
                    None,
                )
                .map_err(format_error)?;

            let unverified_txos_encoded: Vec<String> = unverified_txos
                .iter()
                .map(|(txo, _)| hex::encode(&txo.txo))
                .collect();

            JsonCommandResponse::create_view_only_account_sync_request {
                account_id,
                incomplete_txos_encoded: unverified_txos_encoded,
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
        JsonCommandRequest::export_view_only_account_import_request { account_id } => {
            JsonCommandResponse::export_view_only_account_import_request {
                json_rpc_request: service
                    .get_view_only_account_import_request(&AccountID(account_id))
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequest::get_account { account_id } => JsonCommandResponse::get_account {
            account: json_rpc::v2::models::account::Account::try_from(
                &service
                    .get_account(&AccountID(account_id))
                    .map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
        JsonCommandRequest::get_account_status { account_id } => {
            let account = json_rpc::v2::models::account::Account::try_from(
                &service
                    .get_account(&AccountID(account_id.clone()))
                    .map_err(format_error)?,
            )
            .map_err(format_error)?;

            let network_status = service.get_network_status().map_err(format_error)?;

            let balance = service
                .get_balance_for_account(&AccountID(account_id))
                .map_err(format_error)?;

            let balance_formatted = balance
                .iter()
                .map(|(k, v)| (k.to_string(), Balance::from(v)))
                .collect();
            JsonCommandResponse::get_account_status {
                account,
                network_block_height: network_status.network_block_height.to_string(),
                local_block_height: network_status.local_block_height.to_string(),
                balance_per_token: balance_formatted,
            }
        }
        JsonCommandRequest::get_accounts { offset, limit } => {
            let accounts = service.list_accounts(offset, limit).map_err(format_error)?;
            let json_accounts: Vec<(String, serde_json::Value)> = accounts
                .iter()
                .map(|a| {
                    json_rpc::v2::models::account::Account::try_from(a)
                        .map_err(format_error)
                        .and_then(|v| {
                            serde_json::to_value(v)
                                .map(|v| (a.id.clone(), v))
                                .map_err(format_error)
                        })
                })
                .collect::<Result<Vec<(String, serde_json::Value)>, JsonRPCError>>()?;
            let account_map: Map<String, serde_json::Value> = Map::from_iter(json_accounts);
            JsonCommandResponse::get_accounts {
                account_ids: accounts.iter().map(|a| a.id.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequest::get_address_for_account_at_index { account_id, index } => {
            let assigned_subaddress = service
                .get_address_for_account(&AccountID(account_id), index)
                .map_err(format_error)?;
            JsonCommandResponse::get_address_for_account_at_index {
                address: Address::from(&assigned_subaddress),
            }
        }
        JsonCommandRequest::get_addresses {
            account_id,
            offset,
            limit,
        } => {
            let addresses = service
                .get_addresses(account_id, offset, limit)
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

            JsonCommandResponse::get_addresses {
                public_addresses: addresses
                    .iter()
                    .map(|a| a.assigned_subaddress_b58.clone())
                    .collect(),
                address_map,
            }
        }
        JsonCommandRequest::get_balance_for_account { account_id } => {
            let account_id = AccountID(account_id);
            let account = service.get_account(&account_id).map_err(format_error)?;
            let network_status = service.get_network_status().map_err(format_error)?;

            let balance = service
                .get_balance_for_account(&account_id)
                .map_err(format_error)?;

            let balance_formatted = balance
                .iter()
                .map(|(a, b)| (a.to_string(), Balance::from(b)))
                .collect();
            JsonCommandResponse::get_balance_for_account {
                account_block_height: account.next_block_index.to_string(),
                network_block_height: network_status.network_block_height.to_string(),
                local_block_height: network_status.local_block_height.to_string(),
                balance_per_token: balance_formatted,
            }
        }
        JsonCommandRequest::get_balance_for_address { address } => {
            let subaddress = service.get_address(&address).map_err(format_error)?;
            let account_id = AccountID(subaddress.account_id);
            let account = service.get_account(&account_id).map_err(format_error)?;
            let network_status = service.get_network_status().map_err(format_error)?;

            let balance = service
                .get_balance_for_address(&address)
                .map_err(format_error)?;

            let balance_formatted = balance
                .iter()
                .map(|(a, b)| (a.to_string(), Balance::from(b)))
                .collect();
            JsonCommandResponse::get_balance_for_address {
                account_block_height: account.next_block_index.to_string(),
                network_block_height: network_status.network_block_height.to_string(),
                local_block_height: network_status.local_block_height.to_string(),
                balance_per_token: balance_formatted,
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
        JsonCommandRequest::get_gift_codes { offset, limit } => {
            JsonCommandResponse::get_gift_codes {
                gift_codes: service
                    .list_gift_codes(offset, limit)
                    .map_err(format_error)?
                    .iter()
                    .map(GiftCode::from)
                    .collect(),
            }
        }
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
            let (transaction_log, associated_txos, value_map) = service
                .get_transaction_log(&transaction_log_id)
                .map_err(format_error)?;
            JsonCommandResponse::get_transaction_log {
                transaction_log: json_rpc::v2::models::transaction_log::TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                    &value_map,
                ),
            }
        }
        JsonCommandRequest::get_transaction_logs {
            account_id,
            min_block_index,
            max_block_index,
            offset,
            limit,
        } => {
            let min_block_index = min_block_index
                .map(|i| i.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let max_block_index = max_block_index
                .map(|i| i.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let transaction_logs_and_txos = service
                .list_transaction_logs(account_id, offset, limit, min_block_index, max_block_index)
                .map_err(format_error)?;
            let transaction_log_map: Map<String, serde_json::Value> = Map::from_iter(
                transaction_logs_and_txos
                    .iter()
                    .map(|(t, a, v)| {
                        (
                            t.id.clone(),
                            serde_json::json!(
                                json_rpc::v2::models::transaction_log::TransactionLog::new(t, a, v)
                            ),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_transaction_logs {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _a, _v)| t.id.clone())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_txo { txo_id } => {
            let (txo, status) = service.get_txo(&TxoID(txo_id)).map_err(format_error)?;
            JsonCommandResponse::get_txo {
                txo: Txo::new(&txo, &status),
            }
        }
        JsonCommandRequest::get_txos {
            account_id,
            address,
            status,
            token_id,
            offset,
            limit,
        } => {
            let status = match status {
                Some(s) => Some(TxoStatus::from_str(&s).map_err(format_error)?),
                None => None,
            };

            let token_id = match token_id {
                Some(t) => Some(t.parse::<u64>().map_err(format_error)?),
                None => None,
            };

            let txos_and_statuses = service
                .list_txos(account_id, address, status, token_id, offset, limit)
                .map_err(format_error)?;

            let txo_map: Map<String, serde_json::Value> = Map::from_iter(
                txos_and_statuses
                    .iter()
                    .map(|(t, s)| {
                        (
                            t.id.clone(),
                            serde_json::to_value(Txo::new(t, s)).expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_txos {
                txo_ids: txos_and_statuses
                    .iter()
                    .map(|(t, _)| t.id.clone())
                    .collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_wallet_status => JsonCommandResponse::get_wallet_status {
            wallet_status: WalletStatus::try_from(
                &service.get_wallet_status().map_err(format_error)?,
            )
            .map_err(format_error)?,
        },
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
                account: json_rpc::v2::models::account::Account::try_from(
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
                account: json_rpc::v2::models::account::Account::try_from(
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
            spend_public_key,
            name,
            first_block_index,
            next_subaddress_index,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;
            let ns = next_subaddress_index
                .map(|ns| ns.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            JsonCommandResponse::import_view_only_account {
                account: json_rpc::v2::models::account::Account::try_from(
                    &service
                        .import_view_only_account(view_private_key, spend_public_key, name, fb, ns)
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
        JsonCommandRequest::submit_gift_code {
            from_account_id,
            gift_code_b58,
            tx_proposal,
        } => {
            let gift_code = service
                .submit_gift_code(
                    &AccountID(from_account_id),
                    &EncodedGiftCode(gift_code_b58),
                    &TxProposal::try_from(&tx_proposal).map_err(format_error)?,
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
            let tx_proposal = TxProposal::try_from(&tx_proposal).map_err(format_error)?;
            let result: Option<json_rpc::v2::models::transaction_log::TransactionLog> = service
                .submit_transaction(&tx_proposal, comment, account_id)
                .map_err(format_error)?
                .map(|(transaction_log, associated_txos, value_map)| {
                    json_rpc::v2::models::transaction_log::TransactionLog::new(
                        &transaction_log,
                        &associated_txos,
                        &value_map,
                    )
                });
            JsonCommandResponse::submit_transaction {
                transaction_log: result,
            }
        }
        JsonCommandRequest::sync_view_only_account {
            account_id,
            completed_txos,
            next_subaddress_index,
        } => {
            service
                .sync_account(
                    &AccountID(account_id),
                    completed_txos,
                    next_subaddress_index.parse::<u64>().map_err(format_error)?,
                )
                .map_err(format_error)?;

            JsonCommandResponse::sync_view_only_account
        }
        JsonCommandRequest::update_account_name { account_id, name } => {
            JsonCommandResponse::update_account_name {
                account: json_rpc::v2::models::account::Account::try_from(
                    &service
                        .update_account_name(&AccountID(account_id), name)
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
