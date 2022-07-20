---
description: Get the details of all view only accounts in a given wallet.
---

# Get All

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "get_all_view_only_accounts",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "get_all_view_only_accounts",
    "result": {
        "account_ids": [
            "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5"
        ],
        "account_map": {
            "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5": {
                "object": "view_only_account",
                "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
                "name": "ts-test-2",
                "first_block_index": "0",
                "next_block_index": "679442",
                "main_subaddress_index": "0",
                "change_subaddress_index": "1",
                "next_subaddress_index": "2"
            }
        }
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
