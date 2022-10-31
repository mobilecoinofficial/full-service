---
description: >-
  After building a tx_proposal, you can get the receipts for that transaction
  and provide it to the recipient so they can poll for the transaction status.
---

# Create Receiver Receipts

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `tx_proposal` |  |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "create_receiver_receipts",
  "params": {
    "tx_proposal": {
      "input_txos": [
        "tx_out_proto": "439f9843vmtbgdrv5...",
        "value": "10000000000",
        "token_id": "0",
        "key_image": "dfj42v03mn40c353v53vvjyh5tr...",
      ],
      "payload_txos": [
        "tx_out_proto": "vr243095b89nvrimwec...",
        "value": "5000000000",
        "token_id": "0",
        "recipient_public_address_b58": "ewvr3m49350c932emr3cew2...",
      ],
      "change_txos": [
        "tx_out_proto": "defvr34v5t4b6b...",
        "value": "4060000000",
        "token_id": "0",
        "recipient_public_address_b58": "n23924mtb89vck31...",
      ]
      "fee": "40000000",
      "fee_token_id": "0",
      "tombstone_block_index": "152700",
      "tx_proto": "328fi4n94902cmjinrievn49jg9439nvr3v..."
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
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

