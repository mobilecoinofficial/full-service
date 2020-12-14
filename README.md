# wallet-service
A MobileCoin service for wallet implementations.

## Build and Run

1. Get the appropriate published enclave measurement, and save to `$(pwd)/consensus-enclave.css`

    ```sh
    NAMESPACE=test
    SIGNED_ENCLAVE_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
    curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${SIGNED_ENCLAVE_URI}
    ```

1. Build

    ```sh
    SGX_MODE=HW IAS_MODE=PROD CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo build --release -p mc-wallet-service
    ```

1. Run

    ```sh
    ./target/release/wallet-service \
        --wallet-db /tmp/wallet-db/wallet.db \
        --ledger-db /tmp/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
    ```

   | Param         | Purpose                  | Requirements              |
   | :------------ | :----------------------- | :------------------------ |
   | `wallet-db`   | Path to wallet file      | Created if does not exist |
   | `ledger-db`   | Path to ledger directory | Created if does not exist |
   | `peer`        | URI of consensus node. Used to submit <br /> transactions and to check the network <br /> block height. | MC URI format |
   | `tx-src-urrl` | S3 location of archived ledger. Used to <br /> sync transactions to the local ledger. | S3 URI format |


## API

### Accounts

#### Create Account

Create a new account in the wallet.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "create_account",
        "params": {
          "name": "Alice"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "create_account",
  "result": {
    "public_address": "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU",
    "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  }
}
```

   | Optional Param | Purpose                  | Requirements              |
   | :------------- | :----------------------- | :------------------------ |
   | `name`         | Label for this account   | Can have duplicates (not recommended) |
   | `first_block`  | The block from which to start scanning the ledger |  |

#### Import Account

Import an existing account from the secret entropy.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "import_account",
        "params": {
          "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
          "name": "Alice"
        }
      }' \
   -X POST -H 'Content-type: application/json' | jq

{
 "method": "import_account",
 "result": {
   "public_address": "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU",
   "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
 }
}
```
| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `entropy`      | The secret root entropy  | 32 bytes of randomness, hex-encoded  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `name`         | Label for this account   | Can have duplicates (not recommended) |
| `first_block`  | The block from which to start scanning the ledger |  |

#### List Accounts

```sh
curl -s localhost:9090/wallet \
  -d '{"method": "list_accounts"}' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "list_accounts",
  "result": {
    "accounts": [
      "c7155cb1660f6dfe778dd52f6381ad3a25f35bd9f502ec337b17478f51abaade",
      "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
    ]
  }
}
```

#### Get Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq

{
  "method": "get_account",
  "result": {
    "name": "Alice",
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

#### Update Account Name

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "update_account_name",
        "params": {
          "acount_id": "2b2d5cce6e24f4a396402fcf5f036890f9c06660f5d29f8420b8c89ef9074cd6",
          "name": "Eve"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq
{
  "method": "update_account_name",
  "result": {
    "success": true
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |
| `name`         | The new name for this account  |   |

#### Delete Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "delete_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "delete_account",
  "result": {
    "success": true
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

### TXOs

#### List TXOs for a given account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "list_txos",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq

{
  "method": "list_txos",
  "result": {
    "txos": [
      {
        "txo_id": "000d688cfe28ab128a7514148f700dc6872e97c1498753fdef4fdd8b90601cd1",
        "value": "97582349900010990",
        "txo_type": "received",
        "txo_status": "spent"
      },
      {
        "txo_id": "00a92e639f2601e9af3ba796c62087cc1c6b9d1bc7c4921df4b136d134ff4027",
        "value": "1",
        "txo_type": "received",
        "txo_status": "spent"
      },
      {
        "txo_id": "00ae2c1a638296dbfe0514019e4efa03b0c714c45b391f1d2180a2c50a38ffad",
        "value": "1",
        "txo_type": "received",
        "txo_status": "spent"
      },
      {
        "txo_id": "00d4f35588ed694edaf58762be9edf3a3cb6941f2a9de3ee779f7c91c3a064a0",
        "value": "97584329900010990",
        "txo_type": "received",
        "txo_status": "spent"
      },
    ]
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

Note, you may wish to filter TXOs using a tool like jq. For example, to get all unspent TXOs, you can use:

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "list_txos",
        "params": {"account_id": "1916a9b39ed28ab3a6eea69ac364b834ccc35b8e9763e8516d1a1f06aba5fb72"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq '.result | .txos[] | select(.txo_status | contains("unspent"))'
```

#### Get TXO Details

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_txo",
        "params": {
          "account_id": "1916a9b39ed28ab3a6eea69ac364b834ccc35b8e9763e8516d1a1f06aba5fb72",
          "txo_id": "fff8e8b65e606578a9baeaa3f2919453444fdd9787e3a04ad5667dd248d02aee"}}' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_txo",
  "result": {
    "txo": {
      "txo_id": "fff8e8b65e606578a9baeaa3f2919453444fdd9787e3a04ad5667dd248d02aee",
      "value": "659999999999",
      "assigned_subaddress": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "subaddress_index": "0",
      "txo_type": "received",
      "txo_status": "spent",
      "received_block_height": "27009",
      "pending_tombstone_block_height": null,
      "spent_block_height": "27010",
      "proof": null
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |
| `txo_id`   | The txo ID for which to get details  |  |

#### Get Balance for a given account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_balance",
        "params": {
           "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_balance",
  "result": {
    "status": {
      "unspent": "97580439900010991",
      "pending": "0",
      "spent": "18135938351572161289",
      "secreted": "0",
      "orphaned": "0",
      "local_block_height": "116504",
      "synced_blocks": "116504"
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

### Addresses

#### Create Assigned Subaddress

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "create_address",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "comment": "For transactions from Carol"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "create_address",
  "result": {
    "address": {
      "public_address_b58": "84NXhbCHE9hQ6fbioRyZJMhuoz6NJFo43JJqboZa7PtqrQWU5ozBi2Px5shPYAr7PR2ED4EL9BvuT1rqDc289t3rMLUYSyxQZxX6EnskNLz",
      "subaddress_index": "3",
      "address_book_entry_id": null,
      "comment": "For transactions from Carol"
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `comment`      | Annotation for this subaddress |  |

#### List Assigned Subaddresses

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "list_addresses",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "list_addresses",
  "result": {
    "addresses": [
      {
        "public_address_b58": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
        "subaddress_index": "0",
        "address_book_entry_id": null,
        "comment": "Main"
      },
      {
        "public_address_b58": "7xKjiti17VLvkJZT2Wb16QWtQSgmVxVwBjr34btrWRLXNBmavK9LEwovkEhrchdXQGCwjDtFo93qLhaBNoKNSSfRNqA5WhK8XQGmyN6Kntv",
        "subaddress_index": "1",
        "address_book_entry_id": null,
        "comment": "Change"
      },
      {
        "public_address_b58": "6mWmJtmyuXiB8iBVbTpB3DKKeKM6rdfiGF9SxhKnBqREdHtD3APooCxxFRL8Ga8rQKeo1b3XKPj8sj227tPdkiybBNEaGXXinFGk7XXA7Bu",
        "subaddress_index": "2",
        "address_book_entry_id": null,
        "comment": "For transactions from Bob"
      },
      {
        "public_address_b58": "7uvFzQXBPbKj4K8fndfve7s1wxRKKVogyCnpqepTWkpshk4gRu63fh5G8JD5UagxfLZvtfYfXuazBPcQSkNiwXVAjmWQTcpw3gQahx1cUmM",
        "subaddress_index": "3",
        "address_book_entry_id": null,
        "comment": "For transactions from Carol"
      },
    ]
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

### Transactions

#### Send Transaction

Sending a transaction is a convenience method that first builds and then submits a transaction.

```
curl -s localhost:9090/wallet \
  -d '{
        "method": "send_transaction",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "value": "42000000000000"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "submit_transaction",
  "result": {
    "transaction": {
      "transaction_id": "96df759d272cfc134b71e24374a7b5125fe535f1d00fc44c1f12a91c1f951122"
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id` | The account on which to perform this action  | Account must exist in the wallet  |
| `recipient_public_address` | Recipient for this transaction  | b58-encoded public address bytes  |
| `value` | The amount of MOB to send in this transaction  |   |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `list_txos`) |
| `fee` | The fee amount to submit with this transaction | If not provided, uses `MINIMUM_FEE` = .01 MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 50 |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |
| `comment` | Comment to annotate this transaction in the transaction log   | |

##### Troubleshooting

If you get the following error response:

```
{
  "error": "Connection(Operation { error: TransactionValidation(ContainsSpentKeyImage), total_delay: 0ns, tries: 1 })"
}
```

it may mean that your account is not yet fully synced. Call `check_balance` for the account, and note the `synced_blocks` value. If that value is less than the `local_block_height` value, then your Txos may not all be updated to their spent status.

#### Build Transaction

You can build a transaction to confirm its contents before submitting it to the network.

```
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_transaction",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "value": "42000000000000"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "build_transaction",
  "result": {
    "tx_proposal": {
      "input_list": [
        {
          "tx_out": {
            "amount": {
              "commitment": "629abf4112819dadfa27947e04ce37d279f568350506e4060e310a14131d3f69",
              "masked_value": "17560205508454890368"
            },
            "target_key": "eec9700ee08358842e16d43fe3df6e346c163b7f6007de4fcf3bafc954847174",
            "public_key": "3209d365b449b577721430d6e0534f5a188dc4bdcefa02be2eeef45b2925bc1b",
            "e_fog_hint": "ae39a969db8ef10daa4f70fa4859829e294ec704b0eb0a15f43ae91bb62bd9ff58ba622e5820b5cdfe28dde6306a6941d538d14c807f9045504619acaafbb684f2040107eb6868c8c99943d02077fa2d090d0100"
          },
          "subaddress_index": 0,
          "key_image": "2a14381de88c3fe2b827f6adaa771f620873009f55cc7743dca676b188508605",
          "value": "1",
          "attempted_spend_height": 0,
          "attempted_spend_tombstone": 0,
          "monitor_id": ""
        },
        {
          "tx_out": {
            "amount": {
              "commitment": "8ccbeaf28bad17ac6c64940aab010fedfdd44fb43c50c594c8fa6e8574b9b147",
              "masked_value": "8257145351360856463"
            },
            "target_key": "2c73db6b914847d124a93691884d2fb181dfcf4d9182686e53c0464cf1c9a711",
            "public_key": "ce43370def13a97830cf6e2e73020b5190d673bd75e0692cd18c850030cc3f06",
            "e_fog_hint": "6b24ceb038ed5c31bfa8f69c73be59eca46612ba8bfea7f53bc52c97cdf549c419fa5a0b2219b1434848197fdbac7880b3a20d92c59c67ec570c7d60e263b4c7c61164f0517c8f774321435c3ec600593d610100"
          },
          "subaddress_index": 0,
          "key_image": "a66fa1c3c35e2c2a56109a901bffddc1129625e4c4b381389f6be1b5bb3c7056",
          "value": "97580449900010990",
          "attempted_spend_height": 0,
          "attempted_spend_tombstone": 0,
          "monitor_id": ""
        }
      ],
      "outlay_list": [
        {
          "value": "42000000000000",
          "receiver": {
            "view_public_key": "5c04cc0de88725f811625b56844aacd789815d43d6df30354939aafd6e683d1a",
            "spend_public_key": "aaf2937c73ef657a529d0f10aaaba394f41bf6f67d8da5ae13284afdb5bc657b",
            "fog_report_url": "",
            "fog_authority_fingerprint_sig": "",
            "fog_report_id": ""
          }
        }
      ],
      "tx": {
        "prefix": {
          "inputs": [
            {
              "ring": [
                {
                  "amount": {
                    "commitment": "3c90eb914a5fe5eb11fab745c9bebfd988de71fa777521099bd442d0eecb765a",
                    "masked_value": "5446626203987095523"
                  },
                  "target_key": "f23c5dd112e5f453cf896294be705f52ee90e3cd15da5ea29a0ca0be410a592b",
                  "public_key": "084c6c6861146672eb2929a0dfc9b9087a49b6531964ca1892602a4e4d2b6d59",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                },
                ...
              ],
              "proofs": [
                {
                  "index": "24296",
                  "highest_index": "335531",
                  "elements": [
                    {
                      "range": {
                        "from": "24296",
                        "to": "24296"
                      },
                      "hash": "f7217a219665b1dfa3f216191de1c79e7d62f520e83afe256b6b43c64ead7d3f"
                    },
                  }
                  ...
                  ]
                },
                ...
              ]
            },
            {
              "ring": [
                {
                  "amount": {
                    "commitment": "50b46eef8d223824f87316e6f446d50530929c8a758195005fbe9d41ec7fc227",
                    "masked_value": "11687342289991185016"
                  },
                  "target_key": "241d533daf32ed1523561c96c618808a2db9635075776ef42da32b34e7586058",
                  "public_key": "24725d8e47e4b03f6cb893369cc7582ea565dbd5e1914a5ecb3f4ed7910c5a03",
                  "e_fog_hint": "3fba73a6271141aae115148196ad59412b4d703847e0738c460c4d1831c6d44004c4deee4fabf6407c5f801703a31a13f1c70ed18a43a0d0a071b863a529dfbab51634fdf127ba2e7a7d426731ba59dbe3660100"
                },
                ...
              ],
              "proofs": [
                {
                  "index": "173379",
                  "highest_index": "335531",
                  "elements": [
                    {
                      "range": {
                        "from": "173379",
                        "to": "173379"
                      },
                      "hash": "bcb26ff5d1104b8c0d7c9aed9b326c824151461257737e0fc4533d1a39e3a876"
                    },
                    ...
                  ]
                },
                ...
              ]
            }
          ],
          "outputs": [
            {
              "amount": {
                "commitment": "147113bbd5d4fdc5f9266ccdec6d6e6148e8dbc979d7d3bab1a91e99ab256518",
                "masked_value": "3431426060591787774"
              },
              "target_key": "2c6a9c23810e91d8c504dd4fe59f07c2872a8a866c160a58928750eab7328c64",
              "public_key": "0049281368c270eb5a7291fb012e95e776a07c1ff4336be1aa6a61abb1868229",
              "e_fog_hint": "eb5b104677df5bbc22f70027646a448dcffb61eb31580d50f41cb487a87a9545d507d4c5e13a22f7fe3b2daea3f951b8d9901e73794d24650176faca3251dd904d7cac97ee73f50a84701cb4c297b31cbdf80100"
            },
            {
              "amount": {
                "commitment": "78083af2c1682f765c332c1c69af4260a410914962bddb9a30857a36aed75837",
                "masked_value": "17824177895224156943"
              },
              "target_key": "68a193eeb7614e3dec6e980dfab2b14aa9b2c3dcaaf1c52b077fbbf259081d36",
              "public_key": "6cdfd36e11042adf904d89bcf9b2eba950ad25f48ed6e877589c40caa1a0d50d",
              "e_fog_hint": "c0c9fe3a43e237ad2f4ab055532831b95f82141c69c75bc6e913d0f37633cb224ce162e59240ffab51054b13e451bfeccb5a09fa5bfbd477c5a8e809297a38a0cb5233cc5d875067cbd832947ae48555fbc00100"
            }
          ],
          "fee": "10000000000",
          "tombstone_block": "0"
        },
        "signature": {
          "ring_signatures": [
            {
              "c_zero": "27a97dbbcf36257b31a1d64a6d133a5c246748c29e839c0f1661702a07a4960f",
              "responses": [
                "bc703776fd8b6b1daadf7e4df7ca4cb5df2d6498a55e8ff15a4bceb0e808ca06",
                ...
              ],
              "key_image": "a66fa1c3c35e2c2a56109a901bffddc1129625e4c4b381389f6be1b5bb3c7056"
            },
            {
              "c_zero": "421cc5527eae6519a8f20871996db99ffd91522ae7ed34e401249e262dfb2702",
              "responses": [
                "322852fd40d5bbd0113a6e56d8d6692200bcedbc4a7f32d9911fae2e5170c50e",
                ...
              ],
              "key_image": "2a14381de88c3fe2b827f6adaa771f620873009f55cc7743dca676b188508605"
            }
          ],
          "pseudo_output_commitments": [
            "1a79f311e74027bdc11fb479ce3a5c8feed6794da40e6ccbe45d3931cb4a3239",
            "5c3406600fbf8e93dbf5b7268dfc43273f93396b2d4976b73cb935d5619aed7a"
          ],
          "range_proofs": [
            ...
          ]
        }
      },
      "fee": 10000000000,
      "outlay_index_to_tx_out_index": [
        [
          0,
          0
        ]
      ],
      "outlay_confirmation_numbers": [
        [...]
      ]
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id` | The account on which to perform this action  | Account must exist in the wallet  |
| `recipient_public_address` | Recipient for this transaction  | b58-encoded public address bytes  |
| `value` | The amount of MOB to send in this transaction  |   |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `list_txos`) |
| `fee` | The fee amount to submit with this transaction | If not provided, uses `MINIMUM_FEE` = .01 MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 50 |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |

Note, as the tx_proposal json object is quite large, you may wish to write the result to a file for use in the submit_transaction call, such as:

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_transaction",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "value": "42000000000000"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq -c '.result | .tx_proposal' > test-tx-proposal.json
```

#### Submit Transaction

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "submit_transaction",
        "params": {
          "tx_proposal": '$(cat test-tx-proposal.json)'
        }
      }' \
  -X POST -H 'Content-type: application/json'

{
  "method": "submit_transaction",
  "result": {
    "transaction": {
      "transaction_id": "96df759d272cfc134b71e24374a7b5125fe535f1d00fc44c1f12a91c1f951122"
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `tx_proposal`  | Transaction proposal to submit  | Created with `build_transaction`  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `comment` | Comment to annotate this transaction in the transaction log   | |

#### List Transactions

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "list_transactions",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "list_transactions",
  "result": {
    "transactions": [
      {
        "transaction_id": "96df759d272cfc134b71e24374a7b5125fe535f1d00fc44c1f12a91c1f951122",
        "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
        "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
        "assigned_subaddress": "",
        "value": "1200000000000",
        "fee": "10000000000",
        "status": "pending",
        "sent_time": "",
        "block_height": "114192",
        "comment": "",
        "direction": "sent",
        "inputs": [
          "972d1369fbef99653e336de89d55a365db76743f413e8fb07b075f1e72dcb61f"
        ],
        "outputs": [
          "a73d8ace011a745d2e6a3c39c55ccd0cc176462e1af62061b1ce77530e75318a"
        ],
        "change": [
          "7e35f469b60bc41aaeac90b218f02a4b1a5453eefa405a4ae356c9edc1492715"
        ]
      },
      {
        "transaction_id": "bdc8a8c2c0b259c8b3d3027d4616e681aac937071bb45ff97673b26aa37acd05",
        "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
        "recipient_public_address": "",
        "assigned_subaddress": "PRwfPLTEJQxE5igytABzL1wJ5zujZNsYzfzEewoUsA91AUfpuPyk3QjgrXcwnhv4HVRZvT5MTE8cWyUE8LwkzSwnENuXyFF26kfDUSritG",
        "value": "111000000000",
        "fee": null,
        "status": "succeeded",
        "sent_time": "",
        "block_height": "116867",
        "comment": "",
        "direction": "received",
        "input_txo_ids": [],
        "output_txo_ids": [
          "6ab350a4122eb029fe75038cebb16c24576c033088be73c8f113aab539704c91"
        ],
        "change_txo_ids": []
      }
      ...
    ]
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

#### Get Transaction

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_transaction",
        "params": {"transaction_id": "2b5c7581264583feaac32296b8ade5562f2c1891d3c92ee86e6df0a8746ec9c9"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_transaction",
  "result": {
    "transaction": {
      "transaction_id": "2b5c7581264583feaac32296b8ade5562f2c1891d3c92ee86e6df0a8746ec9c9",
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "recipient_public_address": "84NXhbCHE9hQ6fbioRyZJMhuoz6NJFo43JJqboZa7PtqrQWU5ozBi2Px5shPYAr7PR2ED4EL9BvuT1rqDc289t3rMLUYSyxQZxX6EnskNLz",
      "assigned_subaddress": "",
      "value": "12300000000000",
      "fee": "10000000000",
      "status": "pending",
      "sent_time": "",
      "block_height": "116316",
      "comment": "",
      "direction": "sent",
      "input_txo_ids": [
        "352165cb67adc9f840cc2d561bbded01f8aed011c854ff447dc2924fa457d8ca",
        "72eb5eb66a12dc45e9e01dd60a0555203ddfdc383b18dbd38c82983ca6662408"
      ],
      "output_txo_ids": [
        "871ca6e7e4db3f691349bbacce02774becaf349b7ad87d487d08f48c14c5e1e9"
      ],
      "change_txo_ids": [
        "ed2a2abdf8d126d93782e8f4b1cdb9ee67eead96563ffbab0fe08a6c44d47c9f"
      ]
    }
  }
}
```

## Contributing

### Database Schema

To add or edit tables:

1. `cd full-service`
1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run --database-url /tmp/db.db`, and test delete with `diesel migration redo --database-url /tmp/db.db`

Note that full-service/diesel.toml provides the path to the schema.rs which will be updated in a migration.

### Running Tests

    ```
    SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo test
    ```

    Note: providinig the CONSENESUS_ENCLAVE_CSS allows us to bypass the enclave build.