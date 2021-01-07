# Full Service

A MobileCoin service for wallet implementations.

* You must read and accept the [Terms of Use for MobileCoins and MobileCoin Wallets](./TERMS-OF-USE.md) to use MobileCoin Software.
* Please note that currently, the MobileCoin Wallet is not available for download or use by U.S. persons or entities, persons or entities located in the U.S., or persons or entities in other prohibited jurisdictions.

#### Note to Developers
* MobileCoin Full Service is a prototype. Expect substantial changes before the release.
* Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for notes on contributing bug reports and code.

##### License

MobileCoin Full Service is available under open-source licenses. Look for the [LICENSE](./LICENSE) file for more information.

## Build and Run

1. Get the appropriate published enclave measurement, and save to `$(pwd)/consensus-enclave.css`

    ```sh
    NAMESPACE=test
    SIGNED_ENCLAVE_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
    curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${SIGNED_ENCLAVE_URI}
    ```

1. Build

    ```sh
    SGX_MODE=HW IAS_MODE=PROD CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo build --release -p mc-full-service
    ```

1. Run

    ```sh
    mkdir /tmp/wallet-db/

    ./target/release/full-service \
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

The Full Service API methods are outlined below. For a description of the API objects, see [API.md](./API.md).

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
    "entropy": "c08187899b0ea7272e1371b97c0fdc2aa4cb3983e087ccce4b5fa44fde52b758",
    "account": {
      "object": "account",
      "account_id": "81ca0a6c473ad70199c19033fd6eb3c94b7acfa2ae5f4065c89a4476a9b2345e",
      "name": "Alice",
      "network_height": "152826",
      "local_height": "152826",
      "account_height": "0",
      "is_synced": false,
      "available_pmob": "0",
      "pending_pmob": "0",
      "main_address": "2XyzT9mtAyfvnET7QuvBAknEYxZCZ5xgBXrhJpTSFYAU7EYgM2MrMmQtguHKQXX1kKtY328swkdJHi85ak9xKrtkPwHX3mMX616XkhDPiwV",
      "next_subaddress_index": "2",
      "recovery_mode": false
    }
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
          "name": "Bob"
        }
      }' \
   -X POST -H 'Content-type: application/json' | jq

{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "name": "Bob",
      "network_height": "152826",
      "local_height": "152826",
      "account_height": "1",
      "is_synced": false,
      "available_pmob": "0",
      "pending_pmob": "0",
      "main_address": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
      "next_subaddress_index": "2",
      "recovery_mode": false
    }
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

#### Get All Accounts

```sh
curl -s localhost:9090/wallet \
  -d '{"method": "get_all_accounts"}' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_all_accounts",
  "result": {
    "account_ids": [
      "81ca0a6c473ad70199c19033fd6eb3c94b7acfa2ae5f4065c89a4476a9b2345e",
      "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
    ],
    "account_map": {
      "81ca0a6c473ad70199c19033fd6eb3c94b7acfa2ae5f4065c89a4476a9b2345e": {
        "account_height": "48630",
        "account_id": "81ca0a6c473ad70199c19033fd6eb3c94b7acfa2ae5f4065c89a4476a9b2345e",
        "available_pmob": "0",
        "is_synced": false,
        "local_height": "152826",
        "main_address": "2XyzT9mtAyfvnET7QuvBAknEYxZCZ5xgBXrhJpTSFYAU7EYgM2MrMmQtguHKQXX1kKtY328swkdJHi85ak9xKrtkPwHX3mMX616XkhDPiwV",
        "name": "Alice",
        "network_height": "152826",
        "next_subaddress_index": "2",
        "object": "account",
        "pending_pmob": "0",
        "recovery_mode": false
      },
      "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10":
        "account_height": "27601",
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "available_pmob": "994799199999988869",
        "is_synced": false,
        "local_height": "152826",
        "main_address": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "name": "Bob",
        "network_height": "152826",
        "next_subaddress_index": "2",
        "object": "account",
        "pending_pmob": "0",
        "recovery_mode": false
      }
    }
  }
}
```

#### Get Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_account",
        "params": {
          "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq

{
  "method": "get_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "name": "Bob",
      "network_height": "152826",
      "local_height": "152826",
      "account_height": "44271",
      "is_synced": false,
      "available_pmob": "994100109999988869",
      "pending_pmob": "0",
      "main_address": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
      "next_subaddress_index": "2",
      "recovery_mode": false
    }
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
          "name": "Carol"
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

#### Get All TXOs for a given account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_txos_by_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq

{
  "method": "get_all_txos_by_account",
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
        "offset_count": 262,
        "proof": null,
        "public_key": "0a201a592874a596aeb14cbeb1c7d3449cbd20dc8078ad7fff657e131d619145ef0a",
        "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "received_block_height": "128567",
        "spent_block_height": "128569",
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
        "offset_count": 501
        "proof": "0a204488e153cce1e4bcdd4419eecb778f3d2d2b024b39aaa29532d2e47e238b2e31",
        "public_key": "0a20e6736474f73e440686736bfd045d838c2b3bc056ffc647ad6b1c990f5a46b123",
        "received_account_id": "36fdf8fbdaa35ad8e661209b8a7c7057f29bf16a1e399a34aa92c3873dfb853c",
        "received_block_height": null,
        "spent_block_height": null,
        "subaddress_index": null,
        "target_key": "0a20762d8a723aae2aa70cc11c62c91af715f957a7455b695641fe8c94210812cf1b",
        "txo_id": "84f30233774d728bb7844bed59d471fe55ee3680ab70ddc312840db0f978f3ba",
        "value_pmob": "200",
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
        "offset_count": 8
        "proof": null,
        "public_key": "0a20d803a979c9ec0531f106363a885dde29101fcd70209f9ed686905512dfd14d5f",
        "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "received_block_height": "79",
        "spent_block_height": null,
        "subaddress_index": "0",
        "target_key": "0a209abadbfcec6c81b3d184dc104e51cac4c4faa8bab4da21a3714901519810c20d",
        "txo_id": "58c2c3780792ccf9c51014c7688a71f03732b633f8c5dfa49040fa7f51328280",
        "value_pmob": "4000000000000",
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
        "offset_count": 498
        "proof": null,
        "public_key": "0a209432c589bb4e5101c26e935b70930dfe45c78417527fb994872ebd65fcb9c116",
        "received_account_id": null,
        "received_block_height": null,
        "spent_block_height": null,
        "subaddress_index": null,
        "target_key": "0a208c75723e9b9a4af0c833bfe190c43900c3b41834cf37024f5fecfbe9919dff23",
        "txo_id": "b496f4f3ec3159bf48517aa7d9cda193ef8bfcac343f81eaed0e0a55849e4726",
        "value_pmob": "980000000000",
      }
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
        "method": "get_all_txos_by_account",
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
          "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7"}}' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_txo",
  "result": {
    "txo": {
      "object": "txo",
      "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7",
      "value_pmob": "2960000000000",
      "received_block_height": "8094",
      "spent_block_height": "8180",
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
      "proof": null,
      "offset_count": 25
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
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_wallet_status",
  "result": {
    "status": {
      "object": "wallet_status",
      "network_height": "152826",
      "local_height": "152826",
      "is_synced_all": false,
      "total_available_pmob": "999699770000000000",
      "total_pending_pmob": "0",
      "account_ids": [
        "15893926fd0eaf0055f73fe1246d369db6a55943e77ebf24c955768792050185",
        "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
      ],
      "account_map": {
        "15893926fd0eaf0055f73fe1246d369db6a55943e77ebf24c955768792050185": {
          "account_height": "60310",
          "account_id": "15893926fd0eaf0055f73fe1246d369db6a55943e77ebf24c955768792050185",
          "available_pmob": "0",
          "is_synced": false,
          "local_height": "152826",
          "main_address": "3fGctHzq5t23xSE3Vj9Ya6uyE2bHAdrn58KaFVgzb6CUHFwPrV9obmnq3XcewvrmEtyeMTMhGvFNqRyVT5FUsu4SAkQW8D7LHs22TVTBQ6m",
          "name": "Alice",
          "network_height": "152826",
          "next_subaddress_index": "2",
          "object": "account",
          "pending_pmob": "0",
          "recovery_mode": false
        },
        "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
          "account_height": "3806",
          "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
          "available_pmob": "999699770000000000",
          "is_synced": false,
          "local_height": "152826",
          "main_address": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
          "name": "Bob",
          "network_height": "152826",
          "next_subaddress_index": "2",
          "object": "account",
          "pending_pmob": "0",
          "recovery_mode": false
        }
      }
    }
  }
}
```

