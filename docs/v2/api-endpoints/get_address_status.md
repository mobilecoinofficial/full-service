---
description: Get the current balance for a given address. The response will have a map of the total values for each token_id that is present at that address. If no tokens are found at that address, the map will be empty. Orphaned will always be 0 for addresses.
---

# Get Address Status

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)
| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `min_block_index` | The minimum block index to filter on txos received | |
| `max_block_index` | The maximum block index to filter on txos received | |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_address_status",
  "params": {
    "address": "GJ3yis7S8ucUAYsmouuUbxMEm7q6CRsQ6fU3CjbJ9mSD8MrRMt839mr74n1y5UrzMqDxkfrjLkgu31u55koP15Aj1syHMzmu6cWp4pEPYh"
  },
  "jsonrpc": "2.0",
  "api_version": "2",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"get_address_status",
  "result":{
    "address":{
      "public_address_b58":"GJ3yis7S8ucUAYsmouuUbxMEm7q6CRsQ6fU3CjbJ9mSD8MrRMt839mr74n1y5UrzMqDxkfrjLkgu31u55koP15Aj1syHMzmu6cWp4pEPYh",
      "account_id":"589deddcb912f52787b44d9bd76c9d6f94052bc6ece975f497ba1fd6ba9c067e",
      "metadata":"Main",
      "subaddress_index":"0"
    },
    "account_block_height":"1352709",
    "network_block_height":"1352709",
    "local_block_height":"1352709",
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

