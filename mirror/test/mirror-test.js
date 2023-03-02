// Minimum supported NodeJS version: v12.9.0
const NODE_MAJOR_VERSION = process.versions.node.split('.')[0]
if (NODE_MAJOR_VERSION < 12) {
  throw new Error('Requires Node 12 (or higher)')
}

// Imports
const commander = require('commander')
const client = require('./test_lib/send-request-encrypted')
const { ResponseError } = require('./test_lib/send-request-encrypted')

// Logger
const pino = require('pino')
const pretty = require('pino-pretty')
const logLevel = process.env.LOG_LEVEL || 'info'
const logger = pino({ level: logLevel }, pretty())

const fullServicePath = '/wallet/v2'
const waitTimeMS = 10000
const fields = ['method', 'params', 'jsonrpc', 'id', 'block_index', 'mnemonic', 'key_derivation_version', 'name', 'account_id', 'recipient_public_address', 'value_pmob', 'offset', 'limit', 'subaddress_index', 'memo', 'amount_pmob', 'address', 'transaction_log_id', 'txo_id', 'confirmation', 'amount', 'value', 'token_id']

const timer = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

async function runAllTests (publicMirrorURL, fullServiceURL, keyFile, mnemonic) {
  try {
    let accountJSON

    logger.info('TEST: get_block')
    await testGetBlock(publicMirrorURL, keyFile)
    if (fullServiceURL && mnemonic) {
      logger.info('TEST: import_account')
      accountJSON = await testImportAccount(fullServiceURL, mnemonic)
        .catch((error) => {
          if (error instanceof ResponseError) {
            if (error.response.error.data.server_error.match('AccountAlreadyExists')) {
              return getFirstAccount(publicMirrorURL, keyFile)
            }
          }
          return Promise.reject(error)
        })
    } else {
      logger.info('TEST: get_accounts - return first account')
      accountJSON = await getFirstAccount(publicMirrorURL, keyFile)
    }
    const accountInfo = JSON.parse(accountJSON)
    const accountID = accountInfo.result.account.id
    const mainAddress = accountInfo.result.account.main_address
    logger.info(`  account_id: ${accountID}`)
    logger.info(`  main_address: ${mainAddress}`)

    logger.info('TEST: waitForAccountSync - wait for new account indexing to complete')
    await waitForAccountSync(publicMirrorURL, keyFile, accountID)

    logger.info('TEST: get_account_status - wait for balances to be synced')
    await waitForBalanceToBeSynced(publicMirrorURL, keyFile, accountID)

    logger.info('TEST: get_address_status')
    await testGetAddressStatus(publicMirrorURL, keyFile, mainAddress)

    logger.info('TEST: verify_address')
    await testVerifyAddress(publicMirrorURL, keyFile, mainAddress)

    logger.info('TEST: wallet_status')
    await testWalletStatus(publicMirrorURL, keyFile)

    logger.info('TEST: get_network_status')
    await testNetworkStatus(publicMirrorURL, keyFile)

    logger.info('TEST: create_payment_request')
    await testCreatePaymentRequest(publicMirrorURL, keyFile, accountID, '4000000000', 1)

    logger.info('TEST: get_transaction_logs_for_account - block 1')
    await testGetTransactionLogsForAccount(publicMirrorURL, keyFile, accountID, 1, 1)

    // don't test this if we don't have access to full-service
    if (fullServiceURL) {
      logger.info('TEST: build_and_submit_transaction - requires external full-service')
      const transactionJSON = await testBuildAndSubmitTransaction(fullServiceURL, accountID, mainAddress, '1')
      const transactionInfo = JSON.parse(transactionJSON)
      const transactionLogId = transactionInfo.result.transaction_log.id
      const transactionBlockIndex = transactionInfo.result.transaction_log.submitted_block_index
      const outputTxoHex = transactionInfo.result.transaction_log.output_txos[0].txo_id_hex
      logger.info(`  transactionLogId: ${transactionLogId}`)
      logger.info(`  transaction block: ${transactionBlockIndex}`)
      logger.info(`  outputTxo: ${outputTxoHex}`)

      logger.info('TEST: get_transaction_log')
      await testGetTransactionLogsForBlock(publicMirrorURL, keyFile, transactionBlockIndex)

      logger.info('TEST: get get_transaction_log - wait for transactions to sync by log id')
      await waitForTransactionToBeSynced(publicMirrorURL, keyFile, transactionLogId)

      logger.info('TEST: get_confirmations')
      const confirmationJSON = await testGetConfirmations(publicMirrorURL, keyFile, transactionLogId)
      const confirmationInfo = JSON.parse(confirmationJSON)
      logger.debug('  confirmationJson:')
      logger.debug(JSON.stringify(confirmationInfo, null, 4))

      const confirmation = confirmationInfo.result.confirmations[0].confirmation
      logger.info(`  confirmation: ${confirmation}`)

      logger.info('TEST: validate_confirmation')
      await testValidateConfirmations(publicMirrorURL, keyFile, accountID, outputTxoHex, confirmation)
    }
  } catch (error) {
    logger.error(`Test failed: ${error}`)
    throw error
  }
}

