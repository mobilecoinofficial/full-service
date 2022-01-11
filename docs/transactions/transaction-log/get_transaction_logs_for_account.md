# Get Transaction Logs For Account

## Parameters

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |
| `offset` | integer | The value to offset pagination requests. Requests will exclude all list items up to and including this object. |
| `limit` | integer | The limit of returned results. |

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
    "transaction_log_ids": [],
    "transaction_log_map": {}
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}