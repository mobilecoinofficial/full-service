// Minimum supported NodeJS version: v12.9.0
const NODE_MAJOR_VERSION = process.versions.node.split('.')[0];
if (NODE_MAJOR_VERSION < 12) {
    throw new Error('Requires Node 12 (or higher)');
}

// Imports
const client = require('../test_lib/send-request-encrypted');
const full_service_path = "/wallet";
const public_mirror_path = "/encrypted-request";
const wait_time_ms = 10000;
const fields = ["method", "params", "jsonrpc", "id", "block_index", "mnemonic", "key_derivation_version", "name", "account_id", "recipient_public_address", "value_pmob", "offset", "limit", "subaddress_index", "memo", "amount_pmob", "address", "transaction_log_id", "txo_id", "confirmation"];

function getFuncName() {
    return getFuncName.caller.name
}
const timer = (ms) => new Promise((res) => setTimeout(res, ms));
async function runAllTests(public_mirror_host, public_mirror_port, full_service_host, full_service_port, key_file, mnemonic) {
    try {
        await testGetBlock(public_mirror_host, public_mirror_port, key_file);
        let accountJSON = await testImportAccount(full_service_host, full_service_port, mnemonic);
        let accountInfo = JSON.parse(accountJSON);
        let accountId = accountInfo["result"]["account"]["account_id"];
        let mainAddress = accountInfo["result"]["account"]["main_address"];
        console.log(`account_id: ${accountId}, main_address: ${mainAddress}`);
        let balanceJSON = await waitForBalanceToBeSynced(public_mirror_host, public_mirror_port, key_file, accountId);
        let balanceInfo = JSON.parse(balanceJSON);
        localBlockHeight = balanceInfo["result"]["balance"]["local_block_height"];
        accountBlockHeight = balanceInfo["result"]["balance"]["account_block_height"];
        console.log(balanceJSON);

        let transactionJSON = await testBuildAndSubmitTransaction(full_service_host, full_service_port, accountId, mainAddress, "1");
        let transactionInfo = JSON.parse(transactionJSON);
        let transactionLogId = transactionInfo["result"]["transaction_log"]["transaction_log_id"];
        let transactionBlockIndex = transactionInfo["result"]["transaction_log"]["submitted_block_index"];
        let outputTxoHex = transactionInfo["result"]["transaction_log"]["output_txos"][0]["txo_id_hex"];
        console.log(`transactionLogId: ${transactionLogId}`);
        console.log(`transaction block: ${transactionBlockIndex}`);
        console.log(`outputTxo: ${outputTxoHex}`);
        let addresses = await testGetAddressesForAccount(public_mirror_host, public_mirror_port, key_file, accountId);
        let accountStatus = await testGetAccountStatus(public_mirror_host, public_mirror_port, key_file, accountId);
        let paymentRequest = await testCreatePaymentRequest(public_mirror_host, public_mirror_port, key_file, accountId, 1, 1);
        let balance = await testGetBalanceForAddress(public_mirror_host, public_mirror_port, key_file, mainAddress);
        let addressVerification = await testVerifyAddress(public_mirror_host, public_mirror_port, key_file, mainAddress);
        let walletStatus = await testWalletStatus(public_mirror_host, public_mirror_port, key_file);
        let networkStatus = await testNetworkStatus(public_mirror_host, public_mirror_port, key_file);
        let transactionLogs = await testGetTransactionLogsForBlock(public_mirror_host, public_mirror_port, key_file, transactionBlockIndex);
        transactionLogs = await testGetTransactionLogsForAccount(public_mirror_host, public_mirror_port, key_file, accountId, "1", "1");
        transactionLogs = await waitForTransactionToBeSynced(public_mirror_host, public_mirror_port, key_file, transactionLogId);
        let confirmationJSON = await testGetConfirmations(public_mirror_host, public_mirror_port, key_file, transactionLogId);
        let confirmationInfo = JSON.parse(confirmationJSON);
        console.log(`confirmationJson: ${confirmationJSON}`);
        let confirmation = confirmationInfo["result"]["confirmations"][0]["confirmation"];
        console.log(`confirmation: ${confirmation}`);
        let validation = await testValidateConfirmations(public_mirror_host, public_mirror_port, key_file, accountId, outputTxoHex, confirmation);;

    }
    catch (error) {
        console.error(`Test failed: ${error}`);
        throw "Failed";
    }
}


