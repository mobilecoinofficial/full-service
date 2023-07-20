---
description: >-
  Get the current balance for a given address. The response will have a map of
  the total values for each token_id that is present at that address. If no
  tokens are found at that address, the map will be
---

# Get Address Status

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                                           |
| -------------- | -------------------------------------------- | ------------------------------------------------------ |
| `address`      | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

| Optional Param    | Purpose                                            | Requirements |
| ----------------- | -------------------------------------------------- | ------------ |
| `min_block_index` | The minimum block index to filter on txos received |              |
| `max_block_index` | The maximum block index to filter on txos received |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_address_status",
  "params": {
    "address": "8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_address_status",
  "result":{
    "address":{
      "public_address_b58":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "account_id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "metadata":"Main",
      "subaddress_index":"0"
    },
    "account_block_height":"1769486",
    "network_block_height":"1769486",
    "local_block_height":"1769486",
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
