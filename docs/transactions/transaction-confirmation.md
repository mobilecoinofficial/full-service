---
description: >-
  When constructing a transaction, the wallet produces a "confirmation number"
  for each TXO minted by the transaction.
---

# Confirmation

The confirmation number can be delivered to the recipient to prove that they received the TXO from that particular sender.

## Attributes

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | String, value is "confirmation" | String representing the object's type. Objects of the same type share the same value. |
| `txo_id` | String | Unique identifier for the TXO. |
| `txo_index` | String | The index of the TXO in the ledger. |
| `confirmation` | String | A string with a confirmation number that can be validated to confirm that another party constructed or had knowledge of the construction of the associated TXO. |

## Example

```text
{
  "object": "confirmation",
  "txo_id": "873dfb8c...",
  "txo_index": "1276",
  "confirmation": "984eacd..."
}
```

## Methods

### `get_confirmations`

A TXO constructed by this wallet will contain a confirmation number, which can be shared with the recipient to verify the association between the sender and this TXO. When calling `get_confirmations` for a transaction, only the confirmation numbers for the `output_txo_ids` are returned.

| Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID for which to get confirmation numbers. | The transaction log must exist in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_confirmations",
        "params": {
          "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_confirmations",
  "result": {
    "confirmations": [
      {
        "object": "confirmation",
        "txo_id": "9e0de29bfee9a391e520a0b9411a91f094a454ebc70122bdc0e36889ab59d466",
        "txo_index": "458865",
        "confirmation": "0a20faca10509c32845041e49e009ddc4e35b61e7982a11aced50493b4b8aaab7a1f"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `validate_confirmation`

A sender can provide the confirmation numbers from a transaction to the recipient, who then verifies for a specific TXO ID \(note that TXO ID is specific to the TXO, and is consistent across wallets. Therefore the sender and receiver will have the same TXO ID for the same TXO which was minted by the sender, and received by the receiver\) with the following:

| Param | Description |  |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `txo_id` | The ID of the TXO for which to validate the confirmation number. | TXO must be a received TXO. |
| `confirmation` | The confirmation number to validate. | The confirmation number should be delivered by the sender of the Txo in question. |

{% tabs %}
{% tab title="Body Request" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "validate_confirmation",
        "params": {
          "account_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
          "txo_id": "bbee8b70e80837fc3e10bde47f63de41768ee036263907325ef9a8d45d851f15",
          "confirmation": "0a2005ba1d9d871c7fb0d5ba7df17391a1e14aad1b4aa2319c997538f8e338a670bb"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "validate_confirmation",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

