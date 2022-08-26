use crate::{
    db::{
        account::AccountModel,
        models::{Account, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        Conn,
    },
    json_rpc::v1::models::{
        account::Account as AccountJSON, account_secrets::AccountSecrets,
        transaction_log::TransactionLog as JsonTransactionLog, txo::Txo as JsonTxo,
    },
};
use bip39::Mnemonic;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

pub fn import_accounts(conn: &Conn, client: &reqwest::blocking::Client, request_uri: &str) {
    let get_accounts_body = json!({
        "method": "get_all_accounts",
        "jsonrpc": "2.0",
        "id": 1
    });

    let response = client
        .post(request_uri.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(get_accounts_body.to_string())
        .send()
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();

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
    let get_account_secrets_body = json!({
        "method": "export_account_secrets",
        "jsonrpc": "2.0",
        "id": 1,
        "params": {
            "account_id": account.account_id
        }
    });

    let response = client
        .post(request_uri.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(get_account_secrets_body.to_string())
        .send()
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();

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
        .unwrap();
    } else if let Some(entropy) = account_secrets.entropy {
        panic!("Entropy not yet supported");
    } else {
        panic!(
            "No entropy or mnemonic found for account {}",
            account.account_id
        );
    };

    let get_sent_transaction_logs_body = json!({
        "method": "get_sent_transaction_logs_for_account",
        "jsonrpc": "2.0",
        "id": 1,
        "params": {
            "account_id": account.account_id
        }
    });

    let response = client
        .post(request_uri.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(get_sent_transaction_logs_body.to_string())
        .send()
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();

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
    for input_txo in &transaction_log.input_txos {
        let get_txo_body = json!({
            "method": "get_txo",
            "jsonrpc": "2.0",
            "id": 1,
            "params": {
                "txo_id": input_txo.txo_id_hex
            }
        });

        let response = client
            .post(request_uri.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(get_txo_body.to_string())
            .send()
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap();

        let txo: JsonTxo =
            serde_json::from_value(response.get("result").unwrap().get("txo").unwrap().clone())
                .unwrap();

        Txo::import_from_v1(txo, conn).unwrap();
    }

    for input_txo in &transaction_log.output_txos {
        let get_txo_body = json!({
            "method": "get_txo",
            "jsonrpc": "2.0",
            "id": 1,
            "params": {
                "txo_id": input_txo.txo_id_hex
            }
        });

        let response = client
            .post(request_uri.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(get_txo_body.to_string())
            .send()
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap();

        let txo: JsonTxo =
            serde_json::from_value(response.get("result").unwrap().get("txo").unwrap().clone())
                .unwrap();

        Txo::import_from_v1(txo, conn).unwrap();
    }

    for input_txo in &transaction_log.change_txos {
        let get_txo_body = json!({
            "method": "get_txo",
            "jsonrpc": "2.0",
            "id": 1,
            "params": {
                "txo_id": input_txo.txo_id_hex
            }
        });

        let response = client
            .post(request_uri.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(get_txo_body.to_string())
            .send()
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap();

        let txo: JsonTxo =
            serde_json::from_value(response.get("result").unwrap().get("txo").unwrap().clone())
                .unwrap();

        Txo::import_from_v1(txo, conn).unwrap();
    }

    TransactionLog::log_imported_from_v1(transaction_log.clone(), conn).unwrap();
}
