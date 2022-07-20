---
description: Assign an address to a given account.
---

# Assign Address For Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| ​`metadata` | The metadata for this address. | String; can contain stringified JSON. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "assign_address_for_account",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "metadata": "For transactions from Carol"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "assign_address_for_account",
  "result": {
    "address": {
      "object": "address",
      "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "metadata": "",
      "subaddress_index": "2"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