#### Get Balance for a Given Account

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
      "object": "address",
      "address_id": "3",
      "public_address": "3zjsgFjqCjptUD7FYY7bj4qanJWnZjdbVodBkGcBBwx7W4P9GissUvCLx4F4QhVde33Bt75fshEG5A5KRsVCNxhHkHbeS22SXiPDHssmWvL",
      "account_id": "15893926fd0eaf0055f73fe1246d369db6a55943e77ebf24c955768792050185",
      "address_book_entry_id": null,
      "comment": "For transactions from Frank",
      "subaddress_index": "2",
      "offset_count": 0
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

#### Get All Assigned Subaddresses for a Given Account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_addresses_by_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_all_addresses_by_account",
  "result": {
    "address_ids": [
      "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "2pW3CcHUmg4cafp9ePCpPg72mowC6NJZ1iHQxpkiAuPJuWDVUC9WEGRxychqFmKXx68VqerFKiHeEATwM5hZcf9SKC9Cub2GyMsztSqYdjY",
      "8tV9dCdbvmB6mNZyWvz75xdYte38D5qzx2aWv5z85yM7d74NdwbmB7RiFtxHMVknVPfBwYhaPu6M8GuvypPuXk627nW6WzWHMAy2dQJjHGV"
    ],
    "address_map": {
      "2pW3CcHUmg4cafp9ePCpPg72mowC6NJZ1iHQxpkiAuPJuWDVUC9WEGRxychqFmKXx68VqerFKiHeEATwM5hZcf9SKC9Cub2GyMsztSqYdjY": {
        "account_id": "4e09258a93c1b0cb4acafe42bdfe7868bc428755afeccdc841f15eb7016a74f6",
        "address_book_entry_id": null,
        "address_id": "2pW3CcHUmg4cafp9ePCpPg72mowC6NJZ1iHQxpkiAuPJuWDVUC9WEGRxychqFmKXx68VqerFKiHeEATwM5hZcf9SKC9Cub2GyMsztSqYdjY",
        "comment": "Change",
        "object": "assigned_address",
        "offset_count": 10,
        "public_address": "2pW3CcHUmg4cafp9ePCpPg72mowC6NJZ1iHQxpkiAuPJuWDVUC9WEGRxychqFmKXx68VqerFKiHeEATwM5hZcf9SKC9Cub2GyMsztSqYdjY",
        "subaddress_index": "1"
      },
      "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav": {
        "account_id": "4e09258a93c1b0cb4acafe42bdfe7868bc428755afeccdc841f15eb7016a74f6",
        "address_book_entry_id": null,
        "address_id": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "comment": "Main",
        "object": "assigned_address",
        "offset_count": 9,
        "public_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
        "subaddress_index": "0"
      },
      "8tV9dCdbvmB6mNZyWvz75xdYte38D5qzx2aWv5z85yM7d74NdwbmB7RiFtxHMVknVPfBwYhaPu6M8GuvypPuXk627nW6WzWHMAy2dQJjHGV": {
        "account_id": "4e09258a93c1b0cb4acafe42bdfe7868bc428755afeccdc841f15eb7016a74f6",
        "address_book_entry_id": null,
        "address_id": "8tV9dCdbvmB6mNZyWvz75xdYte38D5qzx2aWv5z85yM7d74NdwbmB7RiFtxHMVknVPfBwYhaPu6M8GuvypPuXk627nW6WzWHMAy2dQJjHGV",
        "comment": "For transactions from Frank",
        "object": "assigned_address",
        "offset_count": 11,
        "public_address": "8tV9dCdbvmB6mNZyWvz75xdYte38D5qzx2aWv5z85yM7d74NdwbmB7RiFtxHMVknVPfBwYhaPu6M8GuvypPuXk627nW6WzWHMAy2dQJjHGV",
        "subaddress_index": "2"
      }
    }
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
      "transaction_log_id": "96df759d272cfc134b71e24374a7b5125fe535f1d00fc44c1f12a91c1f951122"
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
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `get_all_txos_by_account`) |
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
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction   | TXO IDs (obtain from `get_all_txos_by_account`) |
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
      "transaction_log_id": "96df759d272cfc134b71e24374a7b5125fe535f1d00fc44c1f12a91c1f951122"
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

