# Sync View Only Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet as a view only account. |
| `completed_txos` | signed txos. A array of tuples (txoID, KeyImage) | |
| `subaddresses` | The subaddress to sync | |

### subaddress import json fields:
| field | description (all strings) |
| :--- | :--- |
| `object` | "view_only_subaddress" |
| `public_address` |A b58 encoding of the public address materials |
| `account_id` | The account that owns this subaddress |
| `comment` | Additional data associated with this address. |
| `subaddress_index` | The index of this address in the subaddress space for the account |
| `public_spend_key` | The public spend key for this addres |

## Example

{% tabs %}
{% tab title="Request" %}
```
{
    "method": "sync_view_only_account",
    "params": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
        "completed_txos": "[(asdasedeerwe..., sadjashdoauihdkahwk...)]",
        "subaddresses": "[{
            object: "view_only_subaddress",
            public_address: "USm3fpXnKG5EUBx2ndxBDMPVciP5hGey2Jh4NDv6gmeo1LkMeiKrLJUUBk6Z",
            account_id: "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
            comment: "target address",
            "subaddress_index: "5",
            public_spend_key: "asdsfpXnKG5EUBx2ndxBDMPVciP5hGey2Jh4NDv6gmeo1LkMeiKrLJUUBk6Z"
        }]"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "sync_view_only_account",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
