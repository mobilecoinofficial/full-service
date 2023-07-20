# Get TXO Block Index

Allows the public key of a tx out to be checked against the ledger, and if it exists will return the block index

## Request

| Param        | Description                |                                 |
| ------------ | -------------------------- | ------------------------------- |
| `public_key` | The public key of the txo. | public key is hex encoded bytes |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method":"get_txo_block_index",
  "params":{
    "public_key":"167628bd36b6c70aed289cdb3d61d22eb4b40a48f304c484a8f8de781ab54565"
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}

{% tab title="Response Success" %}
```
{
  "method": "get_txo_block_index",
  "result": {
    "block_index": "1769541"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response Failed" %}
```
{
    "method": "get_txo_block_index",
    "error": {
        "code": -32603,
        "message": "InternalError",
        "data": {
            "server_error": "LedgerDB(NotFound)",
            "details": "Error with LedgerDB: Record not found"
        }
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
