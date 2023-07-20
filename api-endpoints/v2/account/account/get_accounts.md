---
description: Get the details of all accounts in a given wallet.
---

# Get Accounts

## Request

| Optional Param | Purpose                                                  | Requirements |
| -------------- | -------------------------------------------------------- | ------------ |
| `offset`       | The pagination offset. Results start at the offset index |              |
| `limit`        | Limit for the number of results                          |              |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "get_accounts",
    "jsonrpc": "2.0",
    "id": 1,
    "params": {}
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_accounts",
  "result":{
    "account_ids":[
      "d9e1ed9c4e49b5ef1671cbd95b45cea1aa1da37de1240e9bbd989a8ab908369f",
      "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa"
    ],
    "account_map":{
      "d9e1ed9c4e49b5ef1671cbd95b45cea1aa1da37de1240e9bbd989a8ab908369f":{
        "id":"d9e1ed9c4e49b5ef1671cbd95b45cea1aa1da37de1240e9bbd989a8ab908369f",
        "name":"Alice",
        "key_derivation_version":"2",
        "main_address":"VE55siJdaM1xrF7ZHQM4kGzx7vFFQgLb5y3oXNBcsDGHz1Pnjjp6BYwgxo8FukLZ7WMuQfySyDfD6BxFFqK9psJaRYR16NZ9fxZj15Goit",
        "next_subaddress_index":"2",
        "first_block_index":"1769448",
        "next_block_index":"1769458",
        "recovery_mode":false,
        "fog_enabled":false,
        "view_only":false
      },
      "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa": {
        "id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "name": "Carol",
        "key_derivation_version": "2",
        "main_address": "8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
        "next_subaddress_index": "4",
        "first_block_index": "1769454",
        "next_block_index": "1769496",
        "recovery_mode": false,
        "fog_enabled": false,
        "view_only": false
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
