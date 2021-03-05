# Full Service API v1

The Full Service Wallet API provides JSON RPC 2.0 endpoints for interacting with your MobileCoin transactions.

## Overview

### Methods Overview

* [get_all_txos_by_account](#get-all-txos-for-a-given-account)
* [get_txo](#get-txo-details)
* [get_proofs](#get-proofs)
* [verify_proof](#verify-proof)
* [get_txo_object](#get-txo-object)
* [get_transaction_object](#get-transaction-object)
* [get_block_object](#get-block-object)

### Full Service Data Types Overview

The methods above return data representations of wallet contents. The Full Service API Data types are as follows:

* [txo](#the-txo-object)
* [proof](#the-proof-object)

## Full Service API Methods

### TXOs

#### Get All TXOs for a given account

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_txos_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        }
      }' \
  -X POST -H 'Content-type: application/json'  | jq

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
        "offset_count": 262,
        "proof": null,
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
        "offset_count": 501
        "proof": "0a204488e153cce1e4bcdd4419eecb778f3d2d2b024b39aaa29532d2e47e238b2e31",
        "public_key": "0a20e6736474f73e440686736bfd045d838c2b3bc056ffc647ad6b1c990f5a46b123",
        "received_account_id": "36fdf8fbdaa35ad8e661209b8a7c7057f29bf16a1e399a34aa92c3873dfb853c",
        "received_block_index": null,
        "spent_block_index": null,
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
        "received_block_index": "79",
        "spent_block_index": null,
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
        "received_block_index": null,
        "spent_block_index": null,
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
        "method": "get_all_txos_for_account",
        "params": {"account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
        }
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
          "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7"}}' \
  -X POST -H 'Content-type: application/json' | jq

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

### Transaction Output Proofs

When constructing a transaction, the wallet produces a "proof" for each Txo minted by the transaction. This proof can be delivered to the recipient to confirm that they received the Txo from the sender.

#### Get Proofs

A Txo constructed by this wallet will contain a proof, which can be shared with the recipient to verify the association between the sender and this Txo. When calling `get_proofs` for a transaction, only the proofs for the "output_txo_ids" are returned.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_proofs",
        "params": {
          "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq
{
  "method": "get_proofs",
  "result": {
    "proofs": [
      {
        "object": "proof",
        "txo_id": "bbee8b70e80837fc3e10bde47f63de41768ee036263907325ef9a8d45d851f15",
        "proof": "0a2005ba1d9d871c7fb0d5ba7df17391a1e14aad1b4aa2319c997538f8e338a670bb"
      }
    ]
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `transaction_log_id`   | The transaction log ID for which to get proofs.  | Transaction log must exist in the wallet  |

#### Verify Proof

A sender can provide the proofs from a transaction to the recipient, who then verifies for a specific txo_id (note that txo_id is specific to the txo, and is consistent across wallets. Therefore the sender and receiver will have the same txo_id for the same Txo which was minted by the sender, and received by the receiver) with the following:

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "verify_proof",
        "params": {
          "account_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
          "txo_id": "bbee8b70e80837fc3e10bde47f63de41768ee036263907325ef9a8d45d851f15",
          "proof": "0a2005ba1d9d871c7fb0d5ba7df17391a1e14aad1b4aa2319c997538f8e338a670bb"
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "verify_proof",
  "result": {
    "verified": true
  }
}
```

| Required Param | Purpose                  | Requirements              |
| :------------- | :----------------------- | :------------------------ |
| `account_id`   | The account on which to perform this action  | Account must exist in the wallet  |
| `txo_id`   | The ID of the Txo for which to verify the proof  | Txo must be a received Txo  |
| `proof`   | The proof to verify  | The proof should be delivered by the sender of the Txo in question |

#### Ledger and Transaction Data

To get the JSON representations of the objects which are used in the MobileCoin blockchain, you can use the following calls:

##### Get Transaction Object

Get the JSON representation of the "Tx" object in the transaction log.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_transaction_object",
        "params": {
          "transaction_log_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_transaction_object",
  "result": {
    "transaction": ...
  }
}
```

##### Get Tx0 Object

Get the JSON representation of the "Txo" object in the ledger.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_txo_object",
        "params": {
          "txo_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_txo_object",
  "result": {
    "txo": ...
  }
}
```

##### Get Block Object

Get the JSON representation of the "Block" object in the ledger.

```sh
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_block_object",
        "params": {
          "block_index": "3204",
        }
      }' \
  -X POST -H 'Content-type: application/json' | jq

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

### The Proof Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| txo_id | string | Unique identifier for the Txo.
| proof | string | A string with a proof that can be verified to confirm that another party constructed or had knowledge of the construction of the associated Txo.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "proof" | String representing the object's type. Objects of the same type share the same value.

#### Example Object

```json
{
  "object": "proof",
  "txo_id": "873dfb8c...",
  "proof": "984eacd..."
}
```

#### API Methods Returning Proof Objects

* [get_proofs](#get-proofs)
* [verify_proof](#verify-proof)
