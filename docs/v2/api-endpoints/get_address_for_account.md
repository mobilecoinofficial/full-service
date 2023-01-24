---
description: Get an assigned address by index for an account.
---

# Get Address For Account At Index

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |
| `index` | The subaddress index to lookup | The address must have already been assigned. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_address_for_account",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "index": 1
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"get_address_for_account",
  "result":{
    "address":{
      "public_address_b58":"4mpdxAGvkSecdpPe1oZGFydyvUkbHmJXrqozPpzYJgq6CLADcpSRwndcf8VTXotvz4wHmCvChUqkZGeq1Wg3947siuUZMK12jchhnfK9aUJ",
      "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
      "metadata":"Legacy Change",
      "subaddress_index":"1"
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

