---
description: Get the JSON representation of the "TXO" object in the ledger.
---

# Get All TXOs For Address
### DEPRECATED

## Parameters

| Parameter | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_all_txos_for_address",
  "params": {
    "address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_txo_object",
  "result": {
    "txo": ...,
    "deprecated": true
  }
}
```
{% endtab %}
{% endtabs %}

