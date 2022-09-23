use crate::{
    db::{
        account::AccountModel,
        models::{Account, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        Conn,
    },
    json_rpc::v1::models::{
        account::Account as AccountJSON,
        account_secrets::AccountSecrets,
        transaction_log::{TransactionLog as JsonTransactionLog, TxoAbbrev},
        txo::Txo as JsonTxo,
    },
};
use bip39::Mnemonic;
use mc_account_keys::RootEntropy;
use mc_mobilecoind_json::data_types::JsonTx;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};
use std::convert::TryFrom;

pub fn import_accounts(conn: &Conn, client: &reqwest::blocking::Client, request_uri: &str) {
    let body = json!({
        "method": "get_all_accounts",
        "jsonrpc": "2.0",
        "id": 1
    });

    let response = send_request(client, request_uri, body);

    let account_map = response.get("result").unwrap().get("account_map").unwrap();
    let account_ids = response
        .get("result")
        .unwrap()
        .get("account_ids")
        .unwrap()
        .as_array()
        .unwrap();

    for account_id in account_ids {
        let account: AccountJSON = serde_json::from_value(
            account_map
                .get(account_id.as_str().unwrap())
                .unwrap()
                .clone(),
        )
        .unwrap();

        import_account(&account, conn, request_uri, client);
    }
}

fn import_account(
    account: &AccountJSON,
    conn: &Conn,
    request_uri: &str,
    client: &reqwest::blocking::Client,
) {
    let body = json!({
        "method": "export_account_secrets",
        "jsonrpc": "2.0",
        "id": 1,
        "params": {
            "account_id": account.account_id
        }
    });

    let response = send_request(client, request_uri, body);

    let account_secrets: AccountSecrets = serde_json::from_value(
        response
            .get("result")
            .unwrap()
            .get("account_secrets")
            .unwrap()
            .clone(),
    )
    .unwrap();

    if let Some(mnemonic) = account_secrets.mnemonic {
        let mnemonic = Mnemonic::from_phrase(&mnemonic, bip39::Language::English).unwrap();
        Account::import(
            &mnemonic,
            Some(account.name.clone()),
            0,
            Some(account.first_block_index.parse::<u64>().unwrap()),
            Some(account.next_subaddress_index.parse::<u64>().unwrap()),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            conn,
        )
        .unwrap_or_else(|| return);
    } else if let Some(entropy) = account_secrets.entropy {
        let entropy_bytes = hex::decode(entropy).unwrap();
        let entropy = RootEntropy::try_from(entropy_bytes.as_slice()).unwrap();
        Account::import_legacy(
            &entropy,
            Some(account.name.clone()),
            0,
            Some(account.first_block_index.parse::<u64>().unwrap()),
            Some(account.next_subaddress_index.parse::<u64>().unwrap()),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            conn,
        )
        .unwrap_or_else(|| return);
    } else {
        println!(
            "No entropy or mnemonic found for account {}",
            account.account_id
        );
        return;
    };

    let body = json!({
        "method": "get_sent_transaction_logs_for_account",
        "jsonrpc": "2.0",
        "id": 1,
        "params": {
            "account_id": account.account_id
        }
    });

    let response = send_request(client, request_uri, body);

    let transaction_log_ids = response
        .get("result")
        .unwrap()
        .get("transaction_log_ids")
        .unwrap()
        .as_array()
        .unwrap();

    for transaction_log_id in transaction_log_ids {
        let transaction_log: JsonTransactionLog = serde_json::from_value(
            response
                .get("result")
                .unwrap()
                .get("transaction_log_map")
                .unwrap()
                .get(transaction_log_id.as_str().unwrap())
                .unwrap()
                .clone(),
        )
        .unwrap();

        import_tx_log(&transaction_log, conn, request_uri, client);
    }
}

fn import_tx_log(
    transaction_log: &JsonTransactionLog,
    conn: &Conn,
    request_uri: &str,
    client: &reqwest::blocking::Client,
) {
    import_txos(&transaction_log.input_txos, client, conn, request_uri);
    import_txos(&transaction_log.output_txos, client, conn, request_uri);
    import_txos(&transaction_log.change_txos, client, conn, request_uri);

    let body = json!({
        "method": "get_mc_protocol_transaction",
        "jsonrpc": "2.0",
        "id": 1,
        "params": {
            "transaction_log_id": transaction_log.transaction_log_id
        }
    });

    let response = send_request(client, request_uri, body);

    let tx_json: JsonTx = serde_json::from_value(
        response
            .get("result")
            .unwrap()
            .get("transaction")
            .unwrap()
            .clone(),
    )
    .unwrap();

    let tx = mc_api::external::Tx::try_from(&tx_json).unwrap();
    let tx = mc_transaction_core::tx::Tx::try_from(&tx).unwrap();
    let tx_proto_bytes = mc_util_serial::encode(&tx);

    TransactionLog::log_imported_from_v1(transaction_log.clone(), tx_proto_bytes.as_slice(), conn)
        .unwrap();
}

fn import_txos(
    txo_abbrevs: &[TxoAbbrev],
    client: &reqwest::blocking::Client,
    conn: &Conn,
    request_uri: &str,
) {
    for txo_abbrev in txo_abbrevs {
        let body = json!({
            "method": "get_txo",
            "jsonrpc": "2.0",
            "id": 1,
            "params": {
                "txo_id": txo_abbrev.txo_id_hex
            }
        });

        let response = send_request(client, request_uri, body);

        let txo: JsonTxo =
            serde_json::from_value(response.get("result").unwrap().get("txo").unwrap().clone())
                .unwrap();

        Txo::import_from_v1(txo, conn).unwrap();
    }
}

fn send_request(client: &reqwest::blocking::Client, request_uri: &str, body: Value) -> Value {
    client
        .post(request_uri)
        .header(CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send()
        .unwrap()
        .json::<Value>()
        .unwrap()
}
