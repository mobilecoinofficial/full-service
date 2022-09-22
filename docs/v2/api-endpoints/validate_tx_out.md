# Validate TxOut

Allows the public key of a tx out to be checked against the ledger, and if it exists will return the block index

## Request

| Param | Description |  |
| :--- | :--- | :--- |
| `public_key` | The public key of the txo. | public key is hex encoded bytes |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "validate_tx_out",
    "params": {
        "public_key": "6607d6189a4dc24823f8da6d42884a046947d00d9400e7033d7425d9df152269"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response Success" %}
```text
{
    "method": "validate_tx_out",
    "result": {
        "block_index": "682053"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response Failed" %}
```text
{
    "method": "validate_tx_out",
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

