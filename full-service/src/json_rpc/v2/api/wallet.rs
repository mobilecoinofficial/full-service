use crate::{
    db::{
        account::{AccountID, AccountModel},
        transaction_log::TransactionId,
        txo::{TxoID, TxoStatus},
    },
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::{
            format_error, format_invalid_params_error, format_invalid_request_error, JsonRPCError,
            JsonRPCResponse,
        },
        v2::{
            api::{request::JsonCommandRequest, response::JsonCommandResponse},
            models::{
                account::{Account, AccountMap},
                account_secrets::AccountSecrets,
                address::{Address, AddressMap},
                balance::{Balance, BalanceMap},
                block::{Block, BlockContents},
                confirmation_number::Confirmation,
                network_status::NetworkStatus,
                public_address::PublicAddress,
                receiver_receipt::ReceiverReceipt,
                transaction_log::TransactionLog,
                tx_proposal::{TxProposal as TxProposalJSON, UnsignedTxProposal},
                txo::Txo,
                wallet_status::WalletStatus,
            },
        },
        wallet::{ApiKeyGuard, WalletState},
    },
    service::{
        self,
        account::AccountService,
        address::AddressService,
        balance::BalanceService,
        confirmation_number::ConfirmationService,
        hardware_wallet::sync_txos,
        ledger::LedgerService,
        memo::MemoService,
        models::tx_proposal::TxProposal,
        network::get_token_metadata,
        payment_request::PaymentRequestService,
        receipt::ReceiptService,
        transaction::{TransactionMemo, TransactionService},
        transaction_log::TransactionLogService,
        txo::TxoService,
        watcher::WatcherService,
        WalletService,
    },
    util::b58::{
        b58_decode_payment_request, b58_encode_public_address, b58_printable_wrapper_type,
        PrintableWrapperType,
    },
};
use mc_account_keys::{burn_address, ShortAddressHash, DEFAULT_SUBADDRESS_INDEX};
use mc_blockchain_types::BlockVersion;
use mc_common::logger::global_log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPrivate, RistrettoPublic};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxOutMembershipProof};
use mc_transaction_core::Amount;
use mc_transaction_extra::BurnRedemptionMemo;
use mc_transaction_signer::types::{AccountId, TxoSyncReq, TxoUnsynced};
use rocket::{self, serde::json::Json};
use serde_json::Map;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    str::FromStr,
};

/// Default amount of recent blocks to return
pub const RECENT_BLOCKS_DEFAULT_LIMIT: usize = 10;

