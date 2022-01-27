---
description: Get the current balance for a given address.
---

# Get Balance For Address

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_balance_for_address",
  "params": {
    "address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
  },
  "jsonrpc": "2.0",
  "api_version": "2",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_balance_for_address",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_height": "152961",
      "local_block_height": "152961",
      "account_block_height": "152961",
      "is_synced": true,
      "unspent_pmob": "11881402222024",
      "pending_pmob": "0",
      "spent_pmob": "84493835554166",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  "api_version": "2"
}
```
{% endtab %}
{% endtabs %}

