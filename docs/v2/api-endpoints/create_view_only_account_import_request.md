# Export View Only Account Import Request

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                      |
| -------------- | -------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "export_view_only_account_import_request",
    "params": {
        "account_id": "6d95067c5fcc0dd7bbcdd42d49cc3571fe1bb2597a9c397c75b7280eca534208"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "export_view_only_account_import_request",
    "result": {
        "json_rpc_request": {
            "method": "import_view_only_account",
            "params": {
                "account": {
                    "view_private_key": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
                    "spend_public_key": "fcewc434g5353v535323f43f43f43g5342v3b67n8576453f4dcv56b77n857b46",
                    "name": "Coins for cats",
                    "first_block_index": "3500",
                    "next_block_index": "4000",
                }
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

