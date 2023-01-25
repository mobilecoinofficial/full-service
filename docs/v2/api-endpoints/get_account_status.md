---
description: >-
  Get the current status of a given account. The account status includes both
  the account object and the balance object.
---

# Get Account Status

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_account_status",
  "params": {
     "account_id": "b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"get_account_status",
  "result":{
    "account":{
      "id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
      "name":"Bob",
      "key_derivation_version":"2",
      "main_address":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
      "next_subaddress_index":"2",
      "first_block_index":"1352037",
      "next_block_index":"1352685",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":false
    },
    "network_block_height":"1352685",
    "local_block_height":"1352685",
    "balance_per_token":{
      
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

