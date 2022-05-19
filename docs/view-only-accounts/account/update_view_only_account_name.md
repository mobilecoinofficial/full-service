---
description: Rename a view only account.
---

# Update Name

## Parameters

| Required Param | Purpose                                      | Requirements                      |
| -------------- | -------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |
| `name`         | The new name for this account.               |                                   |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
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
```
{
    "method": "update_view_only_account_name",
    "result": {
        "view_only_account": {
            "object": "view_only_account",
            "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
            "name": "test-2",
            "first_block_index": "0",
            "next_block_index": "679741",
            "main_subaddress_index": "0",
            "change_subaddress_index": "1",
            "next_subaddress_index": "2"
        }
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
