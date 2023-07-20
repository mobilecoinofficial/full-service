---
description: Create a new account in the wallet.
---

# Create Account

## Request

| Optional Param | Purpose                                                                                                                                                              | Requirements                                            |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| `name`         | A label for this account.                                                                                                                                            | A label can have duplicates, but it is not recommended. |
| `fog_info`     | The [Fog Info](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/account\_key.rs#L67) to include in public addresses |                                                         |

## Response

{% tabs %}
{% tab title="Request Body" %}
```json
{
  "method": "create_account",
  "params": {
    "name": "Alice"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```json
{
  "method":"create_account",
  "result":{
    "account":{
      "id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name":"Alice",
      "key_derivation_version":"2",
      "main_address":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index":"2",
      "first_block_index":"1769454",
      "next_block_index":"1769454",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":false
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
