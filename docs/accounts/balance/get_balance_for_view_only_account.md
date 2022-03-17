---
description: Get the current balance for a given view only account.
---

# Get Balance For View Only Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_balance_for_view_only_account",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_balance_for_view_only_account",
  "result": {
    "balance": {
        "object": "balance",
        "received": "10000000000000",
        "network_block_height": "468847",
        "local_block_height": "468847",
        "account_block_height": "468847",
        "is_synced": true
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

