# Get Transaction Logs For Account

## Parameters

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |
| `offset` | The pagination offset. Results start at the offset index. Optional, defaults to 0. | |
| `limit` | Limit for the number of results. Optional, defaults to 100 | |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_transaction_logs_for_account",
  "params": {
    "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
    "offset": "2",
    "limit": "1"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_transaction_logs_for_account",
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
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
