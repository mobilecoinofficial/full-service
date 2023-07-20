# Validate Confirmations

A sender can provide the confirmation numbers from a transaction to the recipient, who then verifies for a specific TXO ID (note that TXO ID is specific to the TXO, and is consistent across wallets. Therefore the sender and receiver will have the same TXO ID for the same TXO which was minted by the sender, and received by the receiver).

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Param          | Description                                                      |                                                                                   |
| -------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `account_id`   | The account on which to perform this action.                     | Account must exist in the wallet.                                                 |
| `txo_id`       | The ID of the TXO for which to validate the confirmation number. | TXO must be a received TXO.                                                       |
| `confirmation` | The confirmation number to validate.                             | The confirmation number should be delivered by the sender of the Txo in question. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "validate_confirmation",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "txo_id": "245669e1ced312bfe5a1a7e99c77918acf7bb5b4e69eb21d8ef74961b8dcc07e",
    "confirmation": "0a20d0257c93a691dba8e9aa136e9edb7d6882470e92645ed3e08ea43d8570f0182e"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "validate_confirmation",
  "result": {
    "validated": true
  },
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}