#### Get All Transactions

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_transactions_by_account",
        "params": {
          "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_all_transactions_by_account",
  "result": {
    "transaction_log_ids": [
      "6e51851495c628a3b6eefb3e14ee14bb7a167bba5ce727c8710601ba87f74c4c",
      "fcd2979f737f213fc327cd79d10c490a9bd4cb163084d4a154585c5e93e8c075",
    ],
    "transaction_log_map": {
      "6e51851495c628a3b6eefb3e14ee14bb7a167bba5ce727c8710601ba87f74c4c": {
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "assigned_address_id": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "change_txo_ids": [],
        "comment": "",
        "direction": "received",
        "failure_code": null,
        "failure_message": null,
        "fee_pmob": null,
        "finalized_block_height": "144965",
        "input_txo_ids": [],
        "is_sent_recovered": null,
        "object": "transaction_log",
        "offset_count": 296
        "output_txo_ids": [
          "6e51851495c628a3b6eefb3e14ee14bb7a167bba5ce727c8710601ba87f74c4c"
        ],
        "recipient_address_id": null,
        "sent_time": null,
        "status": "succeeded",
        "submitted_block_height": null,
        "transaction_log_id": "6e51851495c628a3b6eefb3e14ee14bb7a167bba5ce727c8710601ba87f74c4c",
        "value_pmob": "443990000000000",
      },
      "6e51851495c628a3b6eefb3e14ee14bb7a167bba5ce727c8710601ba87f74c4c": {
        "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "assigned_address_id": null,
        "change_txo_ids": [
          "e992e718e1f28b67b0cf200e213af560fc7d5a236b3fec590f225b230f88257f"
        ],
        "comment": "",
        "direction": "sent",
        "fee_pmob": "10000000000",
        "failure_code": null,
        "failure_message": null,
        "finalized_block_height": "152826",
        "input_txo_ids": [
          "3de563a16d2da9656ce6c8aa9b12380b682c2e6aad0011fa8d6528c084078827",
          "fa242e21e2155e8f257cd75d2d2939000d0926946c2b7b812946e093165acadb"
        ],
        "is_sent_recovered": null,
        "object": "transaction_log",
        "offset_count": 496
        "output_txo_ids": [
          "badf415972dfc2dc6203ed90be132831ff29f394f65b0be5c35c79048d86af5b"
        ],
        "recipient_address_id": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "sent_time": "2020-12-15 09:30:04 UTC",
        "status": "succeeded",
        "submitted_block_height": "152826",
        "transaction_log_id": "ead39f2c0dea3004732adf1953dee876b73829768d4877809fe06ee0bfc6bf6d",
        "value_pmob": "1000000000000",
      }
     ...
    }
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
        "params": {"transaction_log_id": "ead39f2c0dea3004732adf1953dee876b73829768d4877809fe06ee0bfc6bf6d"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_transaction",
  "result": {
    "transaction": {
      "object": "transaction_log",
      "transaction_log_id": "ead39f2c0dea3004732adf1953dee876b73829768d4877809fe06ee0bfc6bf6d",
      "direction": "sent",
      "is_sent_recovered": null,
      "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "recipient_address_id": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
      "assigned_address_id": null,
      "value_pmob": "1000000000000",
      "fee_pmob": "10000000000",
      "submitted_block_height": "152826",
      "finalized_block_height": "152826",
      "status": "succeeded",
      "input_txo_ids": [
        "3de563a16d2da9656ce6c8aa9b12380b682c2e6aad0011fa8d6528c084078827",
        "fa242e21e2155e8f257cd75d2d2939000d0926946c2b7b812946e093165acadb"
      ],
      "output_txo_ids": [
        "badf415972dfc2dc6203ed90be132831ff29f394f65b0be5c35c79048d86af5b"
      ],
      "change_txo_ids": [
        "e992e718e1f28b67b0cf200e213af560fc7d5a236b3fec590f225b230f88257f"
      ],
      "sent_time": "2020-12-15 09:30:04 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null,
      "offset_count": 496
    }
  }
}
```

### Transaction Output Proofs

When constructing a transaction, the wallet produces a "proof" for each Txo minted by the transaction. This proof can be delivered to the recipient to confirm that they received the Txo from the sender.

#### Verify Proof



## Contributing

See [CONTRIBUTING](./CONTRIBUTING.md).

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

### Linting

    ```
    RUST_LOG=info SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo clippy --all --all-features
    ```