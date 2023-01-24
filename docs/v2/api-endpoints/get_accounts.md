---
description: Get the details of all accounts in a given wallet.
---

# Get Accounts

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `offset` | | |
| `limit` | | |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "get_accounts",
    "jsonrpc": "2.0",
    "id": 1,
    "params": {}
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"get_accounts",
  "result":{
    "account_ids":[
      "f3b957b5140d8a7d6b3204aaba96489a293f8316772462c982a262f822b35bae",
      "589deddcb912f52787b44d9bd76c9d6f94052bc6ece975f497ba1fd6ba9c067e"
    ],
    "account_map":{
      "589deddcb912f52787b44d9bd76c9d6f94052bc6ece975f497ba1fd6ba9c067e":{
        "id":"589deddcb912f52787b44d9bd76c9d6f94052bc6ece975f497ba1fd6ba9c067e",
        "name":"Alice",
        "key_derivation_version":"2",
        "main_address":"GJ3yis7S8ucUAYsmouuUbxMEm7q6CRsQ6fU3CjbJ9mSD8MrRMt839mr74n1y5UrzMqDxkfrjLkgu31u55koP15Aj1syHMzmu6cWp4pEPYh",
        "next_subaddress_index":"2",
        "first_block_index":"0",
        "next_block_index":"1352091",
        "recovery_mode":false,
        "fog_enabled":false,
        "view_only":false
      },
      "f3b957b5140d8a7d6b3204aaba96489a293f8316772462c982a262f822b35bae":{
        "id":"f3b957b5140d8a7d6b3204aaba96489a293f8316772462c982a262f822b35bae",
        "name":"Bob",
        "key_derivation_version":"2",
        "main_address":"3z2TYXp3E9bCh5K5HGmgcGzkSJyyySmuYLtrNK5Qwt54ZM9yhSALeJdE5RnfyBXwFD4GZQb54Qv5AmhPsFpZgpr1p9tAtT5SvrzBFvK4LB3",
        "next_subaddress_index":"2",
        "first_block_index":"1346263",
        "next_block_index":"1352091",
        "recovery_mode":false,
        "fog_enabled":false,
        "view_only":false
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

