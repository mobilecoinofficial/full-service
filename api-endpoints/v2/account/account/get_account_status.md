---
description: >-
  Get the current status of a given account. The account status includes both
  the account object and the balance object.
---

# Get Account Status

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
  "method": "get_account_status",
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
  "method":"get_account_status",
  "result":{
    "account":{
      "id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name":"Alice",
      "key_derivation_version":"2",
      "main_address":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index":"2",
      "first_block_index": "1769454",
      "next_block_index": "1769496",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":false
    },
    "network_block_height":"1352685",
    "local_block_height":"1352685",
    "balance_per_token":{
      "0":{
        "max_spendable":"8039600015840",
        "unverified":"0",
        "unspent":"8040000067868",
        "pending":"0",
        "spent":"8065834220882873",
        "secreted":"0",
        "orphaned":"0"
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
