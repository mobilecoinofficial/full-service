// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON-RPC Responses from the Wallet API.

use crate::json_rpc::{
    Account, Address, Block, BlockContents, MembershipProof, TransactionLog, Txo, WalletStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
/// A JSON RPC response.
pub enum Response {
    create_account {
        entropy: String,
        account: Account,
    },
    import_account {
        account: Account,
    },
    get_all_accounts {
        account_ids: Vec<String>,
        account_map: Map<String, serde_json::Value>,
    },
    get_account {
        account: Account,
    },
    update_account_name {
        account: Account,
    },
    delete_account {
        success: bool,
    },
    get_all_txos_by_account {
        txo_ids: Vec<String>,
        txo_map: Map<String, serde_json::Value>,
    },
    get_txo {
        txo: Txo,
    },
    get_wallet_status {
        status: WalletStatus,
    },
    get_balance {
        status: BalanceResponse,
    },
    create_address {
        address: Address,
    },
    get_all_addresses_by_account {
        address_ids: Vec<String>,
        address_map: Map<String, serde_json::Value>,
    },
    send_transaction {
        transaction: SubmitResponse,
    },
    build_transaction {
        tx_proposal: mc_mobilecoind_json::data_types::JsonTxProposal,
    },
    submit_transaction {
        transaction: SubmitResponse,
    },
    get_all_transactions_by_account {
        transaction_log_ids: Vec<String>,
        transaction_log_map: Map<String, serde_json::Value>,
    },
    get_transaction {
        transaction: TransactionLog,
    },
    get_transaction_object {
        transaction: mc_mobilecoind_json::data_types::JsonTx,
    },
    get_txo_object {
        txo: mc_mobilecoind_json::data_types::JsonTxOut,
    },
    get_block_object {
        block: Block,
        block_contents: BlockContents,
    },
    get_proofs {
        proofs: Vec<MembershipProof>,
    },
    verify_proof {
        verified: bool,
    },
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct CreateAccountResponse {
    pub entropy: String,
    pub account: Account,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct BalanceResponse {
    pub unspent: String,
    pub pending: String,
    pub spent: String,
    pub secreted: String,
    pub orphaned: String,
    pub local_block_count: String,
    pub synced_blocks: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct SubmitResponse {
    pub transaction_id: String,
}
