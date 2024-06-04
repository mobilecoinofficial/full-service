# Create View Only Account Sync Request

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                                             |
| -------------- | -------------------------------------------- | -------------------------------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet as a view only account. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

{% tabs %}
{% tab title="Request" %}
```json
{
    "method": "create_view_only_account_sync_request",
    "params": {
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
<pre class="language-json"><code class="lang-json"><strong>{
</strong>    "method": "create_view_only_account_sync_request",
    "result": {
        "txo_sync_request": {
            "account_id": "0474e3fbaec561bd6a31fff22bad73c3ed5576c6918d5a82f7797b2b2fc4d2dc",
            "txos": [
                {
                    "subaddress": 18446744073709551614,
                    "tx_out_public_key": "6057376adf75eccdc95518228d571e1f03bdcbc83ecabf1bce8263b52c009647"
                }
            ]
        }
    },
    "jsonrpc": "2.0",
    "id": 1
}
</code></pre>
{% endtab %}
{% endtabs %}
