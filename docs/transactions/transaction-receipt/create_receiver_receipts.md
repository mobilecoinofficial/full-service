---
description: >-
  After building a tx_proposal, you can get the receipts for that transaction
  and provide it to the recipient so they can poll for the transaction status.
---

# Create Receiver Receipts

## Parameters

| Required Param | Purpose | Description |
| :--- | :--- | :--- |
| `tx_proposal` |  |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "create_receiver_receipts",
        "params": {
          "tx_proposal": '$(cat tx_proposal.json)',
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
  "method": "create_receiver_receipts",
  "result": {
    "receiver_receipts": [
      {
        "object": "receiver_receipt",
        "public_key": "0a20d2118a065192f11e228e0fce39e90a878b5aa628b7613a4556c193461ebd4f67",
        "confirmation": "0a205e5ca2fa40f837d7aff6d37e9314329d21bad03d5fac2ec1fc844a09368c33e5",
        "tombstone_block": "154512",
        "amount": {
          "object": "amount",
          "commitment": "782c575ed7d893245d10d7dd49dcffc3515a7ed252bcade74e719a17d639092d",
          "masked_value": "12052895925511073331"
        }
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

