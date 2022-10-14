---
description: >-
  Get the transaction logs, grouped by the finalized_block_index, in ascending
  order.
---

# Get All Transaction Logs Ordered By Block

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "get_all_transaction_logs_ordered_by_block",
    "jsonrpc": "2.0",
    "id": 1
}
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
{% endtab %}
{% endtabs %}

