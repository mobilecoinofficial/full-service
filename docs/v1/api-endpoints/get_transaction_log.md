# Get Transaction Log

## Parameters

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_transaction_log",
  "params": {
    "transaction_log_id": "914e703b5b7bc44b61bb3657b4ee8a184d00e87a728e2fe6754a77a38598a800"
  },
  "jsonrpc": "2.0",
  "id": 1
}
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
      "failure_message": null
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

