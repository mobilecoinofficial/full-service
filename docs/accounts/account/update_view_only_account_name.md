---
description: Rename a view only account.
---

# Update View Only Account Name

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `name` | The new name for this account. |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "update_view_only_account_name",
  "params": {
    "acount_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
    "name": "Coins for birds"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "update_view_only_account_name",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Coins for birds",
      "first_block_index": "3500",
      "first_block_index": "40000",
      "object": "view_only_account",
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

