# Create View Only Account Import Request

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                      |
| -------------- | -------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "create_view_only_account_import_request",
    "params": {
        "account_id": "b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"create_view_only_account_import_request",
  "result":{
    "json_rpc_request":{
      "method":"import_view_only_account",
      "params":{
        "view_private_key":"0a2078062debfa72270373d13d52e228b2acc7e3d55790447e7a58905b986fc3780a",
        "spend_public_key":"0a208007986832d9269e62d9c0b2a33478ec761f8b6f6c32316bc8a993ed02964d51",
        "name":"Bob",
        "first_block_index":"1352037",
        "next_subaddress_index":"2"
      },
      "jsonrpc":"2.0",
      "id":1
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
