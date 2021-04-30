---
description: >-
  A Transaction Log is a record of a MobileCoin transaction that was constructed
  and sent from this wallet, or that was received at an address belonging to an
  account in this wallet.
---

# Transaction Log

Due to the privacy properties of the MobileCoin ledger, transactions are ephemeral. Once they have been created, they only exist until they are validated, and then only the outputs are written to the ledger. For this reason, the Full-service Wallet stores transactions in the `transaction_log` table in order to preserve transaction history.

## Attributes

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | String, value is "transaction\_log" | String representing the object's type. Objects of the same type share the same value. |
| `transaction_log_id` | Integer | Unique identifier for the transaction log. This value is not associated to the ledger. |
| `direction` | String | A string that identifies if this transaction log was sent or received. Valid values are "sent" or "received". |
| `is_sent_recovered` | Boolean | Flag that indicates if the sent transaction log was recovered from the ledger. This value is null for "received" transaction logs. If true, some information may not be available on the transaction log and its TXOs without user input. If true, the fee `receipient_address_id`, fee, and `sent_time` will be null without user input. |
| `account_id` | String | Unique identifier for the assigned associated account. If the transaction is outgoing, this account is from whence the TXO came. If received, this is the receiving account. |
| `recipient_address_id` | String | Unique identifier for the recipient associated account. Only available if direction is "sent". |
| `assigned_address_id` | String | Unique identifier for the assigned associated account. Only available if direction is "received". |
| `value_pmob` | String \(uint64\) | Value in pico MOB associated to this transaction log. |
| `fee_pmob` | String \(uint64\) | Fee in pico MOB associated to this transaction log. Only on outgoing transaction logs. Only available if direction is "sent". |
| `submitted_block_index` | String \(uint64\) | The block index of the highest block on the network at the time the transaction was submitted. |
| `finalized_block_index` | String \(uint64\) | The scanned block block index in which this transaction occurred. |
| `status` | String | String representing the transaction log status. On "sent", valid statuses are "built", "pending", "succeeded", "failed". On "received", the status is "succeeded". |
| `input_txo_ids` | List | A list of the IDs of the TXOs which were inputs to this transaction. |
| `output_txo_ids` | List | A list of the IDs of the TXOs which were outputs of this transaction. |
| `change_txo_ids` | List | A list of the IDs of the TXOs which were change in this transaction. |
| `sent_time` | Timestamp | Time at which sent transaction log was created. Only available if direction is "sent". This value is null if "received" or if the sent transactions were recovered from the ledger \(`is_sent_recovered = true`\). |
| `comment` | String | An arbitrary string attached to the object. |
| `failure_code` | Integer | Code representing the cause of "failed" status. |
| `failure_message` | String | Human parsable explanation of "failed" status. |
| `offset_count` | Integer | The value to offset pagination requests for `transaction_log` list. Requests will exclude all list items up to and including this object. |

## Example

{% tabs %}
{% tab title="Received" %}
```text
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
  "failure_message": null,
  "offset_count": 2252
}
```
{% endtab %}

{% tab title="Sent - Failed" %}
```text
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
  "failure_message:": "Contains sent key image.",
  "offset_count": 2252
}
```
{% endtab %}

{% tab title="Sent - Success, Recovered" %}
```text
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
  "failure_message": null,
  "offset_count": 2252
}
```
{% endtab %}
{% endtabs %}

## Methods

### `get_transaction_object`

Get the JSON representation of the TXO object in the transaction log.

{% tabs %}
{% tab title="Request Body" %}
```text
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
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_transaction_object",
  "result": {
    "transaction": ...
  }
}
```
{% endtab %}
{% endtabs %}

### `get_transaction_log`

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
```text
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
{% endtab %}

{% tab title="Response" %}
```text
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
      "failure_message": null,
      "offset_count": 37
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_all_transaction_logs_for_account`

Get all transaction logs for a given account.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
```text
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
{% endtab %}

{% tab title="Response" %}
```text
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
        "failure_message": null,
        "offset_count": 4
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
        "failure_message": null,
        "offset_count": 53
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_all_transaction_logs_for_block`

Get all transaction logs for a given block.

In the below example, the account in the wallet sent a transaction to itself. Therefore, there is one sent `transaction_log` in the block, and two received \(one for the change, and one for the output TXO sent to the same account that constructed the transaction\).

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `block_index` | The block on which to perform this action. | Block must exist in the wallet. |

{% tabs %}
{% tab title="Body Request" %}
```text
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
```
{% endtab %}

{% tab title="Response" %}
```text
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
        "failure_message": null,
        "offset_count": 53
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
        "failure_message": null,
        "offset_count": 54
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
        "failure_message": null,
        "offset_count": 55
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_all_transaction_logs_ordered_by_block`

Get the transaction logs, grouped by the `finalized_block_index`, in ascending order.

{% tabs %}
{% tab title="Body Request" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_transaction_logs_ordered_by_block",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="Response" %}
```text
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
        "failure_message": null,
        "offset_count": 51
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
        "failure_message": null,
        "offset_count": 52
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
        "failure_message": null,
        "offset_count": 53
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

