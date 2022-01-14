---
description: 'Get the transaction protocol for MobileCoin'
---

# Get MobileCoin Protocol Transaction

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `transaction_log_id` | The id of the transaction log. | Must be a valid id for a transaction. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_mc_protocol_transaction",
  "params": {
    "transaction_log_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_mc_protocol_transaction",
  "result": {
    "transaction": ...
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

