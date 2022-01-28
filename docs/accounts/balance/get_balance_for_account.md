---
description: Get the current balance for a given account.
---

# Get Balance For Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_balance_for_account",
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
  "method": "get_balance_for_account",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_height": "152918",
      "local_block_height": "152918",
      "account_block_height": "152003",
      "is_synced": false,
      "unspent_pmob": "110000000000000000",
      "pending_pmob": "0",
      "spent_pmob": "0",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

