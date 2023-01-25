---
description: Assign an address to a given account.
---

# Assign Address For Account

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40-L43)

### Required Params
| Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

### Optional Params
| Param | Purpose | Requirements |
| :--- | :--- | :--- |
| ​`metadata` | The metadata for this address. | String; can contain stringified JSON. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41-L43)

## Examples

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "assign_address_for_account",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "metadata": "For transactions from Carol"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"assign_address_for_account",
  "result":{
    "address":{
      "public_address_b58":"2tzZu4aYJH6MmfNJo2iqPFw6jmJJouKsXkHLtXBBB6zgoSgsQ76YBPcqLJYJdY1yfNYWfWDMMf9BsLPD6QHiq6NeJVM7fzfDxqnXs6kEQtK",
      "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
      "metadata":"For transactions from Carol",
      "subaddress_index":"2"
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

