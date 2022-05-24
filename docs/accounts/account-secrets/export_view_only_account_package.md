# Export View Only Account Package

## Parameters

| Required Param | Purpose                                      | Requirements                      |
| -------------- | -------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "export_view_only_account_package",
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
    "method": "export_view_only_account_package",
    "result": {
        "json_rpc_request": {
            "method": "import_view_only_account",
            "params": {
                "account": {
                    "object": "view_only_account",
                    "account_id": "6d95067c5fcc0dd7bbcdd42d49cc3571fe1bb2597a9c397c75b7280eca534208",
                    "name": "testing",
                    "first_block_index": "661194",
                    "next_block_index": "693043",
                    "main_subaddress_index": "0",
                    "change_subaddress_index": "1",
                    "next_subaddress_index": "2"
                },
                "secrets": {
                    "object": "view_only_account_secrets",
                    "view_private_key": "0a20ec42a30f81c5367042516bcbe499def7346f39870ef0f7d1a467e5325d845007",
                    "account_id": "6d95067c5fcc0dd7bbcdd42d49cc3571fe1bb2597a9c397c75b7280eca534208"
                },
                "subaddresses": [
                    {
                        "object": "view_only_subaddress",
                        "public_address": "3b63EnYDAaGCoeZ473YwcsoHk47qDcuFo6emkFKtiEfSrNy5NuzLpLCau7yJJ5WfavVjMsK8Qa7FKBDEQF5UkRadFVFKEBEaji2FvfLJRTh",
                        "account_id": "6d95067c5fcc0dd7bbcdd42d49cc3571fe1bb2597a9c397c75b7280eca534208",
                        "comment": "Main",
                        "subaddress_index": "0",
                        "public_spend_key": "0a203cbe82bc9af6cc20d485534f79c5cc41a887099f424d64b8d9ee3ae4599d7544"
                    },
                    {
                        "object": "view_only_subaddress",
                        "public_address": "88hRd28N7srH1wtydh9hWBq8EFfgPy492prHXqvuF4kRu6i6rk6dMNNsGN7H8rdDUcTCCBGDzN14nDEvfWS8W5GytJuUVkD9emCYr9cX7Sr",
                        "account_id": "6d95067c5fcc0dd7bbcdd42d49cc3571fe1bb2597a9c397c75b7280eca534208",
                        "comment": "Change",
                        "subaddress_index": "1",
                        "public_spend_key": "0a20c61def43c7b62ca7caeec567c23c1fd62d8a627e385b4206f9f91e80af85ea53"
                    }
                ]
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

