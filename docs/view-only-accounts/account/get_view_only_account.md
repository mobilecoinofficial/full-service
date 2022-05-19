---
description: Get the details of a given view only account.
---

# Get View Only Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The view only account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_view_only_account",
  "params": {
    "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_view_only_account",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Coins for cats",
      "first_block_index": "3500",
      "next_block_index": "3700",
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
If the account is not in the database, you will receive the following error message:

```text
{
  "error": "Database(AccountNotFound(\"a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10\"))",
  "details": "Error interacting with the database: Account Not Found: a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
}
```
{% endhint %}

