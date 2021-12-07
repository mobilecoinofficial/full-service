---
description: Get an assigned address by index for an account.
---

# Get Address For Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |
| `index` | The subaddress index to lookup | The address must have already been assigned. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_address_for_account",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "index": 1
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_address_for_account",
  "result": {
      "address": {
        "object": "address",
        "public_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Main",
        "subaddress_index": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

