# Full Service API

The Full Service Wallet API provides JSON RPC 2.0 endpoints for interacting with your MobileCoin transactions.

## Overview

### Methods Overview

* [create_account](#create-account)
* [import_account](#import-account)
* [import_account_from_legacy_root_entropy](#deprecated-import-legacy-account)
* [get_all_accounts](#get-all-accounts)
* [get_account](#get-account)
* [update_account_name](#update-account-name)
* [remove_account](#remove-account)
* [export_account_secrets](#export-account-secrets)
* [get_all_txos_for_account](#get-all-txos-for-a-given-account)
* [get_txo](#get-txo-details)
* [get_wallet_status](#get-wallet-status)
* [get_balance_for_account](#get-balance-for-a-given-account)
* [get_balance_for_address](#get-balance-for-a-given-address)
* [assign_address_for_account](#assign-address-for-account)
* [get_all_addresses_for_account](#get-all-assigned-addresses-for-a-given-account)
* [verify_address](#verify-address)
* [build_and_submit_transaction](#build-and-submit-transaction)
* [build_transaction](#build-transaction)
* [submit_transaction](#submit-transaction)
* [get_all_transaction_logs_for_account](#get-all-transaction-logs-for-account)
* [get_transaction_log](#get-transaction-log)
* [get_all_transaction_logs_for_block](#get-all-transaction-logs-for-block)
* [get_all_transaction_logs_ordered_by_block](#get-all-transaction-logs-ordered-by-block)
* [get_confirmations](#get-confirmations)
* [validate_confirmation](#validate-confirmation)
* [check_receiver_receipt_status](#check-receiver-receipt-status)
* [create_receiver_receipts](#create-receiver-receipts)
* [build_gift_code](#build-gift-code)
* [submit_gift_code](#submit-gift-code)
* [get_gift_code](#get-gift-code)
* [get_all_gift_codes](#get-all-gift-codes)
* [check_gift_code_status](#check-gift-code-status)
* [claim_gift_code](#claim-gift-code)
* [remove_gift_code](#remove-gift-code)
* [get_txo_object](#get-txo-object)
* [get_transaction_object](#get-transaction-object)
* [get_block_object](#get-block-object)

### Full Service Data Types Overview

The methods above return data representations of wallet contents. The Full Service API Data types are as follows:

* [account](#the-account-object)
* [account_secrets](#the-account-secrets-object)
* [balance](#the-balance-object)
* [wallet_status](#the-wallet-status-object)
* [address](#the-address-object)
* [transaction_log](#the-transaction-log-object)
* [txo](#the-txo-object)
* [confirmation](#the-confirmation-object)
* [receiver_receipt](#the-receiver-receipt-object)
* [gift_code](#the-gift-code-object)

## Full Service API Methods

### Accounts

#### Create Account

Create a new account in the wallet.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "create_account",
        "params": {
          "name": "Alice"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "create_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Alice",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  }
}
```

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `name`         | Label for this account   | Can have duplicates (not recommended) |

#### Import Account

Import an existing account from the secret entropy.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "import_account",
        "params": {
          "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
          "key_derivation_version": "2",
          "name": "Bob"
          "next_subaddress_index": 2,
          "first_block_index": "3500"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
   -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `mnemonic`      | The secret mnemonic to recover the account  | 24 words  |
| `key_derivation_version`      | The version number of the key derivation used to derive an account key from this mnemonic. Current version is 2 |  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `name`         | Label for this account   | Can have duplicates (not recommended) |
| `first_block_index`  | The block from which to start scanning the ledger |  |
| `next_subaddress_index`  | The next known unused subaddress index for the account |  |

#### Deprecated: Import Legacy Account

Import an existing account from the secret entropy. - Deprecated

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "import_account_from_legacy_root_entropy",
        "params": {
          "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
          "name": "Bob"
          "next_subaddress_index": 2,
          "first_block_index": "3500",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
   -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `entropy`      | The secret root entropy  | 32 bytes of randomness, hex-encoded  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `name`         | Label for this account   | Can have duplicates (not recommended) |
| `first_block_index`  | The block from which to start scanning the ledger |  |
| `next_subaddress_index`  | The next known unused subaddress index for the account |  |

##### Troubleshooting

If you receive the following error, it means that you attempted to import an account already in the wallet.

```sh
{"error": "Database(Diesel(DatabaseError(UniqueViolation, "UNIQUE constraint failed: accounts.account_id_hex")))"}
```

##### Troubleshooting

If you receive the following error, it means that you attempted to import an account already in the wallet.

```sh
{"error": "Database(Diesel(DatabaseError(UniqueViolation, "UNIQUE constraint failed: accounts.account_id_hex")))"}
```

#### Get All Accounts

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_accounts",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_all_accounts",
  "result": {
    "account_ids": [
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0"
    ],
    "account_map": {
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52": {
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "key_derivation_version:": "1",
        "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "name": "Alice",
        "next_subaddress_index": "2",
        "first_block_index": "3500",
        "object": "account",
        "recovery_mode": false
      },
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0": {
        "account_id": "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0",
        "key_derivation_version:": "2",
        "main_address": "7EqduSDpM1R5AfQejbjAqFxpuCoh6zJECtvJB9AZFwjK13dCzZgYbyfLf4TfHcE8LVPjzDdpcxYLkdMBh694mHfftJmsFZuz6xUeRtmsUdc",
        "name": "Alice",
        "next_subaddress_index": "2",
        "first_block_index": "3500",
        "object": "account",
        "recovery_mode": false
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

#### Get Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_account",
        "params": {
          "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'  | jq
```

```json
{
  "method": "get_account",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "key_derivation_version:": "2",
      "name": "Alice",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

#### Troubleshooting

If you receive the following error, it means that this account is not in the database.

```sh
{
  "error": "Database(AccountNotFound(\"a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10\"))",
  "details": "Error interacting with the database: Account Not Found: a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
}
```

#### Update Account Name

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "update_account_name",
        "params": {
          "acount_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
          "name": "Carol"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'  | jq
```

```json
{
  "method": "update_account_name",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "name": "Carol",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |
| `name`         | The new name for this account  |   |

#### Remove Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "remove_account",
        "params": {
          "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "remove_account",
  "result": {
    "removed": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

#### Export Account Secrets

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "export_account_secrets",
        "params": {
          "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "export_account_secrets",
  "result": {
    "account_secrets": {
      "object": "account_secrets",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "entropy": "c0b285cc589447c7d47f3yfdc466e7e946753fd412337bfc1a7008f0184b0479",
      "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
      "key_derivation_version": "2",
      "account_key": {
        "object": "account_key",
        "view_private_key": "0a20be48e147741246f09adb195b110c4ec39302778c4554cd3c9ff877f8392ce605",
        "spend_private_key": "0a201f33b194e13176341b4e696b70be5ba5c4e0021f5a79664ab9a8b128f0d6d40d",
        "fog_report_url": "",
        "fog_report_id": "",
        "fog_authority_spki": ""
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

If the account was generated using version 1 of the key derviation, entropy will be provided as a hex encoded string.

If the account was generated using version 2 of the key derivation, mnemonic will be provided as a 24 word mnemonic string.

### TXOs

#### Get All TXOs for a given account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_txos_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'  | jq
```

```json
{
  "method": "get_all_txos_for_account",
  "result": {
    "txo_ids": [
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167",
      "00408833347550b046f0996afe92313745f76e307904686e93de5bab3590e9da",
      "005b41a40be1401426f9a00965cc334e4703e4089adb8fa00616e7b25b92c6e5",
      ...
    ],
    "txo_map": {
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167": {
        "account_status_map": {
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "spent",
            "txo_type": "received"
          }
        },
        "assigned_subaddress": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "e_fog_hint": "0a54bf0a5f37989b379b9db3e8937387c5033428b399d44ee524c02b53ce8b7fa7ffc7181a854255cefc68704f69eedd43a891d2ed65c9f6e4c0fc645c2bc156278395221100a4fc3a1d617d04f6eca8851e846a0100",
        "is_spent_recovered": false,
        "key_image": "0a20f041e3da520a6e3328d43a920b90bf87826a1602c9249cf6591dd32328a4544e",
        "minted_account_id": null,
        "object": "txo",
        "confirmation": null,
        "public_key": "0a201a592874a596aeb14cbeb1c7d3449cbd20dc8078ad7fff657e131d619145ef0a",
        "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "received_block_index": "128567",
        "spent_block_index": "128569",
        "subaddress_index": "0",
        "target_key": "0a209e1067117870549a77a47de04bd810da052abfc23d60a0c433367bfc689b7428",
        "txo_id": "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167",
        "value_pmob": "990000000000"
      },
      "84f30233774d728bb7844bed59d471fe55ee3680ab70ddc312840db0f978f3ba": {
        "account_status_map": {
          "36fdf8fbdaa35ad8e661209b8a7c7057f29bf16a1e399a34aa92c3873dfb853c": {
            "txo_status": "unspent",
            "txo_type": "received"
          },
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "secreted",
            "txo_type": "minted"
          }
        },
        "assigned_subaddress": null,
        "e_fog_hint": "0a5472b079a520696518cc7d7c3036e855cbbcf1a3e247db32ab2e62e835183077b862ef86ec4963a584650cc028eb645569f9de1392b88f8fd7fa07aa28c4e035fd5f4866f3db3d403a05d2adb5e4f2992c010b0100",
        "is_spent_recovered": false,
        "key_image": null,
        "minted_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "object": "txo",
        "confirmation": "0a204488e153cce1e4bcdd4419eecb778f3d2d2b024b39aaa29532d2e47e238b2e31",
        "public_key": "0a20e6736474f73e440686736bfd045d838c2b3bc056ffc647ad6b1c990f5a46b123",
        "received_account_id": "36fdf8fbdaa35ad8e661209b8a7c7057f29bf16a1e399a34aa92c3873dfb853c",
        "received_block_index": null,
        "spent_block_index": null,
        "subaddress_index": null,
        "target_key": "0a20762d8a723aae2aa70cc11c62c91af715f957a7455b695641fe8c94210812cf1b",
        "txo_id": "84f30233774d728bb7844bed59d471fe55ee3680ab70ddc312840db0f978f3ba",
        "value_pmob": "200"
      },
      "58c2c3780792ccf9c51014c7688a71f03732b633f8c5dfa49040fa7f51328280": {
        "account_status_map": {
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "unspent",
            "txo_type": "received"
          }
        },
        "assigned_subaddress": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "e_fog_hint": "0a546f862ccf5e96a89b3ede770a70aa26ce8be704a7e5a73fff02d16ee1f694297b6c17d2e668d6181df047ae68730dfc7913b28aca66450ee1de0ca3b0bedb07664918899848f217bcbbe48be2ef40074ae5dd0100",
        "is_spent_recovered": false,
        "key_image": "0a20784ab38c4541ce23abbec6744431d6ae14101c49c6535b3e9bf3fd728db13848",
        "minted_account_id": null,
        "object": "txo",
        "confirmation": null,
        "public_key": "0a20d803a979c9ec0531f106363a885dde29101fcd70209f9ed686905512dfd14d5f",
        "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "received_block_index": "79",
        "spent_block_index": null,
        "subaddress_index": "0",
        "target_key": "0a209abadbfcec6c81b3d184dc104e51cac4c4faa8bab4da21a3714901519810c20d",
        "txo_id": "58c2c3780792ccf9c51014c7688a71f03732b633f8c5dfa49040fa7f51328280",
        "value_pmob": "4000000000000"
      },
      "b496f4f3ec3159bf48517aa7d9cda193ef8bfcac343f81eaed0e0a55849e4726": {
        "account_status_map": {
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "secreted",
            "txo_type": "minted"
          }
        },
        "assigned_subaddress": null,
        "e_fog_hint": "0a54338fcf8609cf80dfe017bee2339b22b626af2957ef579ae8829f3d8e7fab6c20365b6a99727fcd5e3de7784fca7e1cbb77ec35e7f2c39ea47ef6121716119ba5a67f8a6026a6a6274e7262ea8ea8280782440100",
        "is_spent_recovered": false,
        "key_image": null,
        "minted_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "object": "txo",
        "confirmation": null,
        "public_key": "0a209432c589bb4e5101c26e935b70930dfe45c78417527fb994872ebd65fcb9c116",
        "received_account_id": null,
        "received_block_index": null,
        "spent_block_index": null,
        "subaddress_index": null,
        "target_key": "0a208c75723e9b9a4af0c833bfe190c43900c3b41834cf37024f5fecfbe9919dff23",
        "txo_id": "b496f4f3ec3159bf48517aa7d9cda193ef8bfcac343f81eaed0e0a55849e4726",
        "value_pmob": "980000000000"
      }
    ]
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `limit`     | Don't return more than this many entries | |
| `offset`     | Start returning results after this many entries | |


Note, you may wish to filter TXOs using a tool like jq. For example, to get all unspent TXOs, you can use:

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_txos_for_account",
        "params": {
          "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
        },
        "jsonrpc": "2.0",
        "id": 1,
      }' \
  -X POST -H 'Content-type: application/json' \
  | jq '.result | .txo_map[] | select( . | .account_status_map[].txo_status | contains("unspent"))'
```

#### Get TXO Details

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_txo",
        "params": {
          "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
```json
{
  "method": "get_txo",
  "result": {
    "txo": {
      "object": "txo",
      "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7",
      "value_pmob": "2960000000000",
      "received_block_index": "8094",
      "spent_block_index": "8180",
      "is_spent_recovered": false,
      "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "minted_account_id": null,
      "account_status_map": {
        "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
          "txo_status": "spent",
          "txo_type": "received"
        }
      },
      "target_key": "0a209eefc082a656a34fae5cec81044d1b13bd8963c411afa28aecfce4839fc9f74e",
      "public_key": "0a20f03f9684e5420d5410fe732f121626352d45e4e799d725432a0c61fa1343ac51",
      "e_fog_hint": "0a544944e7527b7f09322651b7242663edf17478fd1804aeea24838a35ad3c66d5194763642ae1c1e0cd2bbe2571a97a8c0fb49e346d2fd5262113e7333c7f012e61114bd32d335b1a8183be8e1865b0a10199b60100",
      "subaddress_index": "0",
      "assigned_subaddress": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
      "key_image": "0a205445b406012d26baebb51cbcaaaceb0d56387a67353637d07265f4e886f33419",
      "confirmation": null
    }
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |
| `txo_id`   | The txo ID for which to get details  |  |

#### Get Wallet Status

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_wallet_status",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_wallet_status",
  "result": {
    "wallet_status": {
      "account_ids": [
        "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
        "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470"
      ],
      "account_map": {
        "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470": {
          "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
          "key_derivation_version:": "2",
          "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "name": "Bob",
          "next_subaddress_index": "2",
          "first_block_index": "3500",
          "object": "account",
          "recovery_mode": false
        },
        "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17": {
          "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
          "key_derivation_version:": "2",
          "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
          "name": "Carol",
          "next_subaddress_index": "2",
          "first_block_index": "3500",
          "object": "account",
          "recovery_mode": false
        }
      },
      "is_synced_all": false,
      "local_block_index": "152918",
      "network_block_index": "152918",
      "object": "wallet_status",
      "total_orphaned_pmob": "0",
      "total_pending_pmob": "70148220000000000",
      "total_secreted_pmob": "0",
      "total_spent_pmob": "0",
      "total_unspent_pmob": "220588320000000000"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

#### Get Balance for a Given Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_balance_for_account",
        "params": {
           "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_balance_for_account",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152918",
      "local_block_index": "152918",
      "account_block_index": "152003",
      "is_synced": false,
      "unspent_pmob": "110000000000000000",
      "pending_pmob": "0",
      "spent_pmob": "0",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |


#### Get Balance for a Given Address

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_balance_for_address",
        "params": {
           "address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
        },
        "jsonrpc": "2.0",
        "api_version": "2",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_balance_for_address",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152961",
      "local_block_index": "152961",
      "account_block_index": "152961",
      "is_synced": true,
      "unspent_pmob": "11881402222024",
      "pending_pmob": "0",
      "spent_pmob": "84493835554166",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  "api_version": "2"
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `address`      | The address on which to perform this action  | Address must be assigned for an account in the wallet  |


#### Get Account Status for a Given Account

The account status includes both the account object and the balance object.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_account_status",
        "params": {
           "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_account_status",
  "result": {
    "account": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "balance": {
      "account_block_index": "152918",
      "is_synced": true,
      "local_block_index": "152918",
      "network_block_index": "152918",
      "object": "balance",
      "orphaned_pmob": "0",
      "pending_pmob": "2040016523222112112",
      "secreted_pmob": "204273415999956272",
      "spent_pmob": "0",
      "unspent_pmob": "51080511222211091"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}

```

### Addresses

#### Assign Address for Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "assign_address_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "metadata": "For transactions from Carol"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "assign_address_for_account",
  "result": {
    "address": {
      "object": "address",
      "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "metadata": "",
      "subaddress_index": "2"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `metadata`     | Metadata for this address | String; can contain stringified json  |

#### Get All Assigned Addresses for a Given Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_addresses_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_all_addresses_for_account",
  "result": {
    "public_addresses": [
      "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG",
      "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
    ],
    "address_map": {
      "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3": {
        "object": "address",
        "public_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Main",
        "subaddress_index": "0"
      },
      "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG": {
        "object": "address",
        "public_address": "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Change",
        "subaddress_index": "1"
      },
      "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z": {
        "object": "address",
        "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "",
        "subaddress_index": "2"
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

#### Verify Address

Verify whether an address is correctly b58 encoded.


```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "verify_address",
        "params": {
          "address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "verify_address",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

### Transactions

#### Build and Submit Transaction

Sending a transaction is a convenience method that first builds and then submits a transaction.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_and_submit_transaction",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "value_pmob": "42000000000000"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
`
```json
{
  "method": "build_and_submit_transaction",
  "result": {
    "transaction_log": {
      "object": "transaction_log",
      "transaction_log_id": "937f102052500525ff0f54aa4f7d94234bd824260bfd7ba40d0561166dda7780",
      "direction": "tx_direction_sent",
      "is_sent_recovered": null,
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "assigned_address_id": null,
      "value_pmob": "42000000000000",
      "fee_pmob": "10000000000",
      "submitted_block_index": "152948",
      "finalized_block_index": null,
      "status": "tx_status_pending",
      "input_txo_ids": [
        "8432bb4e25f1bde68e4759b27ec72d290252cb99943f2f38a9035dba230895b7"
      ],
      "output_txo_ids": [
        "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb"
      ],
      "change_txo_ids": [
        "44c03ddbccb33e5c37365d7b263568a49e6f608e5e818db604541cc09389b762"
      ],
      "sent_time": "2021-02-28 01:27:52 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id` | The account on which to perform this action  | Account must exist in the wallet  |
| `recipient_public_address` | Recipient for this transaction  | b58-encoded public address bytes  |
| `value_pmob` | The amount of MOB to send in this transaction  |   |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `get_all_txos_for_account`) |
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

it may mean that your account is not yet fully synced. Call `check_balance` for the account, and note the `synced_blocks` value. If that value is less than the `local_block_index` value, then your Txos may not all be updated to their spent status.

#### Build Transaction

You can build a transaction to confirm its contents before submitting it to the network.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_transaction",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "value_pmob": "42000000000000"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```
{
  "method": "build_transaction",
  "result": {
    "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
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
          "subaddress_index": "0",
          "key_image": "2a14381de88c3fe2b827f6adaa771f620873009f55cc7743dca676b188508605",
          "value": "1",
          "attempted_spend_height": "0",
          "attempted_spend_tombstone": "0",
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
          "subaddress_index": "0",
          "key_image": "a66fa1c3c35e2c2a56109a901bffddc1129625e4c4b381389f6be1b5bb3c7056",
          "value": "97580449900010990",
          "attempted_spend_height": "0",
          "attempted_spend_tombstone": "0",
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
      "fee": "10000000000",
      "outlay_index_to_tx_out_index": [
        [
          "0",
          "0"
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
| `value_pmob` | The amount of MOB to send in this transaction  |   |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `get_all_txos_for_account`) |
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
          "value_pmob": "42000000000000"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq -c '.result | .tx_proposal' > test-tx-proposal.json
```

#### Submit Transaction

##### Logging the Submitted Transaction for an Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "submit_transaction",
        "params": {
          "tx_proposal": '$(cat test-tx-proposal.json)',
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'
```

```json
{
  "method": "submit_transaction",
  "result": {
    "transaction_log": {
      "object": "transaction_log",
      "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
      "direction": "tx_direction_sent",
      "is_sent_recovered": null,
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "assigned_address_id": null,
      "value_pmob": "42000000000000",
      "fee_pmob": "10000000000",
      "submitted_block_index": "152950",
      "finalized_block_index": null,
      "status": "tx_status_pending",
      "input_txo_ids": [
        "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d"
      ],
      "output_txo_ids": [
        "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c"
      ],
      "change_txo_ids": [
        "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4"
      ],
      "sent_time": "2021-02-28 01:42:28 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

##### Without Logging the Submitted Transaction

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "submit_transaction",
        "params": {
          "tx_proposal": '$(cat test-tx-proposal.json)'
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'

{
  "method": "submit_transaction",
  "result": {
    "transaction_log": null
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `tx_proposal`  | Transaction proposal to submit  | Created with `build_transaction`  |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id` | Account ID for which to log the transaction. If omitted, the transaction is not logged.   | |
| `comment` | Comment to annotate this transaction in the transaction log   | |

#### Get All Transaction Logs For Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_transaction_logs_for_account",
        "params": {
          "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_all_transaction_logs_for_account",
  "result": {
    "transaction_log_ids": [
      "49da8168e26331fc9bc109d1e59f7ed572b453f232591de4196f9cefb381c3f4",
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1"
    ],
    "transaction_log_map": {
      "49da8168e26331fc9bc109d1e59f7ed572b453f232591de4196f9cefb381c3f4": {
        "object": "transaction_log",
        "transaction_log_id": "49da8168e26331fc9bc109d1e59f7ed572b453f232591de4196f9cefb381c3f4",
        "direction": "tx_direction_received",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": null,
        "assigned_address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "value_pmob": "8199980000000000",
        "fee_pmob": null,
        "submitted_block_index": null,
        "finalized_block_index": "130689",
        "status": "tx_status_succeeded",
        "input_txo_ids": [],
        "output_txo_ids": [
          "49da8168e26331fc9bc109d1e59f7ed572b453f232591de4196f9cefb381c3f4"
        ],
        "change_txo_ids": [],
        "sent_time": null,
        "comment": "",
        "failure_code": null,
        "failure_message": null
      },
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1": {
        "object": "transaction_log",
        "transaction_log_id": "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
        "direction": "tx_direction_sent",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "assigned_address_id": null,
        "value_pmob": "8000000000008",
        "fee_pmob": "10000000000",
        "submitted_block_index": "152951",
        "finalized_block_index": "152951",
        "status": "tx_status_succeeded",
        "input_txo_ids": [
          "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb",
          "c91b5f27e28460ef6c4f33229e70c4cfe6dc4bc1517a22122a86df9fb8e40815"
        ],
        "output_txo_ids": [
          "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
        ],
        "change_txo_ids": [
          "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc"
        ],
        "sent_time": "2021-02-28 03:05:11 UTC",
        "comment": "",
        "failure_code": null,
        "failure_message": null
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |

#### Get Transaction Log

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_transaction_log",
        "params": {
          "transaction_log_id": "914e703b5b7bc44b61bb3657b4ee8a184d00e87a728e2fe6754a77a38598a800"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_transaction_log",
  "result": {
    "transaction_log": {
      "object": "transaction_log",
      "transaction_log_id": "914e703b5b7bc44b61bb3657b4ee8a184d00e87a728e2fe6754a77a38598a800",
      "direction": "tx_direction_received",
      "is_sent_recovered": null,
      "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "recipient_address_id": null,
      "assigned_address_id": null,
      "value_pmob": "51068338999989068",
      "fee_pmob": null,
      "submitted_block_index": null,
      "finalized_block_index": "152905",
      "status": "tx_status_succeeded",
      "input_txo_ids": [],
      "output_txo_ids": [
        "914e703b5b7bc44b61bb3657b4ee8a184d00e87a728e2fe6754a77a38598a800"
      ],
      "change_txo_ids": [],
      "sent_time": null,
      "comment": "",
      "failure_code": null,
      "failure_message": null
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `transaction_log_id`   | The transaction log ID to get.  | Transaction log must exist in the wallet  |

#### Get All Transaction Logs for Block

Get the transaction logs in a given block. In the below example, the account in the wallet sent a transaction to itself. Therefore, there is one sent transaction_log in the block, and two received (one for the change, and one for the output txo sent to the same account that constructed the transaction).

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_transaction_logs_for_block",
        "params": {
          "block_index": "152951"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_all_transaction_logs_for_block",
  "result": {
    "transaction_log_ids": [
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
      "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc",
      "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
    ],
    "transaction_log_map": {
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1": {
        "object": "transaction_log",
        "transaction_log_id": "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
        "direction": "tx_direction_sent",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "assigned_address_id": null,
        "value_pmob": "8000000000008",
        "fee_pmob": "10000000000",
        "submitted_block_index": "152951",
        "finalized_block_index": "152951",
        "status": "tx_status_succeeded",
        "input_txo_ids": [
          "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb",
          "c91b5f27e28460ef6c4f33229e70c4cfe6dc4bc1517a22122a86df9fb8e40815"
        ],
        "output_txo_ids": [
          "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
        ],
        "change_txo_ids": [
          "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc"
        ],
        "sent_time": "2021-02-28 03:05:11 UTC",
        "comment": "",
        "failure_code": null,
        "failure_message": null
      },
      "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc": {
        "object": "transaction_log",
        "transaction_log_id": "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc",
        "direction": "tx_direction_received",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": null,
        "assigned_address_id": "2pW3CcHUmg4cafp9ePCpPg72mowC6NJZ1iHQxpkiAuPJuWDVUC9WEGRxychqFmKXx68VqerFKiHeEATwM5hZcf9SKC9Cub2GyMsztSqYdjY",
        "value_pmob": "11891402222024",
        "fee_pmob": null,
        "submitted_block_index": null,
        "finalized_block_index": "152951",
        "status": "tx_status_succeeded",
        "input_txo_ids": [],
        "output_txo_ids": [
          "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc"
        ],
        "change_txo_ids": [],
        "sent_time": null,
        "comment": "",
        "failure_code": null,
        "failure_message": null
      },
      "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab": {
        "object": "transaction_log",
        "transaction_log_id": "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab",
        "direction": "tx_direction_received",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": null,
        "assigned_address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "value_pmob": "8000000000008",
        "fee_pmob": null,
        "submitted_block_index": null,
        "finalized_block_index": "152951",
        "status": "tx_status_succeeded",
        "input_txo_ids": [],
        "output_txo_ids": [
          "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
        ],
        "change_txo_ids": [],
        "sent_time": null,
        "comment": "",
        "failure_code": null,
        "failure_message": null
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

#### Get All Transaction Logs Ordered By Block

Get the transaction logs, grouped by the `finalized_block_index`, in ascending order.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_transaction_logs_ordered_by_block",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_all_transaction_logs_ordered_by_block",
  "result": {
    "transaction_log_map": {
      "c91b5f27e28460ef6c4f33229e70c4cfe6dc4bc1517a22122a86df9fb8e40815": {
        "object": "transaction_log",
        "transaction_log_id": "c91b5f27e28460ef6c4f33229e70c4cfe6dc4bc1517a22122a86df9fb8e40815",
        "direction": "tx_direction_received",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": null,
        "assigned_address_id": "2pW3CcHUmg4cafp9ePCpPg72mowC6NJZ1iHQxpkiAuPJuWDVUC9WEGRxychqFmKXx68VqerFKiHeEATwM5hZcf9SKC9Cub2GyMsztSqYdjY",
        "value_pmob": "11901402222024",
        "fee_pmob": null,
        "submitted_block_index": null,
        "finalized_block_index": "152923",
        "status": "tx_status_succeeded",
        "input_txo_ids": [],
        "output_txo_ids": [
          "c91b5f27e28460ef6c4f33229e70c4cfe6dc4bc1517a22122a86df9fb8e40815"
        ],
        "change_txo_ids": [],
        "sent_time": null,
        "comment": "",
        "failure_code": null,
        "failure_message": null
      },
      "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb": {
        "object": "transaction_log",
        "transaction_log_id": "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb",
        "direction": "tx_direction_received",
        "is_sent_recovered": null,
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "recipient_address_id": null,
        "assigned_address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "value_pmob": "8000000000008",
        "fee_pmob": null,
        "submitted_block_index": null,
        "finalized_block_index": "152948",
        "status": "tx_status_succeeded",
        "input_txo_ids": [],
        "output_txo_ids": [
          "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb"
        ],
        "change_txo_ids": [],
        "sent_time": null,
        "comment": "",
        "failure_code": null,
        "failure_message": null
      },
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1": {
        "object": "transaction_log",
        "transaction_log_id": "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
        "direction": "tx_direction_sent",
        "is_sent_recovered": null,
        "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
        "recipient_address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "assigned_address_id": null,
        "value_pmob": "8000000000008",
        "fee_pmob": "10000000000",
        "submitted_block_index": "152951",
        "finalized_block_index": "152951",
        "status": "tx_status_succeeded",
        "input_txo_ids": [
          "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb",
          "c91b5f27e28460ef6c4f33229e70c4cfe6dc4bc1517a22122a86df9fb8e40815"
        ],
        "output_txo_ids": [
          "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
        ],
        "change_txo_ids": [
          "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc"
        ],
        "sent_time": "2021-02-28 03:05:11 UTC",
        "comment": "",
        "failure_code": null,
        "failure_message": null
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}

```

### Transaction Output Confirmation Numbers

When constructing a transaction, the wallet produces a "confirmation number" for each Txo minted by the transaction. This confirmation number can be delivered to the recipient to prove that they received the Txo from that particular sender.

#### Get Confirmations

A Txo constructed by this wallet will contain a confirmation number, which can be shared with the recipient to verify the association between the sender and this Txo. When calling `get_confirmations` for a transaction, only the confirmation numbers for the "output_txo_ids" are returned.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_confirmations",
        "params": {
          "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_confirmations",
  "result": {
    "confirmations": [
      {
        "object": "confirmation",
        "txo_id": "9e0de29bfee9a391e520a0b9411a91f094a454ebc70122bdc0e36889ab59d466",
        "txo_index": "458865",
        "confirmation": "0a20faca10509c32845041e49e009ddc4e35b61e7982a11aced50493b4b8aaab7a1f"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `transaction_log_id`   | The transaction log ID for which to get confirmation numbers.  | Transaction log must exist in the wallet  |

#### Validate Confirmation

A sender can provide the confirmation numbers from a transaction to the recipient, who then verifies for a specific txo id (note that txo id is specific to the txo, and is consistent across wallets. Therefore the sender and receiver will have the same txo id for the same Txo which was minted by the sender, and received by the receiver) with the following:

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "validate_confirmation",
        "params": {
          "account_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
          "txo_id": "bbee8b70e80837fc3e10bde47f63de41768ee036263907325ef9a8d45d851f15",
          "confirmation": "0a2005ba1d9d871c7fb0d5ba7df17391a1e14aad1b4aa2319c997538f8e338a670bb"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "validate_confirmation",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |
| `txo_id`   | The ID of the Txo for which to validate the confirmation number  | Txo must be a received Txo  |
| `confirmation`   | The confirmation number to validate  | The confirmation number should be delivered by the sender of the Txo in question |

### Transaction Receipts

Senders can optionally provide `receiver_receipts` to the recipient of a transaction. This has more information than the confirmation number (it contains the confirmation number), and can be used by the receiver to poll for the status of the transaction.

#### Check Receiver Receipt Status

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "check_receiver_receipt_status",
        "params": {
          "address": "3Dg4iFavKJScgCUeqb1VnET5ADmKjZgWz15fN7jfeCCWb72serxKE7fqz7htQvRirN4yeU2xxtcHRAN2zbF6V9n7FomDm69VX3FghvkDfpq",
          "receiver_receipt": {
            "object": "receiver_receipt",
            "public_key": "0a20d2118a065192f11e228e0fce39e90a878b5aa628b7613a4556c193461ebd4f67",
            "confirmation": "0a205e5ca2fa40f837d7aff6d37e9314329d21bad03d5fac2ec1fc844a09368c33e5",
            "tombstone_block": "154512",
            "amount": {
              "object": "amount",
              "commitment": "782c575ed7d893245d10d7dd49dcffc3515a7ed252bcade74e719a17d639092d",
              "masked_value": "12052895925511073331"
            }
          }
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "check_receiver_receipt_status",
  "result": {
    "receipts_transaction_status": "TransactionSuccess",
    "txo": {
      "object": "txo",
      "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7",
      "value_pmob": "2960000000000",
      "received_block_index": "8094",
      "spent_block_index": "8180",
      "is_spent_recovered": false,
      "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "minted_account_id": null,
      "account_status_map": {
        "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
          "txo_status": "spent",
          "txo_type": "received"
        }
      },
      "target_key": "0a209eefc082a656a34fae5cec81044d1b13bd8963c411afa28aecfce4839fc9f74e",
      "public_key": "0a20f03f9684e5420d5410fe732f121626352d45e4e799d725432a0c61fa1343ac51",
      "e_fog_hint": "0a544944e7527b7f09322651b7242663edf17478fd1804aeea24838a35ad3c66d5194763642ae1c1e0cd2bbe2571a97a8c0fb49e346d2fd5262113e7333c7f012e61114bd32d335b1a8183be8e1865b0a10199b60100",
      "subaddress_index": "0",
      "assigned_subaddress": "3Dg4iFavKJScgCUeqb1VnET5ADmKjZgWz15fN7jfeCCWb72serxKE7fqz7htQvRirN4yeU2xxtcHRAN2zbF6V9n7FomDm69VX3FghvkDfpq",
      "key_image": "0a205445b406012d26baebb51cbcaaaceb0d56387a67353637d07265f4e886f33419",
      "confirmation": null
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

#### Create Receiver Receipts

After building a TxProposal, you can get the receipts for that transaction and provide it to the recipient so they can poll for the transaction status.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "create_receiver_receipts",
        "params": {
          "tx_proposal": '$(cat tx_proposal.json)',
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "create_receiver_receipts",
  "result": {
    "receiver_receipts": [
      {
        "object": "receiver_receipt",
        "public_key": "0a20d2118a065192f11e228e0fce39e90a878b5aa628b7613a4556c193461ebd4f67",
        "confirmation": "0a205e5ca2fa40f837d7aff6d37e9314329d21bad03d5fac2ec1fc844a09368c33e5",
        "tombstone_block": "154512",
        "amount": {
          "object": "amount",
          "commitment": "782c575ed7d893245d10d7dd49dcffc3515a7ed252bcade74e719a17d639092d",
          "masked_value": "12052895925511073331"
        }
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

### Gift Codes

Gift codes are onetime accounts that contain a single Txo. They provide a means to send MOB in a way that can be "claimed," for example, by pasting a QR code for a gift code into a group chat, and the first person to consume the gift code claims the MOB.

#### Build Gift Code

Builds a Gift Code in a tx_proposal ready to submit to the ledger.

NOTE: You will need to call [submit_gift_code](#submit-gift-code) with the tx_proposal returned from this method in order to submit the transaction to fund the gift code.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_gift_code",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "value_pmob": "42000000000000",
          "memo": "Happy Birthday!"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "build_gift_code",
  "result": {
    "tx_proposal": "...",
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id` | The account on which to perform this action  | Account must exist in the wallet  |
| `value_pmob` | The amount of MOB to send in this transaction  |   |

| Optional Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `get_all_txos_for_account`) |
| `fee` | The fee amount to submit with this transaction | If not provided, uses `MINIMUM_FEE` = .01 MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 50 |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |
| `memo` | Memo for whoever claims the Gift Code.   | |

#### Submit Gift Code

Convenience method to submit a tx_proposal related to a recently built gift code to the ledger. Will add the gift code to the wallet_db once the tx_proposal has been appended to the ledger.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "submit_gift_code",
        "params": {
          "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
          "tx_proposal": '$(cat test-tx-proposal.json)',
          "from_account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "submit_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "Happy Birthday!",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `gift_code_b58` | The b58-encoded gift code contents  | Must be a valid b58-encoded gift code.  |
| `from_account_id` | The account on which to perform this action  | Account must exist in the wallet  |
| `tx_proposal` | Transaction proposal to submit  | Created with `build_gift_code`  |


#### Get Gift Code

Gift codes are stored in the database. You can get a Gift Code to recall the entropy, value, and memo.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_gift_code",
        "params": {
          "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "Happy Birthday!",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `gift_code_b58` | The b58-encoded gift code contents  | Must be a valid b58-encoded gift code.  |


#### Get All Gift Codes

Get all the Gift Codes currently in the database.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_gift_codes",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_all_gift_codes",
  "result": {
    "gift_codes": [
      {
        "object": "gift_code",
        "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
        "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
        "value_pmob": "80000000000",
        "memo": "Happy New Year!",
        "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
        "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
      },
      {
        "object": "gift_code",
        "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY",
        "entropy": "14aa16d9d4000628c82826d9c43bbc17414f8677e74882bf21e44db75d4c2b87",
        "value_pmob": "20000000000",
        "memo": "Happy Birthday!",
        "account_id": "dba3d3b99fe9ce6bc666490b8176be91ace0f4166853b0327ea39928640ea840",
        "txo_id": "ab917ed9e69fa97bd9422452b1a2f615c2405301b220f7a81eb091f75eba3f54"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

#### Check Gift Code Status

Check the status of a Gift Code - whether it is Pending, Available, or Spent.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "check_gift_code_status",
        "params": {
          "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeAvailable",
    "gift_code_value": 100000000,
    "gift_code_memo": "Happy Birthday!"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}

{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeSubmittedPending",
    "gift_code_value": null
    "gift_code_memo": "",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `gift_code_b58` | The b58-encoded gift code contents  | Must be a valid b58-encoded gift code.  |


| Gift Code Status | Meaning                  |
| :------------- | :----------------------- |
| `GiftCodeAvailable` | The gift code Txo is available to be claimed.  |
| `GiftCodeSubmittedPending` | The gift code Txo has not yet appeared in the ledger.  |
| `GiftCodeClaimed` | The gift code Txo has been spent.  |

#### Claim Gift Code

Claim a gift code to an account in this wallet.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "claim_gift_code",
        "params": {
          "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "claim_gift_code",
  "result": {
    "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `gift_code_b58` | The b58-encoded gift code contents  | Must be a valid b58-encoded gift code.  |
| `account_id` | The account on which to perform this action  | Account must exist in the wallet  |

#### Remove Gift Code

Remove a gift code from the database.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "remove_gift_code",
        "params": {
          "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "remove_gift_code",
  "result": {
    "removed": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `gift_code_b58` | The b58-encoded gift code contents  | Must be a valid b58-encoded gift code that exists in the database |

### Ledger and Transaction Data

To get the JSON representations of the objects which are used in the MobileCoin blockchain, you can use the following calls:

#### Get Transaction Object

Get the JSON representation of the "Tx" object in the transaction log.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_transaction_object",
        "params": {
          "transaction_log_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_transaction_object",
  "result": {
    "transaction": ...
  }
}
```

#### Get Txo Object

Get the JSON representation of the "Txo" object in the ledger.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_txo_object",
        "params": {
          "txo_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_txo_object",
  "result": {
    "txo": ...
  }
}
```

#### Get Block Object

Get the JSON representation of the "Block" object in the ledger.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_block_object",
        "params": {
          "block_index": "3204",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

```json
{
  "method": "get_block_object",
  "result": {
    "block": ...
    "block_contents": ...
  }
}
```

## Full Service Data Types

The Full Service Wallet API provides several objects that correspond to the data types of the wallet

### The Account Object

An account in the wallet.

An Account is associated with one AccountKey, containing a View keypair and a Spend keypair.

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "account" | String representing the object's type. Objects of the same type share the same value
| account_id | string | Unique identifier for the account.
| name | string | Display name for the account.
| main_address | string | B58 Address Code for the account's main address. The main address is determined by the seed subaddress. It is not assigned to a single recipient, and should be considered a free-for-all address.
| next_subaddress_index | string (uint64) | This index represents the next subaddress to be assigned as an address. This is useful information in case the account is imported elsewhere.
| recovery_mode | boolean | A flag that indicates this imported account is attempting to un-orphan found TXOs. It is recommended to move all MOB to another account after recovery if the user is unsure of the assigned addresses.

#### Example Object

```json
{
  "object": "account",
  "account_id": "1916a9b3...",
  "name": "I love MobileCoin",
  "main_address": "4bgkVAH...",
  "next_subaddress_index": "3",
  "first_block_index": "3500",
  "recovery_mode": false
}

```

#### API Methods Returning Account Objects

* [create_account](#create-account)
* [import_account](#import-account)
* [import_account_from_legacy_root_entropy](#import-legacy-account-deprecated)
* [get_all_accounts](#get-all-accounts)
* [get_account](#get-account)
* [update_account_name](#update-account-name)


### The Account Secrets Object

Secret keys for an account.

This is returned separately from other account information, to enable more careful handling of cryptographically sensitive information.


#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "account_secrets" | String representing the object's type. Objects of the same type share the same value
| account_id | string | Unique identifier for the account.
| mnemonic | string | A BIP39 encoded mnemonic phrase used to generate the account key.
| key_derivation_version | string (uint64) | The version number of the key derivation path used to generate the account key from the mnemonic.
| account_key |  account_key | The view and spend keys used to transact on the mobilecoin network. Also may contain keys to connect to the Fog ledger scanning service.

#### Example Object

```json
{
  "object": "account_secrets",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
  "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
  "key_derivation_version": "2",
  "account_key": {
    "object": "account_key",
    "view_private_key": "0a20be48e147741246f09adb195b110c4ec39302778c4554cd3c9ff877f8392ce605",
    "spend_private_key": "0a201f33b194e13176341b4e696b70be5ba5c4e0021f5a79664ab9a8b128f0d6d40d",
    "fog_report_url": "",
    "fog_report_id": "",
    "fog_authority_spki": ""
  }
}
```

### The Balance Object

The balance for an account, as well as some information about syncing status needed to interpret the balance correctly.

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "balance" | String representing the object's type. Objects of the same type share the same value
| network_block_index | string (uint64) | The block height of MobileCoin's distributed ledger. The local_block_index is synced when it reaches the network_block_index.
| local_block_index | string (uint64) | The local block height downloaded from the ledger. The local database will sync up to the network_block_index. The account_block_index can only sync up to local_block_index.
| account_block_index| string (uint64) | The scanned local block height for this account. This value will never be greater than the local_block_index. At fully synced, it will match network_block_index.
| is_synced | boolean | Whether the account is synced with the network_block_index. Balances may not appear correct if the account is still syncing.
| unspent_pmob | string (uint64) | Unspent pico MOB for this account at the current account_block_index. If the account is syncing, this value may change.
| pending_pmob | string (uint64) | Pending, out-going pico MOB. The pending value will clear once the ledger processes the outgoing txos. The pending_pmob will reflect the change.
| spent_pmob | string (uint64) | Spent pico MOB. This is the sum of all the Txos in the wallet which have been spent.
| secreted_pmob | string (uint64) | Secreted (minted) pico MOB. This is the sum of all the Txos which have been created in the wallet for outgoing transactions.
| orphaned_pmob | string (uint64) | Orphaned pico MOB. The orphaned value represents the Txos which were view-key matched, but which can not be spent until their subaddress index is recovered.

#### Example Object

```json
{
  "account_block_index": "152003",
  "is_synced": false,
  "local_block_index": "152918",
  "network_block_index": "152918",
  "object": "balance",
  "orphaned_pmob": "0",
  "pending_pmob": "0",
  "secreted_pmob": "0",
  "spent_pmob": "0",
  "unspent_pmob": "110000000000000000"
}
```

#### API Methods Returning Balance Objects

* [get_balance_for_account](#get-balance-for-a-given-account)

### The Wallet Status Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| network_block_index | string (uint64) | The block height of the MobileCoin ledger. The local_block_index is synced when it reaches the value.
| local_block_index | string (uint64) | The local block height downloaded from the ledger. The local database will sync up to the network_block_index. The account_block_index can only sync up to local_block_index.
| is_synced_all | boolean | Whether ALL accounts are synced with the network_block_index. Balances may not appear correct if any account is still syncing.
| total_unspent_pmob | string (uint64) | Unspent pico mob for ALL accounts at the account_block_index. If the account is syncing, this value may change.
| total_pending_pmob | string (uint64) | Pending outgoing pico mob from ALL accounts. Pending pico mobs will clear once the ledger processes the outgoing txo. The available_pmob will reflect the change.
| total_spent_pmob | string (uint64) | Spent pico MOB. This is the sum of all the Txos in the wallet which have been spent.
| total_secreted_pmob | string (uint64) | Secreted (minted) pico MOB. This is the sum of all the Txos which have been created in the wallet for outgoing transactions.
| total_orphaned_pmob | string (uint64) | Orphaned pico MOB. The orphaned value represents the Txos which were view-key matched, but which can not be spent until their subaddress index is recovered.
| account_ids | list | A list of all account_ids imported into the wallet in order of import.
| account_map | hash map | A normalized hash mapping account_id to account objects.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "wallet_status" | String representing the object's type. Objects of the same type share the same value.

#### Example Object

```json
{
"wallet_status": {
  "account_ids": [
    "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
    "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470"
  ],
  "account_map": {
    "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470": {
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "key_derivation_version:": "2",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "name": "Bob",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "key_derivation_version:": "2",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "is_synced_all": false,
  "local_block_index": "152918",
  "network_block_index": "152918",
  "object": "wallet_status",
  "total_orphaned_pmob": "0",
  "total_pending_pmob": "70148220000000000",
  "total_secreted_pmob": "0",
  "total_spent_pmob": "0",
  "total_unspent_pmob": "220588320000000000"
}
```

#### API Methods Returning Wallet Status Objects

* [get_wallet_status](#get-wallet-status)

### The Address Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "address" | String representing the object's type. Objects of the same type share the same value.
| public_address | string | Shareable B58 encoded string that represents this address.
| account_id | string | Unique identifier for the assigned associated account.
| metadata | string | An arbitrary string attached to the object.
| subaddress_index | string (uint64) | The assigned subaddress index on the associated account.

#### Example Object

```json
{
  "object": "address",
  "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
  "metadata": "",
  "subaddress_index": "2"
}
```

#### API Methods Returning Assigned Address Objects

* [assign_address_for_account](#assign-address-for-account)
* [get_all_addresses_for_account](#get-all-assigned-addresses-for-a-given-account)

### The Transaction Log Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "transaction_log" | String representing the object's type. Objects of the same type share the same value.
| transaction_log_id | int | Unique identifier for the transaction log. This value is not associated to the ledger.
| direction | string | A string that identifies if this transaction log was sent or received. Valid values are "sent" or "received".
| is_sent_recovered | boolean | Flag that indicates if the sent transaction log was recovered from the ledger. This value is null for "received" transaction logs. If true, some information may not be available on the transaction log and its txos without user input. If true, the fee receipient_address_id, fee, and sent_time will be null without user input.
| account_id | string | Unique identifier for the assigned associated account. If the transaction is outgoing, this account is from whence the txo came. If received, this is the receiving account.
| recipient_address_id | string | Unique identifier for the recipient associated account. Only available if direction is "sent".
| assigned_address_id | string | Unique identifier for the assigned associated account. Only available if direction is "received".
| value_pmob | string (uint64) | Value in pico MOB associated to this transaction log.
| fee_pmob | string (uint64) | Fee in pico MOB associated to this transaction log. Only on outgoing transaction logs. Only available if direction is "sent".
| submitted_block_index | string (uint64) | The block index of the highest block on the network at the time the transaction was submitted.
| finalized_block_index | string (uint64) | The scanned block block index in which this transaction occurred.
| status | string | String representing the transaction log status. On "sent", valid statuses are "built", "pending", "succeeded", "failed".  On "received", the status is "succeeded".
| input_txo_ids | list | A list of the IDs of the Txos which were inputs to this transaction.
| output_txo_ids | list | A list of the IDs of the Txos which were outputs of this transaction.
| change_txo_ids | list | A list of the IDs of the Txos which were change in this transaction.
| sent_time | timestamp | Time at which sent transaction log was created. Only available if direction is "sent". This value is null if "received" or if the sent transactions were recovered from the ledger (is_sent_recovered = true).
| comment | string | An arbitrary string attached to the object.
| failure_code | int | Code representing the cause of "failed" status.
| failure_message | string | Human parsable explanation of "failed" status.

#### Example Objects

Received:

```json
{
  "object": "transaction_log",
  "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
  "direction": "tx_direction_sent",
  "is_sent_recovered": null,
  "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
  "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
  "assigned_address_id": null,
  "value_pmob": "42000000000000",
  "fee_pmob": "10000000000",
  "submitted_block_index": "152950",
  "finalized_block_index": null,
  "status": "tx_status_pending",
  "input_txo_ids": [
    "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d"
  ],
  "output_txo_ids": [
    "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c"
  ],
  "change_txo_ids": [
    "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4"
  ],
  "sent_time": "2021-02-28 01:42:28 UTC",
  "comment": "",
  "failure_code": null,
  "failure_message": null
}
```

Sent - Failed:

```json
{
  "object": "transaction_log",
  "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
  "direction": "tx_direction_sent",
  "is_sent_recovered": null,
  "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
  "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
  "assigned_address_id": null,
  "value_pmob": "42000000000000",
  "fee_pmob": "10000000000",
  "submitted_block_index": "152950",
  "finalized_block_index": null,
  "status": "failed",
  "input_txo_ids": [
    "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d"
  ],
  "output_txo_ids": [
    "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c"
  ],
  "change_txo_ids": [
    "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4"
  ],
  "sent_time": "2021-02-28 01:42:28 UTC",
  "comment": "This is an example of a failed sent transaction log of 1.288 MOB and 0.01 MOB fee!",
  "failure_code": 3,
  "failure_message:": "Contains sent key image."
}
```

Sent - Success, Recovered:

```json
{
  "object": "transaction_log",
  "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
  "direction": "tx_direction_sent",
  "is_sent_recovered": true,
  "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
  "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
  "assigned_address_id": null,
  "value_pmob": "42000000000000",
  "fee_pmob": "10000000000",
  "submitted_block_index": "152950",
  "finalized_block_index": null,
  "status": "tx_status_pending",
  "input_txo_ids": [
    "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d"
  ],
  "output_txo_ids": [
    "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c"
  ],
  "change_txo_ids": [
    "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4"
  ],
  "sent_time": "2021-02-28 01:42:28 UTC",
  "comment": "",
  "failure_code": null,
  "failure_message": null
}
```

#### API Methods Returning Transaction Log Objects

* [get_all_transaction_logs_for_account](#get-all-transaction-logs-for-account)
* [get_transaction_log](#get-transaction-log)
* [get_all_transaction_logs_for_block](#get-all-transaction-logs-for-block)
* [get_all_transaction_logs_ordered_by_block](#get-all-transaction-logs-ordered-by-block)
* [build_and_submit_transaction](#build-and-submit-transaction)
* [submit_transaction](#submit-transaction)

### The TXO Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "txo" | String representing the object's type. Objects of the same type share the same value.
| value_pmob | string (uint64) | Available pico MOB for this account at the current account_block_index. If the account is syncing, this value may change.
| received_block_index | string (uint64) | Block index in which the Txo was received by an account.
| spent_block_index | string (uint64) | Block index in which the Txo was spent by an account.
| is_spent_recovered | boolean | Flag that indicates if the spent_block_index was recovered from the ledger. This value is null if the Txo is unspent. If true, some information may not be available on the txo without user input. If true, the confirmation number will be null without user input.
| received_account_id | string | The account_id for the account which has received this Txo. This account has spend authority.
| minted_account_i | string | The account_id for the account which minted this Txo.
| account_status_map | hash map | A normalized hash mapping account_id to account objects. Keys include "type" and "status".
| | key: txo_type | With respect to this account, the Txo may be "minted" or "received".
| | key: txo_status | With respect to this account, the Txo may be "unspent", "pending", "spent", "secreted" or "orphaned". For received Txos received as an assigned address, the lifecycle is "unspent" -> "pending" -> "spent". For outbound, minted Txos, we cannot monitor its received lifecycle status with respect to the minting account, we note its status as "secreted". If a Txo is received at an address unassigned (likely due to a recovered account or using the account on another client), the Txo is considered "orphaned" until its address is calculated -- in this case, there are manual ways to discover the missing assigned address for orphaned Txos or to recover an entire account.
| target_key | string (hex) | A cryptographic key for this Txo.
| public_key | string (hex) | The public key for this Txo, can be used as an identifier to find the txo in the ledger.
| e_fog_hint | string (hex) | The encrypted fog hint for this Txo.
| subaddress_index | string (uint64) | The assigned subaddress index for this Txo with respect to its received account.
| assigned_address | string (uint64) | The address corresponding to the subaddress index which was assigned as an intended sender for this Txo.
| key_image (only on pending/spent) | string (hex) | A fingerprint of the Txo derived from your private spend key materials, required to spend a Txo
| confirmation | string (hex) | A confirmation that the sender of the Txo can provide to validate that they participated in the construction of this Txo.

#### Example Objects

Received and Spent Txo

```json
{
  "object": "txo",
  "txo_id": "14ad2f88...",
  "value_pmob": "8500000000000",
  "received_block_index": "14152",
  "spent_block_index": "20982",
  "is_spent_recovered": false,
  "received_account_id": "1916a9b3...",
  "minted_account_id": null,
  "account_status_map": {
    "1916a9b3...": {
      "txo_status": "spent",
      "txo_type": "received"
    }
  },
  "target_key": "6d6f6f6e...",
  "public_key": "6f20776f...",
  "e_fog_hint": "726c6421...",
  "subaddress_index": "20",
  "assigned_subaddress": "7BeDc5jpZ...",
  "key_image": "6d6f6269...",
  "confirmation": "23fd34a..."
}
```

Txo Spent from One Account to Another in the Same Wallet

```json
{
  "object": "txo",
  "txo_id": "84f3023...",
  "value_pmob": "200",
  "received_block_index": null,
  "spent_block_index": null,
  "is_spent_recovered": false,
  "received_account_id": "36fdf8...",
  "minted_account_id": "a4db032...",
  "account_status_map": {
    "36fdf8...": {
      "txo_status": "unspent",
      "txo_type": "received"
    },
    "a4db03...": {
      "txo_status": "secreted",
      "txo_type": "minted"
    }
  },
  "target_key": "0a2076...",
  "public_key": "0a20e6...",
  "e_fog_hint": "0a5472...",
  "subaddress_index": null,
  "assigned_subaddress": null,
  "key_image": null,
  "confirmation": "0a2044..."
}
```

#### API Methods Returning Txo Objects

* [get_all_txos_for_account](#get-all-txos-for-a-given-account)
* [get_txo](#get-txo-details)

### The Confirmation Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "confirmation" | String representing the object's type. Objects of the same type share the same value.
| txo_id | string | Unique identifier for the Txo.
| txo_index | string | The index of the Txo in the ledger.
| confirmation | string | A string with a confirmation number that can be validated to confirm that another party constructed or had knowledge of the construction of the associated Txo.

#### Example Object

```json
{
  "object": "confirmation",
  "txo_id": "873dfb8c...",
  "txo_index": "1276",
  "confirmation": "984eacd..."
}
```

#### API Methods Returning Confirmation Objects

* [get_confirmations](#get-confirmations)
* [validate_confirmation](#validate-confirmation)

### The Receiver Receipt Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "confirmation" | String representing the object's type. Objects of the same type share the same value.
| public_key | string | Hex-encoded public key for the Txo.
| tombstone_block | string | The block index after which this Txo would be rejected by consensus.
| confirmation | string | Hex-encoded confirmation that can be validated to confirm that another party constructed or had knowledge of the construction of the associated Txo.
| amount | string | The encrypted amount in the Txo referenced by this receipt.

#### Example Object

```json
{
  "object": "receiver_receipt",
  "public_key": "0a20d2118a065192f11e228e0fce39e90a878b5aa628b7613a4556c193461ebd4f67",
  "confirmation": "0a205e5ca2fa40f837d7aff6d37e9314329d21bad03d5fac2ec1fc844a09368c33e5",
  "tombstone_block": "154512",
  "amount": {
    "object": "amount",
    "commitment": "782c575ed7d893245d10d7dd49dcffc3515a7ed252bcade74e719a17d639092d",
    "masked_value": "12052895925511073331"
  }
}
```

#### API Methods Returning Receipt Objects

* [create_receiver_receipts](#create-receiver-receipts)

### The Gift Code Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "gift_code" | String representing the object's type. Objects of the same type share the same value.
| gift_code | string | The base58-encoded gift code string to share.
| entropy | string | The entropy for the account in this gift code.
| value_pmob | string | The amount of MOB contained in the gift code account.
| memo | string | A memo associated with this gift code.

#### Example Object

```json
{
  "object": "gift_code",
  "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
  "entropy": "41e1e794f8a2f7227fa8b5cd936f115b8799da712984c85f499e03bca43cba9c",
  "value_pmob": "60000000000",
  "memo": "Happy New Year!",
  "account_id": "050d8d97aaf31c70d63c6aed828c11d3fb16b56b44910659b6724621047b81f9",
  "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
}
```

#### API Methods Returning Gift Code Objects

* [build_gift_code](#build-gift-code)
* [get_gift_code](#get-gift-code)
* [get_all_gift_codes](#get-all-gift-codes)
* [check_gift_code_status](#check-gift-code-status)
* [claim_gift_code](#claim-gift-code)
* [remove_gift_code](#remove-gift-code)

### Future API Objects

#### The Recipient Address object

(Coming soon!)

##### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| address_id | string | Unique identifier for the address.
| public_address | string | Shareable B58 encoded string that represents this address.
| address_book_entry_id | serialized id | The id for an Address Book Entry object if associated to the address.
| comment | string | An arbitrary string attached to the object.

##### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "address" | String representing the object's type. Objects of the same type share the same value.
| account_id | string | Unique identifier for the assigned associated account. Only for "assigned" addresses

##### Example Object

```json
{
  "object": "recipient_address",
  "address_id": "42Dik1AA...",
  "public_address": "42Dik1AA...",
  "address_book_entry_id": 36,
  "comment": "This is a receipient addresses"
}
```

#### The Address Book Entry

(Coming soon!)

##### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| address_book_entry_id | int | Unique identifier for the address book entry. This value is not associated to the ledger.
| alias | string | An arbitrary string attached to the object. Useful as a user-level identifier.
| comment | string | An arbitrary string attached to the object.
| recipient_address_ids | list | A list of all recipient address_ids associated to this address book entry.
| assigned_address_ids | list | A list of all recipient address_ids associated to this address book entry.
| assigned_address_ids_by_account_map | hash map | A normalized hash mapping account_id to a list of assigned address_ids.

##### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "address_book_entry" | String representing the object's type. Objects of the same type share the same value.

##### Example Object

```json
{
  "object": "address_book_entry",
  "address_book_entry_id": 36,
  "alias": "Ojo de Tigre",
  "comment": "Homeboy from way back",
  "recipient_address_ids": ["42Dik1AA...", "MZ1gUP8E...", "4nZaeNa5..."],
  "assigned_address_ids": [ "HpaL8g88...", "YuG7Aa82...", "cPTw8yhs...", "6R6JwQAW..."],
  "assigned_address_ids_by_account_map": {
    "1916a9b3...": ["HpaL8g88...", "YuG7Aa82...", "cPTw8yhs..."],
    "9b3ea14b...": ["6R6JwQAW..."]
  }
}
```
