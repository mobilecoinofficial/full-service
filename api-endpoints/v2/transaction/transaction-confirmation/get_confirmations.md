---
description: >-
  A TXO constructed by this wallet will contain a confirmation number, which can
  be shared with the recipient to verify the association between the sender and
  this TXO.
---

# Get Confirmations

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Param                | Purpose                                                       | Requirements                                  |
| -------------------- | ------------------------------------------------------------- | --------------------------------------------- |
| `transaction_log_id` | The transaction log ID for which to get confirmation numbers. | The transaction log must exist in the wallet. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

When calling `get_confirmations` for a transaction, only the confirmation numbers for the `output_txo_ids` are returned.

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_confirmations",
  "params": {
    "transaction_log_id": "daf0c1439633d1d53a13b9bf086946032c20bef882d5bd7735b4a99816c24657"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_confirmations",
  "result": {
    "confirmations": [
      {
        "txo_id": "245669e1ced312bfe5a1a7e99c77918acf7bb5b4e69eb21d8ef74961b8dcc07e",
        "txo_index": "5367192",
        "confirmation": "0a20d0257c93a691dba8e9aa136e9edb7d6882470e92645ed3e08ea43d8570f0182e"
      }
    ]
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
