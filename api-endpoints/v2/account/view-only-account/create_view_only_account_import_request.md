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
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "create_view_only_account_import_request",
  "result": {
    "json_rpc_request": {
      "method": "import_view_only_account",
      "params": {
        "view_private_key": "0a20952e7ea32b80f5249c8fa470913dd67b1f722e12f516d0aff8a64f95faf6cb07",
        "spend_public_key": "0a20e22bcedd966ac737cbe53445dc983433072b771440e7c9f2619f775db2cc0448",
        "name": "Alice",
        "first_block_index": "1769454",
        "next_subaddress_index": "4"
      },
      "jsonrpc": "2.0",
      "id": 1
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
