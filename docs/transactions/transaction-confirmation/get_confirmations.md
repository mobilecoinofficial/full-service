---
description: >-
  A TXO constructed by this wallet will contain a confirmation number, which can
  be shared with the recipient to verify the association between the sender and
  this TXO.
---

# Get Confirmations

## Parameters

| Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID for which to get confirmation numbers. | The transaction log must exist in the wallet. |

## Example

When calling `get_confirmations` for a transaction, only the confirmation numbers for the `output_txo_ids` are returned.

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_confirmations",
  "params": {
    "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_confirmations",
  "result": {
    "confirmations": [
      {
        "object": "confirmation",
        "txo_id": "9e0de29bfee9a391e520a0b9411a91f094a454ebc70122bdc0e36889ab59d466",
        "txo_index": "458865",
        "confirmation": "0a20faca10509c32845041e49e009ddc4e35b61e7982a11aced50493b4b8aaab7a1f"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

