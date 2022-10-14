---
description: Verify whether an address is correctly b58-encoded.
---

# Verify Address

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "verify_address",
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
  "method": "verify_address",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

