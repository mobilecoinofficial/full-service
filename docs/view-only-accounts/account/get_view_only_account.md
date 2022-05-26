---
description: Get the details of a given view only account.
---

# Get

## Parameters

| Required Param | Purpose                                                | Requirements                      |
| -------------- | ------------------------------------------------------ | --------------------------------- |
| `account_id`   | The view only account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "get_view_only_account",
    "params": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "get_view_only_account",
    "result": {
        "view_only_account": {
            "object": "view_only_account",
            "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
            "name": "ts-test-2",
            "first_block_index": "0",
            "next_block_index": "679739",
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

{% hint style="warning" %}
If the account is not in the database, you will receive the following error message:

```
{
  "error": "Database(AccountNotFound(\"a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10\"))",
  "details": "Error interacting with the database: Account Not Found: a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
}
```
{% endhint %}
