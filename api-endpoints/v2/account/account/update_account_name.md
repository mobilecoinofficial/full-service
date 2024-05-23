---
description: Rename an account.
---

# Update Account Name

## Request

| Required Param | Purpose                                      | Requirements                                            |
| -------------- | -------------------------------------------- | ------------------------------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet.                       |
| `name`         | The new name for this account.               | A label can have duplicates, but it is not recommended. |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "update_account_name",
    "params": {
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "name": "Carol"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"update_account_name",
  "result":{
    "account":{
      "id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name":"Carol",
      "key_derivation_version":"2",
      "main_address":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index":"2",
      "first_block_index":"1769454",
      "next_block_index":"1769454",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":false,
      "require_spend_subaddress":false
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
