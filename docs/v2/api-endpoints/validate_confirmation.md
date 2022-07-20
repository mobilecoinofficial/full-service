# Validate Confirmations

A sender can provide the confirmation numbers from a transaction to the recipient, who then verifies for a specific TXO ID \(note that TXO ID is specific to the TXO, and is consistent across wallets. Therefore the sender and receiver will have the same TXO ID for the same TXO which was minted by the sender, and received by the receiver\).

## Parameters

| Param | Description |  |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `txo_id` | The ID of the TXO for which to validate the confirmation number. | TXO must be a received TXO. |
| `confirmation` | The confirmation number to validate. | The confirmation number should be delivered by the sender of the Txo in question. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "validate_confirmation",
  "params": {
    "account_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
    "txo_id": "bbee8b70e80837fc3e10bde47f63de41768ee036263907325ef9a8d45d851f15",
    "confirmation": "0a2005ba1d9d871c7fb0d5ba7df17391a1e14aad1b4aa2319c997538f8e338a670bb"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "validate_confirmation",
  "result": {
    "validated": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

