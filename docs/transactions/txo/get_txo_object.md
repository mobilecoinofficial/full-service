---
description: Get the JSON representation of the "TXO" object in the ledger.
---

# get\_txo\_object

## Parameters

| Parameter | Purpose | Requirements |
| :--- | :--- | :--- |
| `txo_id` | A TXO identifier. |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_txo_object",
  "params": {
    "txo_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
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
    "txo": ...
  }
}
```
{% endtab %}
{% endtabs %}

