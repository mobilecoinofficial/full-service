# Create New Subaddress Request

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet as a view only account. |
| `num_subaddresses_to_generate` | The number of desired subaddress. | |

## Example

{% tabs %}
{% tab title="Request" %}
```
{
    "method": "create_new_subaddresses_request",
    "params": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
        "num_subaddresses_to_generate": "10"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "create_new_subaddresses_request",
    "result": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
        "next_subaddress_index": "10",
        "num_subaddresses_to_generate": "10"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
