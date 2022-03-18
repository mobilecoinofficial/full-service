---
description: Create a view-only account by importing the private key from an existing account
---

# Import View Only Account

## Parameters

| :--- | :--- | :--- |
| Required Param | Purpose | Requirements |
| `view-private-key` | The view private key for an existing account | |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |
| `next_subaddress_index` | The next known unused subaddress index for the account. |  |
| `first_block_index` | The block from which to start scanning the ledger. |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "import_view_only_account",
  "params": {
    "private-view-key": "0a207960bd832aae551ee03d6e5ab48baa229acd7ca4d2c6aaf7c8c4e77ac3e92307",
    "name": "Coins for cats"
    "next_subaddress_index": 2,
    "first_block_index": "3500"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "import_view_only_account",
  "result": {
    "account": {
      "object": "view_only_account_account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Coins for cats",
      "first_block_index": "3500",
      "next_block_index": "4000",
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

