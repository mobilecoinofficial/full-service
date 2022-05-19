---
description: Get the details of all view only accounts in a given wallet.
---

# Get All View Only Accounts

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "get_all_view_only_accounts",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_all_view_only_accounts",
  "result": {
    "account_ids": [
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0"
    ],
    "account_map": {
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52": {
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Coins for cats",
      "first_block_index": "3500",
      "next_block_index": "3700",
      },
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0": {
        "account_id": "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0",
      "name": "Coins for cats",
      "first_block_index": "200",
      "next_block_index": "3700",
      "object": "view_only_account" 
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

