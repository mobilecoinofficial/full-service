// Copyright (c) 2020-2021 MobileCoin Inc.

//! JSON-RPC Requests to the Wallet API.

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
/// An RPC call to the wallet service is represented by sending a Request to the server.
pub enum Request {
    create_account {
        name: Option<String>,
        first_block: Option<String>,
    },
    import_account {
        entropy: String,
        name: Option<String>,
        first_block: Option<String>,
    },
    get_all_accounts,
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
    get_all_txos_by_account {
        account_id: String,
    },
    get_txo {
        txo_id: String,
    },
    get_wallet_status,
    get_balance {
        account_id: String,
    },
    create_address {
        account_id: String,
        comment: Option<String>,
    },
    get_all_addresses_by_account {
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
        tx_proposal: mc_mobilecoind_json::data_types::JsonTxProposal,
        comment: Option<String>,
        account_id: Option<String>,
    },
    get_all_transactions_by_account {
        account_id: String,
    },
    get_transaction {
        transaction_log_id: String,
    },
    get_transaction_object {
        transaction_log_id: String,
    },
    get_txo_object {
        txo_id: String,
    },
    get_block_object {
        block_index: String,
    },
    get_proofs {
        transaction_log_id: String,
    },
    verify_proof {
        account_id: String,
        txo_id: String,
        proof: String,
    },
}
