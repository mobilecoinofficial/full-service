// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON-RPC Responses from the Wallet API.
//!
//! API v2

use crate::{
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        json_rpc_response::JsonCommandResponse as JsonCommandResponseTrait,
        v2::models::{
            account::{Account, AccountMap},
            account_secrets::AccountSecrets,
            address::{Address, AddressMap},
            balance::BalanceMap,
            block::{Block, BlockContents},
            confirmation_number::Confirmation,
            ledger::LedgerSearchResult,
            network_status::NetworkStatus,
            public_address::PublicAddress,
            receiver_receipt::ReceiverReceipt,
            signed_contingent_input::ValidateProofOfReserveSciResult,
            transaction_log::TransactionLog,
            tx_proposal::{TxProposal, UnsignedTxProposal},
            txo::Txo,
            wallet_status::WalletStatus,
            watcher::WatcherBlockInfo,
        },
    },
    service::receipt::ReceiptTransactionStatus,
    util::b58::PrintableWrapperType,
};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxOutMembershipProof};
use mc_transaction_signer::types::TxoSyncReq;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::HashMap;

/// Responses from the Full Service Wallet.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
#[allow(clippy::large_enum_variant)]
pub enum JsonCommandResponse {
    assign_address_for_account {
        address: Address,
    },
    build_and_submit_transaction {
        transaction_log: TransactionLog,
        tx_proposal: TxProposal,
    },
    build_burn_transaction {
        tx_proposal: TxProposal,
        transaction_log_id: String,
    },
    build_transaction {
        tx_proposal: TxProposal,
        transaction_log_id: String,
    },
    build_unsigned_burn_transaction {
        account_id: String,
        unsigned_tx_proposal: UnsignedTxProposal,
    },
    build_unsigned_transaction {
        account_id: String,
        unsigned_tx_proposal: UnsignedTxProposal,
    },
    check_b58_type {
        b58_type: PrintableWrapperType,
        data: HashMap<String, String>,
    },
    check_receiver_receipt_status {
        receipt_transaction_status: ReceiptTransactionStatus,
        txo: Option<Txo>,
    },
    create_account {
        account: Account,
    },
    create_payment_request {
        payment_request_b58: String,
    },
    create_receiver_receipts {
        receiver_receipts: Vec<ReceiverReceipt>,
    },
    create_view_only_account_import_request {
        json_rpc_request: JsonRPCRequest,
    },
    create_view_only_account_sync_request {
        txo_sync_request: TxoSyncReq,
    },
    export_account_secrets {
        account_secrets: AccountSecrets,
    },
    get_account_status {
        account: Account,
        network_block_height: String,
        local_block_height: String,
        balance_per_token: BalanceMap,
    },
    get_accounts {
        account_ids: Vec<String>,
        account_map: AccountMap,
    },
    get_address_details {
        details: PublicAddress,
        address_hash: String,
    },
    get_address {
        address: Address,
    },
    get_address_for_account {
        address: Address,
    },
    get_addresses {
        public_addresses: Vec<String>,
        address_map: AddressMap,
    },
    get_address_status {
        address: Address,
        account_block_height: String,
        network_block_height: String,
        local_block_height: String,
        balance_per_token: BalanceMap,
    },
    get_block {
        block: Block,
        block_contents: BlockContents,
        watcher_info: Option<WatcherBlockInfo>,
    },
    get_blocks {
        blocks: Vec<Block>,
        block_contents: Vec<BlockContents>,
        watcher_infos: Vec<Option<WatcherBlockInfo>>,
    },
    get_recent_blocks {
        blocks: Vec<Block>,
        block_contents: Vec<BlockContents>,
        watcher_infos: Vec<Option<WatcherBlockInfo>>,
        network_status: NetworkStatus,
    },
    get_confirmations {
        confirmations: Vec<Confirmation>,
    },
    get_mc_protocol_transaction {
        transaction: JsonTx,
    },
    get_mc_protocol_txo {
        txo: JsonTxOut,
    },
    get_network_status {
        network_status: NetworkStatus,
    },
    get_token_metadata {
        verified: bool,
        metadata: String,
    },
    get_transaction_log {
        transaction_log: TransactionLog,
    },
    get_transaction_logs {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_txo {
        txo: Txo,
    },
    get_txo_block_index {
        block_index: String,
    },
    get_txos {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_txo_membership_proofs {
        outputs: Vec<JsonTxOut>,
        membership_proofs: Vec<JsonTxOutMembershipProof>,
    },
    get_wallet_status {
        wallet_status: WalletStatus,
    },
    import_account {
        account: Account,
    },
    import_account_from_legacy_root_entropy {
        account: Account,
    },
    import_account_from_private_keys {
        account: Account,
    },
    import_view_only_account {
        account: Account,
    },
    import_view_only_account_from_hardware_wallet {
        account: Account,
    },
    remove_account {
        removed: bool,
    },
    resync_account,
    sample_mixins {
        mixins: Vec<JsonTxOut>,
        membership_proofs: Vec<JsonTxOutMembershipProof>,
    },
    search_ledger {
        results: Vec<LedgerSearchResult>,
    },
    set_require_spend_subaddress {
        account: Account,
    },
    submit_transaction {
        transaction_log: Option<TransactionLog>,
    },
    sync_view_only_account,
    update_account_name {
        account: Account,
    },
    validate_confirmation {
        validated: bool,
    },
    validate_sender_memo {
        validated: bool,
    },
    validate_proof_of_reserve_sci {
        #[serde(flatten)]
        result: ValidateProofOfReserveSciResult,
    },
    verify_address {
        verified: bool,
        address_hash: Option<String>,
    },
    version {
        string: String,
        number: (String, String, String, String),
        commit: String,
    },
}

impl JsonCommandResponseTrait for JsonCommandResponse {}
