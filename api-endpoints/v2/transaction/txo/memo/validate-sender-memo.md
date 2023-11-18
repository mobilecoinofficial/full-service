# Validate Sender Memo

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Parameter        | Purpose                                              | Requirements               |
| ---------------- | ---------------------------------------------------- | -------------------------- |
| `txo_id`         | The TXO ID for which to validate the memo            |                            |
| `sender_address` | The public address of the expected sender of the txo | b58 encoded public address |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```json
{
  "method": "validate_sender_memo",
  "params": {
    "txo_id": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
    "sender_address": "33c32PjAPKBGLzVyDY6JXA1wunmRUmUqe4mRk24YJD2HsnFvBfZmHBqY8YEtL6zhacnqi5ZsKNFLgn2BgNoup2ihkMA63MYJK7tctXdHRDdKXX1EEiFpnXKESU6M9fKxHtKRbEzEnm27y3ydP5mA4sBwcRuW67ECauvHAK1rG71vdNcPMdc5j8ttBeJxyR38e8otkjJU2pAmhEjzXC6ZDRVy9tjvFSF6SQqtE9Auj5KX6VX2m",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```json
{
    "method": "validate_sender_memo",
    "result": {
        "validated": false
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
