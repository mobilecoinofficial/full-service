# Create View Only Account Sync Request

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet as a view only account. |

{% tabs %}
{% tab title="Request" %}
```
{
    "method": "create_view_only_account_sync_request",
    "params": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "create_view_only_account_sync_request",
    "result": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
        "incomplete_txos_encoded": [
            "0a2d0a220a20528c20f24b7b85203a475beaf904da73fd626805a6bf93e0d56b8fbba87b9c3811086bc8567df7354e12220a209e715ba7c0ea72c650a4b9ff06777c8f860803332ce33d9caa4f13e413a8f3001a220a2060ebdd120439102051664ee8b45988d5e236d44da802b5a4b11019e0f859207c22560a54b279a140856590907927242871b62242486269b9ce51892ac91d91d187bd69fd90f59afbd30ccb805bd39c372ce8b24b2bd0eef6e4d97e5f0092d52c4ebbbb2c301bd6d25e1368ada8636c7978af2e20d6d40100"
        ]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