async function testImportAccount(full_service_host, full_service_port, mnemonic) {
    try {
        let request = {
            method: "import_account",
            params: {
                mnemonic: mnemonic,
                key_derivation_version: "2",
                name: "Bob"
            },
            jsonrpc: "2.0",
            id: 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequestUnencrypted(full_service_host, full_service_port, full_service_path, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testBuildAndSubmitTransaction(full_service_host, full_service_port, account_id, recipient_address, value_pmob) {
    try {
        let request = {
            "method": "build_and_submit_transaction",
            "params": {
                "account_id": account_id,
                "recipient_public_address": recipient_address,
                "value_pmob": value_pmob
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequestUnencrypted(full_service_host, full_service_port, full_service_path, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}


async function testGetBlock(public_mirror_host, public_mirror_port, key_file) {
    try {
        let request = {
            method: "get_block",
            params: {
                block_index: "0"
            },
            jsonrpc: "2.0",
            id: 1
        };
        let requestString = JSON.stringify(request, fields);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testGetBalanceForAccount(public_mirror_host, public_mirror_port, key_file, account_id) {
    try {
        let request = {
            "method": "get_balance_for_account",
            "params": {
                "account_id": account_id
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function waitForBalanceToBeSynced(public_mirror_host, public_mirror_port, key_file, account_id) {
    let balanceJSON = await testGetBalanceForAccount(public_mirror_host, public_mirror_port, key_file, account_id);
    let balanceInfo = JSON.parse(balanceJSON);
    let balance_synced = balanceInfo["result"]["balance"]["is_synced"];
    while (!balance_synced) {
        await timer(wait_time_ms);
        balanceJSON = await testGetBalanceForAccount(public_mirror_host, public_mirror_port, key_file, account_id);
        balanceInfo = JSON.parse(balanceJSON);
        balance_synced = balanceInfo["result"]["balance"]["is_synced"];
    }
    return balanceJSON;
}
async function testGetAddressesForAccount(public_mirror_host, public_mirror_port, key_file, account_id) {
    try {
        let request = {
            "method": "get_addresses_for_account",
            "params": {
                "account_id": account_id,
                "offset": "1",
                "limit": "1000"
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testGetAccountStatus(public_mirror_host, public_mirror_port, key_file, account_id) {
    try {
        let request = {
            "method": "get_account_status",
            "params": {
                "account_id": account_id
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testCreatePaymentRequest(public_mirror_host, public_mirror_port, key_file, account_id, amount_pmob, subaddress_index) {
    try {
        let request = {
            "method": "create_payment_request",
            "params": {
                "account_id": account_id,
                "amount_pmob": amount_pmob,
                "subaddress_index": subaddress_index,
                "memo": "testCreatePaymentRequest"
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testGetBalanceForAddress(public_mirror_host, public_mirror_port, key_file, address) {
    try {
        let request = {
            "method": "get_balance_for_address",
            "params": {
                "address": address
            },
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testVerifyAddress(public_mirror_host, public_mirror_port, key_file, address) {
    try {
        let request = {
            "method": "verify_address",
            "params": {
                "address": address
            },
            "jsonrpc": "2.0",
            "id": 1
        };

        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}

async function testWalletStatus(public_mirror_host, public_mirror_port, key_file) {
    try {
        let request = {
            "method": "get_wallet_status",
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}

async function testNetworkStatus(public_mirror_host, public_mirror_port, key_file) {
    try {
        let request = {
            "method": "get_network_status",
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testGetTransactionLogsForBlock(public_mirror_host, public_mirror_port, key_file, block_index) {
    try {
        let request = {
            "method": "get_all_transaction_logs_for_block",
            "params": {
                "block_index": block_index
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}

async function testGetTransactionLogsForAccount(public_mirror_host, public_mirror_port, key_file, account_id, offset, limit) {
    try {
        let request = {
            "method": "get_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
                "offset": offset,
                "limit": limit
            },
            "jsonrpc": "2.0",
            "id": 1
        };
        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}

async function testGetTransactionLogsById(public_mirror_host, public_mirror_port, key_file, transaction_log_id) {
    try {
        let request = {
            "method": "get_transaction_log",
            "params": {
                "transaction_log_id": transaction_log_id
            },
            "jsonrpc": "2.0",
            "id": 1
        };

        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function waitForTransactionToBeSynced(public_mirror_host, public_mirror_port, key_file, transaction_log_id) {
    let transactionLogsJSON = await testGetTransactionLogsById(public_mirror_host, public_mirror_port, key_file, transaction_log_id);
    let transactionInfo = JSON.parse(transactionLogsJSON);
    let transaction_status = transactionInfo["result"]["transaction_log"]["status"];
    while (transaction_status === "tx_status_pending") {
        await timer(wait_time_ms);
        transactionLogsJSON = await testGetTransactionLogsById(public_mirror_host, public_mirror_port, key_file, transaction_log_id);
        transactionInfo = JSON.parse(transactionLogsJSON);
        transaction_status = transactionInfo["result"]["transaction_log"]["status"];
    }
    return transactionLogsJSON;
}
async function testGetConfirmations(public_mirror_host, public_mirror_port, key_file, transaction_log_id) {
    try {
        let request = {
            "method": "get_confirmations",
            "params": {
                "transaction_log_id": transaction_log_id
            },
            "jsonrpc": "2.0",
            "id": 1
        };

        let requestString = JSON.stringify(request, fields);
        console.log(requestString);
        return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
    }
    catch (error) {
        throw `Error in ${getFuncName()}: ${error}`;
    }
}
async function testValidateConfirmations(public_mirror_host, public_mirror_port, key_file, account_id, txo_id, confirmation) {
    for (let i = 0; i < 3; i++) {
        try {
            let request = {
                "method": "validate_confirmation",
                "params": {
                    "account_id": account_id,
                    "txo_id": txo_id,
                    "confirmation": confirmation
                },
                "jsonrpc": "2.0",
                "id": 1
            };
            let requestString = JSON.stringify(request, fields);
            console.log(requestString);
            return await client.sendRequest(public_mirror_host, public_mirror_port, key_file, requestString);
        }
        catch (error) {
            if (i >= 3) {
                throw `Error in ${getFuncName()}: ${error}`;
            }
            await timer(wait_time_ms);
        }
    }
}
console.log("Starting test script")
// Command line parsing
if (process.argv.length != 8) {
    console.log(`Usage: node test_script.js <public mirror host> <public mirror port> <full_service_host> <full_service_port> <key file> <mnemonic>`);
    console.log(`For example: node test_script.js 127.0.0.1 9091 127.0.0.1 5554 mirror-client.pem '<mnemonic>'`);
    console.log('To generate keys please run the generate-rsa-keypair binary. See README.md for more details')
    throw "invalid arguments";
}

let public_mirror_host = process.argv[2];
let public_mirror_port = process.argv[3];
let full_service_host = process.argv[4];
let full_service_port = process.argv[5];
let key_file = process.argv[6];
let mnemonic = process.argv[7];


runAllTests(public_mirror_host, public_mirror_port, full_service_host, full_service_port, key_file, mnemonic).then(result => {
    console.log("Run all tests succeeded");
}).catch((error) => {
    console.log("Run all tests had an error: " + error)
});