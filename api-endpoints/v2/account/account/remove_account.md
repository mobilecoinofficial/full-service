---
description: Remove an account from a given wallet.
---

# Remove Account

## Request

| Required Param | Purpose                                      | Requirements                      |
| -------------- | -------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "remove_account",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "remove_account",
  "result": {
    "removed": true
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
