# Sync View Only Account

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param          | Purpose                                                      | Requirements                                             |
| ----------------------- | ------------------------------------------------------------ | -------------------------------------------------------- |
| `account_id`            | The account on which to perform this action.                 | Account must exist in the wallet as a view only account. |
| `completed_txos`        | signed txos. A array of tuples (txoID, KeyImage)             |                                                          |
| `next_subaddress_index` | The updated next subaddress index to assign for this account |                                                          |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request" %}
```
{
    "method": "sync_view_only_account",
    "params": {
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "completed_txos": "[(asdasedeerwe..., sadjashdoauihdkahwk...)]",
        "next_subaddress_index": "3"
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