/// Maximal amount of blocks we can return in a single request
pub const MAX_BLOCKS_PER_REQUEST: usize = 100;

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

    match wallet_api_inner(&state.service, request).await {
        Ok(command_response) => {
            global_log::info!(
                "Command executed successfully with response: {:?}",
                command_response
            );
            response.result = Some(command_response);
        }
        Err(rpc_error) => {
            global_log::error!("Command failed with error: {:?}", rpc_error);
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

        return Err(format_error(format!(
            "Resync in progress, please wait until it is completed to perform API calls... ({}% complete)", wallet_status.percent_synced()
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
            addresses_and_amounts,
            recipient_public_address,
            amount,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
            comment,
            block_version,
            sender_memo_credential_subaddress_index,
            payment_request_id,
        } => {
            // The user can specify a list of addresses and values,
            // or a single address and a single value.
            let mut addresses_and_amounts = addresses_and_amounts.unwrap_or_default();
            if let (Some(address), Some(amount)) = (recipient_public_address, amount) {
                addresses_and_amounts.push((address, amount));
            }

            let block_version = match block_version {
                Some(block_version) => Some(
                    BlockVersion::try_from(block_version.parse::<u32>().map_err(format_error)?)
                        .map_err(format_error)?,
                ),
                None => None,
            };

            let sender_memo_credential_subaddress_index = sender_memo_credential_subaddress_index
                .map(|i| i.parse::<u64>().map_err(format_error))
                .transpose()?;

            let payment_request_id = payment_request_id
                .map(|i| i.parse::<u64>().map_err(format_error))
                .transpose()?;

            let transaction_memo = match payment_request_id {
                Some(payment_request_id) => TransactionMemo::RTHWithPaymentRequestId {
                    subaddress_index: sender_memo_credential_subaddress_index,
                    payment_request_id,
                },
                None => TransactionMemo::RTH {
                    subaddress_index: sender_memo_credential_subaddress_index,
                },
            };

            let (transaction_log, associated_txos, value_map, tx_proposal) = service
                .build_sign_and_submit_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    comment,
                    transaction_memo,
                    block_version,
                )
                .await
                .map_err(format_error)?;

            JsonCommandResponse::build_and_submit_transaction {
                transaction_log: TransactionLog::new(
                    &transaction_log,
                    &associated_txos,
                    &value_map,
                ),
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
            }
        }
        JsonCommandRequest::build_burn_transaction {
            account_id,
            amount,
            redemption_memo_hex,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
            block_version,
        } => {
            let mut memo_data = [0; BurnRedemptionMemo::MEMO_DATA_LEN];
            if let Some(redemption_memo_hex) = redemption_memo_hex {
                if redemption_memo_hex.len() != BurnRedemptionMemo::MEMO_DATA_LEN * 2 {
                    return Err(format_error(format!(
                        "Invalid redemption memo length: {}. Must be 128 characters (64 bytes).",
                        redemption_memo_hex.len()
                    )));
                }

                hex::decode_to_slice(&redemption_memo_hex, &mut memo_data).map_err(format_error)?;
            }

            let block_version = match block_version {
                Some(block_version) => Some(
                    BlockVersion::try_from(block_version.parse::<u32>().map_err(format_error)?)
                        .map_err(format_error)?,
                ),
                None => None,
            };

            let tx_proposal = service
                .build_and_sign_transaction(
                    &account_id,
                    &[(
                        b58_encode_public_address(&burn_address()).map_err(format_error)?,
                        amount,
                    )],
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    TransactionMemo::BurnRedemption(memo_data),
                    block_version,
                )
                .await
                .map_err(format_error)?;

            JsonCommandResponse::build_burn_transaction {
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
                transaction_log_id: TransactionId::from(&tx_proposal.tx).to_string(),
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
            block_version,
            sender_memo_credential_subaddress_index,
            payment_request_id,
        } => {
            // The user can specify a list of addresses and values,
            // or a single address and a single value.
            let mut addresses_and_amounts = addresses_and_amounts.unwrap_or_default();
            if let (Some(address), Some(amount)) = (recipient_public_address, amount) {
                addresses_and_amounts.push((address, amount));
            }

            let block_version = match block_version {
                Some(block_version) => Some(
                    BlockVersion::try_from(block_version.parse::<u32>().map_err(format_error)?)
                        .map_err(format_error)?,
                ),
                None => None,
            };

            let sender_memo_credential_subaddress_index = sender_memo_credential_subaddress_index
                .map(|i| i.parse::<u64>().map_err(format_error))
                .transpose()?;

            let payment_request_id = payment_request_id
                .map(|i| i.parse::<u64>().map_err(format_error))
                .transpose()?;

            let transaction_memo = match payment_request_id {
                Some(payment_request_id) => TransactionMemo::RTHWithPaymentRequestId {
                    subaddress_index: sender_memo_credential_subaddress_index,
                    payment_request_id,
                },
                None => TransactionMemo::RTH {
                    subaddress_index: sender_memo_credential_subaddress_index,
                },
            };

            let tx_proposal = service
                .build_and_sign_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    transaction_memo,
                    block_version,
                )
                .await
                .map_err(format_error)?;

            JsonCommandResponse::build_transaction {
                tx_proposal: TxProposalJSON::try_from(&tx_proposal).map_err(format_error)?,
                transaction_log_id: TransactionId::from(&tx_proposal.tx).to_string(),
            }
        }
        JsonCommandRequest::build_unsigned_burn_transaction {
            account_id,
            amount,
            redemption_memo_hex,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
            block_version,
        } => {
            let mut memo_data = [0; BurnRedemptionMemo::MEMO_DATA_LEN];
            if let Some(redemption_memo_hex) = redemption_memo_hex {
                if redemption_memo_hex.len() != BurnRedemptionMemo::MEMO_DATA_LEN * 2 {
                    return Err(format_error(format!(
                        "Invalid redemption memo length: {}. Must be 128 characters (64 bytes).",
                        redemption_memo_hex.len()
                    )));
                }

                hex::decode_to_slice(&redemption_memo_hex, &mut memo_data).map_err(format_error)?;
            }

            let block_version = match block_version {
                Some(block_version) => Some(
                    BlockVersion::try_from(block_version.parse::<u32>().map_err(format_error)?)
                        .map_err(format_error)?,
                ),
                None => None,
            };

            let unsigned_tx_proposal: UnsignedTxProposal = (&service
                .build_transaction(
                    &account_id,
                    &[(
                        b58_encode_public_address(&burn_address()).map_err(format_error)?,
                        amount,
                    )],
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    TransactionMemo::BurnRedemption(memo_data),
                    block_version,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            JsonCommandResponse::build_unsigned_transaction {
                account_id,
                unsigned_tx_proposal,
            }
        }
        JsonCommandRequest::build_unsigned_transaction {
            account_id,
            recipient_public_address,
            amount,
            fee_value,
            fee_token_id,
            tombstone_block,
            addresses_and_amounts,
            input_txo_ids,
            max_spendable_value,
            block_version,
        } => {
            let mut addresses_and_amounts = addresses_and_amounts.unwrap_or_default();
            if let (Some(address), Some(amount)) = (recipient_public_address, amount) {
                addresses_and_amounts.push((address, amount));
            }

            let block_version = match block_version {
                Some(block_version) => Some(
                    BlockVersion::try_from(block_version.parse::<u32>().map_err(format_error)?)
                        .map_err(format_error)?,
                ),
                None => None,
            };

            let unsigned_tx_proposal: UnsignedTxProposal = (&service
                .build_transaction(
                    &account_id,
                    &addresses_and_amounts,
                    input_txo_ids.as_ref(),
                    fee_value,
                    fee_token_id,
                    tombstone_block,
                    max_spendable_value,
                    TransactionMemo::Empty,
                    block_version,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            JsonCommandResponse::build_unsigned_transaction {
                account_id,
                unsigned_tx_proposal,
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
                    b58_data.insert("token_id".to_string(), payment_request.token_id.to_string());
                    b58_data.insert("memo".to_string(), payment_request.memo);
                    b58_data.insert("token_id".to_string(), payment_request.token_id.to_string());
                }
            }
            JsonCommandResponse::check_b58_type {
                b58_type,
                data: b58_data,
            }
        }
        JsonCommandRequest::check_receiver_receipt_status {
            address,
            receiver_receipt,
        } => {
            let receipt = service::receipt::ReceiverReceipt::try_from(&receiver_receipt)
                .map_err(format_error)?;
            let (status, txo_status_and_memo) = service
                .check_receipt_status(&address, &receipt)
                .map_err(format_error)?;

            JsonCommandResponse::check_receiver_receipt_status {
                receipt_transaction_status: status,
                txo: txo_status_and_memo.map(|txo_info| (&txo_info).into()),
            }
        }
        JsonCommandRequest::create_account { name, fog_info } => {
            let fog_info = fog_info.unwrap_or_default();

            let account = service
                .create_account(name, fog_info.report_url, fog_info.authority_spki)
                .map_err(format_error)?;

            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;

            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;

            JsonCommandResponse::create_account { account }
        }
        JsonCommandRequest::create_payment_request {
            account_id,
            subaddress_index,
            amount,
            memo,
        } => JsonCommandResponse::create_payment_request {
            payment_request_b58: service
                .create_payment_request(
                    account_id,
                    subaddress_index,
                    Amount::try_from(&amount).map_err(format_error)?,
                    memo,
                )
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
        JsonCommandRequest::create_view_only_account_import_request { account_id } => {
            JsonCommandResponse::create_view_only_account_import_request {
                json_rpc_request: service
                    .get_view_only_account_import_request(&AccountID(account_id))
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequest::create_view_only_account_sync_request { account_id } => {
            let unverified_txos = service
                .list_txos(
                    Some(account_id),
                    None,
                    Some(TxoStatus::Unverified),
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .map_err(format_error)?;

            let mut unsynced_txos = vec![];
            for txo_info in unverified_txos {
                let txo_pubkey: RistrettoPublic =
                    (&txo_info.txo.public_key().map_err(format_error)?)
                        .try_into()
                        .map_err(format_error)?;
                // We can guarantee that subaddress index will exist because the query for
                // unverified txos only returns txos that have a subaddress
                // index but not a key image.
                let subaddress_index = txo_info.txo.subaddress_index.unwrap() as u64;
                unsynced_txos.push(TxoUnsynced {
                    subaddress: subaddress_index,
                    tx_out_public_key: txo_pubkey.into(),
                });
            }

            // let account_id: AccountId =
            // account_id.as_str().try_into().map_err(format_error)?;
            let account_id = AccountId::from([0u8; 32]);

            let txo_sync_request = TxoSyncReq {
                account_id,
                txos: unsynced_txos,
            };

            JsonCommandResponse::create_view_only_account_sync_request { txo_sync_request }
        }
        JsonCommandRequest::export_account_secrets { account_id } => {
            let account = service
                .get_account(&AccountID(account_id))
                .map_err(format_error)?;
            JsonCommandResponse::export_account_secrets {
                account_secrets: AccountSecrets::try_from(&account).map_err(format_error)?,
            }
        }
        JsonCommandRequest::get_account_status { account_id }
        | JsonCommandRequest::get_balance { account_id } => {
            let account = service
                .get_account(&AccountID(account_id.clone()))
                .map_err(format_error)?;

            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account_id.clone()))
                .map_err(format_error)?;

            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;

            let network_status = service.get_network_status().map_err(format_error)?;

            let balance = service
                .get_balance_for_account(&AccountID(account_id))
                .map_err(format_error)?;

            let balance_formatted = BalanceMap(
                balance
                    .iter()
                    .map(|(k, v)| (k.to_string(), Balance::from(v)))
                    .collect(),
            );

            JsonCommandResponse::get_account_status {
                account,
                network_block_height: network_status.network_block_height.to_string(),
                local_block_height: network_status.local_block_height.to_string(),
                balance_per_token: balance_formatted,
            }
        }
        JsonCommandRequest::get_accounts { offset, limit } => {
            let accounts = service.list_accounts(offset, limit).map_err(format_error)?;
            let account_map = AccountMap(
                accounts
                    .iter()
                    .map(|a| {
                        let next_subaddress_index = service
                            .get_next_subaddress_index_for_account(&AccountID(a.id.clone()))
                            .map_err(format_error)?;
                        let main_public_address: mc_account_keys::PublicAddress = (&service
                            .get_address_for_account(
                                &a.id.clone().into(),
                                DEFAULT_SUBADDRESS_INDEX as i64,
                            )
                            .map_err(format_error)?)
                            .try_into()
                            .map_err(format_error)?;
                        Ok((
                            a.id.to_string(),
                            Account::new(a, &main_public_address, next_subaddress_index)
                                .map_err(format_error)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
            );

            JsonCommandResponse::get_accounts {
                account_ids: accounts.iter().map(|a| a.id.clone()).collect(),
                account_map,
            }
        }
        JsonCommandRequest::get_address { public_address_b58 } => {
            let assigned_address = service
                .get_address(&public_address_b58)
                .map_err(format_error)?;
            JsonCommandResponse::get_address {
                address: Address::from(&assigned_address),
            }
        }
        JsonCommandRequest::get_address_details { address } => {
            let address = service.verify_address(&address).map_err(format_error)?;

            JsonCommandResponse::get_address_details {
                details: PublicAddress::from(&address),
                short_address_hash: hex::encode(ShortAddressHash::from(&address).as_ref()),
            }
        }
        JsonCommandRequest::get_address_for_account { account_id, index } => {
            let assigned_subaddress = service
                .get_address_for_account(&AccountID(account_id), index)
                .map_err(format_error)?;
            JsonCommandResponse::get_address_for_account {
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

            let address_map = AddressMap(
                addresses
                    .iter()
                    .map(|a| (a.public_address_b58.clone(), Address::from(a)))
                    .collect(),
            );

            JsonCommandResponse::get_addresses {
                public_addresses: addresses
                    .iter()
                    .map(|a| a.public_address_b58.clone())
                    .collect(),
                address_map,
            }
        }
        JsonCommandRequest::get_address_status { address } => {
            let subaddress = service.get_address(&address).map_err(format_error)?;
            let account_id = AccountID(subaddress.account_id.clone());
            let account = service.get_account(&account_id).map_err(format_error)?;
            let network_status = service.get_network_status().map_err(format_error)?;

            let balance = service
                .get_balance_for_address(&address)
                .map_err(format_error)?;

            let balance_per_token = BalanceMap(
                balance
                    .iter()
                    .map(|(a, b)| (a.to_string(), Balance::from(b)))
                    .collect(),
            );

            JsonCommandResponse::get_address_status {
                address: Address::from(&subaddress),
                account_block_height: account.next_block_index.to_string(),
                network_block_height: network_status.network_block_height.to_string(),
                local_block_height: network_status.local_block_height.to_string(),
                balance_per_token,
            }
        }
        JsonCommandRequest::get_block {
            block_index,
            txo_public_key,
        } => {
            let (block, block_contents) = match (block_index, txo_public_key) {
                (None, None) => {
                    return Err(format_invalid_params_error(
                        "Must specify either block_index or txo_public_key",
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err(format_invalid_params_error(
                        "Must specify either block_index or txo_public_key, not both",
                    ))
                }
                (None, Some(txo_public_key)) => {
                    let public_key_bytes = hex::decode(txo_public_key).map_err(format_error)?;
                    let public_key: CompressedRistrettoPublic = public_key_bytes
                        .as_slice()
                        .try_into()
                        .map_err(format_error)?;
                    let block_index = service
                        .get_block_index_from_txo_public_key(&public_key)
                        .map_err(format_error)?;
                    service
                        .get_block_object(block_index)
                        .map_err(format_error)?
                }
                (Some(block_index), None) => service
                    .get_block_object(block_index.parse::<u64>().map_err(format_error)?)
                    .map_err(format_error)?,
            };

            let watcher_info = service
                .get_watcher_block_info(block.index)
                .map_err(format_error)?;

            JsonCommandResponse::get_block {
                block: Block::new(&block),
                block_contents: BlockContents::new(&block_contents),
                watcher_info: watcher_info.as_ref().map(Into::into),
            }
        }
        JsonCommandRequest::get_blocks {
            first_block_index,
            limit,
        } => {
            if limit > MAX_BLOCKS_PER_REQUEST {
                return Err(format_error(format!(
                    "Limit must be less than or equal to {MAX_BLOCKS_PER_REQUEST}"
                )));
            }

            let first_block_index = first_block_index.parse::<u64>().map_err(format_error)?;

            let blocks_and_contents = service
                .get_block_objects(first_block_index, limit)
                .map_err(format_error)?;

            let (blocks, block_contents): (Vec<_>, Vec<_>) = blocks_and_contents
                .iter()
                .map(|(block, block_contents)| {
                    (Block::new(block), BlockContents::new(block_contents))
                })
                .unzip();

            let watcher_infos = blocks_and_contents
                .iter()
                .map(|(block, _contents)| {
                    service
                        .get_watcher_block_info(block.index)
                        .map_err(format_error)
                })
                .collect::<Result<Vec<_>, _>>()?;

            JsonCommandResponse::get_blocks {
                blocks,
                block_contents,
                watcher_infos: watcher_infos
                    .iter()
                    .map(|info| info.as_ref().map(Into::into))
                    .collect(),
            }
        }
        JsonCommandRequest::get_recent_blocks { limit } => {
            let limit = limit.unwrap_or(RECENT_BLOCKS_DEFAULT_LIMIT);
            if limit > MAX_BLOCKS_PER_REQUEST {
                return Err(format_error(format!(
                    "Limit must be less than or equal to {MAX_BLOCKS_PER_REQUEST}"
                )));
            }

            let blocks_and_contents = service
                .get_recent_block_objects(limit)
                .map_err(format_error)?;

            let (blocks, block_contents): (Vec<_>, Vec<_>) = blocks_and_contents
                .iter()
                .map(|(block, block_contents)| {
                    (Block::new(block), BlockContents::new(block_contents))
                })
                .unzip();

            let watcher_infos = blocks_and_contents
                .iter()
                .map(|(block, _contents)| {
                    service
                        .get_watcher_block_info(block.index)
                        .map_err(format_error)
                })
                .collect::<Result<Vec<_>, _>>()?;

            let network_status =
                NetworkStatus::try_from(&service.get_network_status().map_err(format_error)?)
                    .map_err(format_error)?;

            JsonCommandResponse::get_recent_blocks {
                blocks,
                block_contents,
                watcher_infos: watcher_infos
                    .iter()
                    .map(|info| info.as_ref().map(Into::into))
                    .collect(),
                network_status,
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
        JsonCommandRequest::get_token_metadata => {
            let metadata_info = get_token_metadata().map_err(format_error)?;
            JsonCommandResponse::get_token_metadata {
                verified: metadata_info.verified,
                metadata: metadata_info.metadata,
            }
        }
        JsonCommandRequest::get_transaction_log { transaction_log_id } => {
            let (transaction_log, associated_txos, value_map) = service
                .get_transaction_log(&transaction_log_id)
                .map_err(format_error)?;
            JsonCommandResponse::get_transaction_log {
                transaction_log: TransactionLog::new(
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

            let transaction_log_map = Map::from_iter(
                transaction_logs_and_txos
                    .iter()
                    .map(|(t, a, v)| {
                        (
                            t.id.clone(),
                            serde_json::to_value(TransactionLog::new(t, a, v))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_transaction_logs {
                transaction_log_ids: transaction_logs_and_txos
                    .iter()
                    .map(|(t, _, _)| t.id.clone())
                    .collect(),
                transaction_log_map,
            }
        }
        JsonCommandRequest::get_txo { txo_id } => {
            let txo_info = service.get_txo(&TxoID(txo_id)).map_err(format_error)?;
            JsonCommandResponse::get_txo {
                txo: (&txo_info).into(),
            }
        }
        JsonCommandRequest::get_txo_block_index { public_key } => {
            let public_key_bytes = hex::decode(public_key).map_err(format_error)?;
            let public_key: CompressedRistrettoPublic = public_key_bytes
                .as_slice()
                .try_into()
                .map_err(format_error)?;
            let block_index = service
                .get_block_index_from_txo_public_key(&public_key)
                .map_err(format_error)?;
            JsonCommandResponse::get_txo_block_index {
                block_index: block_index.to_string(),
            }
        }
        JsonCommandRequest::get_txos {
            account_id,
            address,
            status,
            token_id,
            min_received_block_index,
            max_received_block_index,
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

            let min_received_block_index = match min_received_block_index {
                Some(i) => Some(i.parse::<u64>().map_err(format_error)?),
                None => None,
            };

            let max_received_block_index = match max_received_block_index {
                Some(i) => Some(i.parse::<u64>().map_err(format_error)?),
                None => None,
            };

            let txos_and_statuses = service
                .list_txos(
                    account_id,
                    address,
                    status,
                    token_id,
                    min_received_block_index,
                    max_received_block_index,
                    offset,
                    limit,
                )
                .map_err(format_error)?;

            let txo_map = Map::from_iter(
                txos_and_statuses
                    .iter()
                    .map(|txo_info| {
                        (
                            txo_info.txo.id.clone(),
                            serde_json::to_value(Txo::from(txo_info))
                                .expect("Could not get json value"),
                        )
                    })
                    .collect::<Vec<(String, serde_json::Value)>>(),
            );

            JsonCommandResponse::get_txos {
                txo_ids: txos_and_statuses
                    .into_iter()
                    .map(|txo_info| txo_info.txo.id)
                    .collect(),
                txo_map,
            }
        }
        JsonCommandRequest::get_txo_membership_proofs { outputs } => {
            let public_keys = outputs
                .clone()
                .into_iter()
                .map(|tx_out| {
                    let public_key_bytes = hex::decode(tx_out.public_key).map_err(format_error)?;
                    let public_key: CompressedRistrettoPublic = public_key_bytes
                        .as_slice()
                        .try_into()
                        .map_err(format_error)?;
                    Ok(public_key)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let indices = service
                .get_indices_from_txo_public_keys(&public_keys)
                .map_err(format_error)?;

            let membership_proofs = service
                .get_tx_out_proof_of_memberships(&indices)
                .map_err(format_error)?
                .iter()
                .map(|proof| {
                    let proof: mc_api::external::TxOutMembershipProof =
                        proof.try_into().map_err(format_error)?;
                    let json_proof = JsonTxOutMembershipProof::from(&proof);
                    Ok(json_proof)
                })
                .collect::<Result<Vec<_>, _>>()?;

            JsonCommandResponse::get_txo_membership_proofs {
                outputs,
                membership_proofs,
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
            name,
            first_block_index,
            next_subaddress_index,
            fog_info,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;
            let ns = next_subaddress_index
                .map(|ns| ns.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let fog_info = fog_info.unwrap_or_default();

            let account = service
                .import_account(
                    mnemonic,
                    name,
                    fb,
                    ns,
                    fog_info.report_url,
                    fog_info.authority_spki,
                )
                .map_err(format_error)?;

            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;

            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;

            JsonCommandResponse::import_account { account }
        }
        JsonCommandRequest::import_account_from_legacy_root_entropy {
            entropy,
            name,
            first_block_index,
            next_subaddress_index,
            fog_info,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;
            let ns = next_subaddress_index
                .map(|ns| ns.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let fog_info = fog_info.unwrap_or_default();

            let account = service
                .import_account_from_legacy_root_entropy(
                    entropy,
                    name,
                    fb,
                    ns,
                    fog_info.report_url,
                    fog_info.authority_spki,
                )
                .map_err(format_error)?;

            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;

            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;

            JsonCommandResponse::import_account { account }
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

            let mut view_private_key_bytes = [0u8; 32];
            hex::decode_to_slice(view_private_key, &mut view_private_key_bytes)
                .map_err(format_error)?;
            let view_private_key: RistrettoPrivate =
                (&view_private_key_bytes).try_into().map_err(format_error)?;

            let mut spend_public_key_bytes = [0u8; 32];
            hex::decode_to_slice(spend_public_key, &mut spend_public_key_bytes)
                .map_err(format_error)?;
            let spend_public_key: RistrettoPublic =
                (&spend_public_key_bytes).try_into().map_err(format_error)?;

            let account = service
                .import_view_only_account(
                    &view_private_key.into(),
                    &spend_public_key.into(),
                    name,
                    fb,
                    ns,
                )
                .map_err(format_error)?;
            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&AccountID(account.id.clone()))
                .map_err(format_error)?;
            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;
            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;

            JsonCommandResponse::import_view_only_account { account }
        }
        JsonCommandRequest::import_view_only_account_from_hardware_wallet {
            name,
            first_block_index,
            fog_info,
        } => {
            let fb = first_block_index
                .map(|fb| fb.parse::<u64>())
                .transpose()
                .map_err(format_error)?;

            let account = service
                .import_view_only_account_from_hardware_wallet(name, fb, fog_info)
                .await
                .map_err(format_error)?;

            let next_subaddress_index = 1;

            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;

            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;

            JsonCommandResponse::import_view_only_account_from_hardware_wallet { account }
        }
        JsonCommandRequest::remove_account { account_id } => JsonCommandResponse::remove_account {
            removed: service
                .remove_account(&AccountID(account_id))
                .map_err(format_error)?,
        },
        JsonCommandRequest::resync_account { account_id } => {
            service
                .resync_account(&AccountID(account_id))
                .map_err(format_error)?;

            JsonCommandResponse::resync_account
        }
        JsonCommandRequest::sample_mixins {
            num_mixins,
            excluded_outputs,
        } => {
            let public_keys = excluded_outputs
                .into_iter()
                .map(|tx_out| {
                    let public_key_bytes = hex::decode(tx_out.public_key).map_err(format_error)?;
                    let public_key: CompressedRistrettoPublic = public_key_bytes
                        .as_slice()
                        .try_into()
                        .map_err(format_error)?;
                    Ok(public_key)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let excluded_indices = service
                .get_indices_from_txo_public_keys(&public_keys)
                .map_err(format_error)?;
            let (mixins, membership_proofs) = service
                .sample_mixins(num_mixins as usize, &excluded_indices)
                .map_err(format_error)?;

            let mixins = mixins
                .iter()
                .map(|tx_out| {
                    let tx_out: mc_api::external::TxOut =
                        tx_out.try_into().map_err(format_error)?;
                    let json_tx_out = JsonTxOut::from(&tx_out);
                    Ok(json_tx_out)
                })
                .collect::<Result<Vec<_>, _>>()?;

            let membership_proofs = membership_proofs
                .iter()
                .map(|proof| {
                    let proof: mc_api::external::TxOutMembershipProof =
                        proof.try_into().map_err(format_error)?;
                    let json_proof = JsonTxOutMembershipProof::from(&proof);
                    Ok(json_proof)
                })
                .collect::<Result<Vec<_>, _>>()?;

            JsonCommandResponse::sample_mixins {
                mixins,
                membership_proofs,
            }
        }
        JsonCommandRequest::search_ledger { query } => {
            let results = service.search_ledger(&query).map_err(format_error)?;
            JsonCommandResponse::search_ledger {
                results: results.iter().map(Into::into).collect(),
            }
        }
        JsonCommandRequest::submit_transaction {
            tx_proposal,
            comment,
            account_id,
        } => {
            let tx_proposal = TxProposal::try_from(&tx_proposal).map_err(format_error)?;
            let result: Option<TransactionLog> = service
                .submit_transaction(&tx_proposal, comment, account_id)
                .map_err(format_error)?
                .map(|(transaction_log, associated_txos, value_map)| {
                    TransactionLog::new(&transaction_log, &associated_txos, &value_map)
                });
            JsonCommandResponse::submit_transaction {
                transaction_log: result,
            }
        }
        JsonCommandRequest::sync_view_only_account {
            account_id,
            synced_txos,
        } => {
            let synced_txos = match synced_txos {
                Some(synced_txos) => synced_txos,
                None => {
                    let account = service
                        .get_account(&AccountID(account_id.clone()))
                        .map_err(format_error)?;
                    let view_account_keys = account.view_account_key().map_err(format_error)?;

                    let unverified_txos = service
                        .list_txos(
                            Some(account_id.clone()),
                            None,
                            Some(TxoStatus::Unverified),
                            None,
                            None,
                            None,
                            None,
                            None,
                        )
                        .map_err(format_error)?;

                    let unsynced_txos = unverified_txos
                        .iter()
                        .map(|txo_info| {
                            let subaddress_index = match txo_info.txo.subaddress_index {
                                Some(subaddress_index) => subaddress_index,
                                None => {
                                    return Err(format_error(
                                        "Unsynced txo is missing subaddress index, which is required to sync",
                                    ))
                                }
                            };
                            let txo = service.get_txo_object(&txo_info.txo.id).map_err(format_error)?;
                            Ok((txo, subaddress_index as u64))
                        })
                        .collect::<Result<Vec<_>, JsonRPCError>>()?;

                    sync_txos(unsynced_txos, &view_account_keys)
                        .await
                        .map_err(format_error)?
                }
            };

            service
                .sync_account(&AccountID(account_id), synced_txos)
                .map_err(format_error)?;

            JsonCommandResponse::sync_view_only_account
        }
        JsonCommandRequest::update_account_name { account_id, name } => {
            let account_id = AccountID(account_id);
            let account = service
                .update_account_name(&account_id, name)
                .map_err(format_error)?;
            let next_subaddress_index = service
                .get_next_subaddress_index_for_account(&account_id)
                .map_err(format_error)?;
            let main_public_address: mc_account_keys::PublicAddress = (&service
                .get_address_for_account(
                    &account.id.clone().into(),
                    DEFAULT_SUBADDRESS_INDEX as i64,
                )
                .map_err(format_error)?)
                .try_into()
                .map_err(format_error)?;
            let account = Account::new(&account, &main_public_address, next_subaddress_index)
                .map_err(format_error)?;
            JsonCommandResponse::update_account_name { account }
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
        JsonCommandRequest::validate_sender_memo {
            txo_id,
            sender_address,
        } => {
            let result = service
                .validate_sender_memo(&txo_id, &sender_address)
                .map_err(format_error)?;
            JsonCommandResponse::validate_sender_memo { validated: result }
        }
        JsonCommandRequest::verify_address { address } => match service.verify_address(&address) {
            Ok(public_address) => JsonCommandResponse::verify_address {
                verified: true,
                short_address_hash: Some(hex::encode(
                    ShortAddressHash::from(&public_address).as_ref(),
                )),
            },
            Err(_) => JsonCommandResponse::verify_address {
                verified: false,
                short_address_hash: None,
            },
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