async function getAccounts (publicMirrorURL, keyFile) {
  const request = {
    method: 'get_accounts',
    params: {},
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function getFirstAccount (publicMirrorURL, keyFile) {
  const accountsJSON = await getAccounts(publicMirrorURL, keyFile)
  // Get first account
  const accounts = JSON.parse(accountsJSON)
  const first = accounts.result.account_ids[0]
  const accountMap = accounts.result.account_map

  // fake an import_account response
  return JSON.stringify({
    method: 'import_account',
    result: {
      account: accountMap[first]
    },
    jsonrpc: '2.0',
    id: 1
  })
}

async function testImportAccount (fullServiceURL, mnemonic) {
  const request = {
    method: 'import_account',
    params: {
      mnemonic,
      key_derivation_version: '2',
      name: 'Bob'
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequestUnencrypted(fullServiceURL, fullServicePath, requestString)
}

async function testBuildAndSubmitTransaction (fullServiceURL, accountID, recipientAddress, valuePmob) {
  const request = {
    method: 'build_and_submit_transaction',
    params: {
      account_id: accountID,
      recipient_public_address: recipientAddress,
      amount: { value: valuePmob, token_id: '0' }
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequestUnencrypted(fullServiceURL, fullServicePath, requestString)
}

async function testGetBlock (publicMirrorURL, keyFile) {
  const request = {
    method: 'get_block',
    params: {
      block_index: '0'
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function waitForAccountSync (publicMirrorURL, keyFile, accountID) {
  let accountJSON = await testGetAccountStatus(publicMirrorURL, keyFile, accountID)
  let account = JSON.parse(accountJSON)
  let accountSynced = account.result.account.next_block_index >= account.result.account.local_block_height
  while (!accountSynced) {
    await timer(waitTimeMS)
    accountJSON = await testGetAccountStatus(publicMirrorURL, keyFile, accountID)
    account = JSON.parse(accountJSON)
    accountSynced = account.result.account.next_block_index >= account.result.local_block_height
  }
  return accountJSON
}

async function testGetAccountStatus (publicMirrorURL, keyFile, accountID) {
  const request = {
    method: 'get_account_status',
    params: {
      account_id: accountID
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function waitForBalanceToBeSynced (publicMirrorURL, keyFile, accountID) {
  let balanceJSON = await testGetAccountStatus(publicMirrorURL, keyFile, accountID)
  let balanceInfo = JSON.parse(balanceJSON)
  let balanceSynced = balanceInfo.result.account.next_block_index >= balanceInfo.result.network_block_height
  while (!balanceSynced) {
    await timer(waitTimeMS)
    balanceJSON = await testGetAccountStatus(publicMirrorURL, keyFile, accountID)
    balanceInfo = JSON.parse(balanceJSON)
    balanceSynced = balanceInfo.result.account.next_block_index >= balanceInfo.result.network_block_height
  }
  return balanceJSON
}

async function testCreatePaymentRequest (publicMirrorURL, keyFile, accountID, amountPMOB, subaddressIndex) {
  const request = {
    method: 'create_payment_request',
    params: {
      account_id: accountID,
      amount: {
        value: amountPMOB,
        token_id: '0'
      },
      subaddress_index: subaddressIndex,
      memo: 'testCreatePaymentRequest'
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testGetAddressStatus (publicMirrorURL, keyFile, address) {
  const request = {
    method: 'get_address_status',
    params: {
      address
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testVerifyAddress (publicMirrorURL, keyFile, address) {
  const request = {
    method: 'verify_address',
    params: {
      address
    },
    jsonrpc: '2.0',
    id: 1
  }

  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testWalletStatus (publicMirrorURL, keyFile) {
  const request = {
    method: 'get_wallet_status',
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testNetworkStatus (publicMirrorURL, keyFile) {
  const request = {
    method: 'get_network_status',
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testGetTransactionLogsForBlock (publicMirrorURL, keyFile, blockIndex) {
  const request = {
    method: 'get_transaction_logs',
    params: {
      min_block_index: blockIndex,
      max_block_index: blockIndex
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, fields, 4)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testGetTransactionLogsForAccount (publicMirrorURL, keyFile, accountID, offset, limit) {
  const request = {
    method: 'get_transaction_logs',
    params: {
      account_id: accountID,
      offset,
      limit
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testGetTransactionLogsById (publicMirrorURL, keyFile, transactionLogID) {
  const request = {
    method: 'get_transaction_log',
    params: {
      transaction_log_id: transactionLogID
    },
    jsonrpc: '2.0',
    id: 1
  }

  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function waitForTransactionToBeSynced (publicMirrorURL, keyFile, transactionLogID) {
  let transactionLogsJSON = await testGetTransactionLogsById(publicMirrorURL, keyFile, transactionLogID)
  let transactionInfo = JSON.parse(transactionLogsJSON)
  let transactionStatus = transactionInfo.result.transaction_log.status
  while (transactionStatus === 'pending') {
    await timer(waitTimeMS)
    transactionLogsJSON = await testGetTransactionLogsById(publicMirrorURL, keyFile, transactionLogID)
    transactionInfo = JSON.parse(transactionLogsJSON)
    transactionStatus = transactionInfo.result.transaction_log.status
  }
  return transactionLogsJSON
}

async function testGetConfirmations (publicMirrorURL, keyFile, transactionLogID) {
  const request = {
    method: 'get_confirmations',
    params: {
      transaction_log_id: transactionLogID
    },
    jsonrpc: '2.0',
    id: 1
  }

  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

async function testValidateConfirmations (publicMirrorURL, keyFile, accountID, txoID, confirmation) {
  const request = {
    method: 'validate_confirmation',
    params: {
      account_id: accountID,
      txo_id: txoID,
      confirmation
    },
    jsonrpc: '2.0',
    id: 1
  }
  const requestString = JSON.stringify(request, null)
  logger.debug(JSON.stringify({ request }, null, 4))
  return await client.sendRequest(publicMirrorURL, keyFile, requestString)
}

logger.info('Starting test script')
// Command line parsing

const description = `Examples:
Use with pre-created walets - no direct accesst to full-service required

  node test_script.js --public-mirror-url http://127.0.0.1:9091 --key-file ./mirror-client.pem

Import providied account mnemonic - requires direct access to full-service

  node test_script.js --public-mirror-url http://127.0.0.1:9091 --key-file ./mirror-client.pem \\
    --full-service-url http://127.0.0.1:9090 \\
    --mnemonic '<24 words>'
`

commander
  .name('mirror test script')
  .description(description)
  .requiredOption('--public-mirror-url <public-mirror>', 'http[s] url to the public side of the mirror.')
  .requiredOption('--key-file <path>', 'path to client public key pem file')
  .option('--full-service-url <full-service>', '(optional) http[s] url for direct connection to full-service. Used to add an account.')
  .option('--mnemonic <24 words>', '(optional) add an account mnemonic, requires direct access to full-service')

commander.parse()
const options = commander.opts()

const publicMirrorURL = options.publicMirrorUrl
const fullServiceURL = options.fullServiceUrl
const keyFile = options.keyFile
const mnemonic = options.mnemonic

runAllTests(publicMirrorURL, fullServiceURL, keyFile, mnemonic).then(result => {
  logger.info('Run all tests succeeded')
}).catch((error) => {
  logger.error('Run all tests had an error: ' + error)
  return Promise.reject(error)
})
