---
description: Get the JSON representation of the TXO object in the transaction log.
---

# Get Transaction Object

## Parameters

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_transaction_object",
  "params": {
    "transaction_log_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_transaction_object",
  "result": {
    "transaction": ...
  }
}
```
{% endtab %}
{% endtabs %}

