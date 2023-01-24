---
description: Get assigned addresses for an account.
---

# Get Address

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `public_address_b58` | The public address b58 string to query for. |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_address",
  "params": {
    "public_address_b58": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"get_addresses",
  "result":{
    "public_addresses":[
      "8fG7VsHGGW5dQdFABRZc4R1Bsd3FxkbvTCexjybPNUYyAhbmFt95XGyZA9ZGahdaBa9XEHLJUPbL9b1biaJkZRzPBp7YQds58NdaLk4rsR9"
    ],
    "address_map":{
      "8fG7VsHGGW5dQdFABRZc4R1Bsd3FxkbvTCexjybPNUYyAhbmFt95XGyZA9ZGahdaBa9XEHLJUPbL9b1biaJkZRzPBp7YQds58NdaLk4rsR9":{
        "public_address_b58":"8fG7VsHGGW5dQdFABRZc4R1Bsd3FxkbvTCexjybPNUYyAhbmFt95XGyZA9ZGahdaBa9XEHLJUPbL9b1biaJkZRzPBp7YQds58NdaLk4rsR9",
        "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
        "metadata":"Change",
        "subaddress_index":"18446744073709551614"
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

