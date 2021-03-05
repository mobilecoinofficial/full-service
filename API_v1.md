# Full Service API v1

The Full Service Wallet API provides JSON RPC 2.0 endpoints for interacting with your MobileCoin transactions.

## Overview

### Methods Overview

* [get_txo_object](#get-txo-object)
* [get_transaction_object](#get-transaction-object)
* [get_block_object](#get-block-object)

## Full Service API Methods

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

##### Get Txo Object

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