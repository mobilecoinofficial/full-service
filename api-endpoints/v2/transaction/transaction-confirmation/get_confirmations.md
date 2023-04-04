---
description: >-
  A TXO constructed by this wallet will contain a confirmation number, which can
  be shared with the recipient to verify the association between the sender and
  this TXO.
---

# Get Confirmations

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Param                | Purpose                                                       | Requirements                                  |
| -------------------- | ------------------------------------------------------------- | --------------------------------------------- |
| `transaction_log_id` | The transaction log ID for which to get confirmation numbers. | The transaction log must exist in the wallet. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

When calling `get_confirmations` for a transaction, only the confirmation numbers for the `output_txo_ids` are returned.

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_confirmations",
  "params": {
    "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_confirmations",
  "result":{
    "confirmations":[
      {
        "txo_id":"fa9b95605688898f2d6bca52fb39608bd80eca74a342e3033f6dc0eef1c4e542",
        "txo_index":"4061770",
        "confirmation":"0a20bf46e135b4eeb5c45fcc8ee69a5e564469fd3985269010f6738a96f832992afe"
      }
    ]
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
