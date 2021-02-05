// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON-RPC Responses from the Wallet API.

use crate::api::{
    JsonAccount, JsonAddress, JsonBlock, JsonBlockContents, JsonProof, JsonTransactionLog, JsonTxo,
    JsonWalletStatus,
};
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut, JsonTxProposal};
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
/// A JSON RPC response.
pub enum Response {
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

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct JsonCreateAccountResponse {
    pub entropy: String,
    pub account: JsonAccount,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonBalanceResponse {
    pub unspent: String,
    pub pending: String,
    pub spent: String,
    pub secreted: String,
    pub orphaned: String,
    pub local_block_count: String,
    pub synced_blocks: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonSubmitResponse {
    pub transaction_id: String,
}
