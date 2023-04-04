---
description: >-
  A receiver receipt contains the confirmation number and recipients can poll
  the receiver receipt for the status of the transaction.
---

# Receiver Receipt

## Attributes

| _Name_            | _Type_       | _Description_                                                                                                                                        |
| ----------------- | ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `public_key`      | string       | Hex-encoded public key for the TXO.                                                                                                                  |
| `tombstone_block` | string       | The block index after which this TXO would be rejected by consensus.                                                                                 |
| `confirmation`    | string       | Hex-encoded confirmation that can be validated to confirm that another party constructed or had knowledge of the construction of the associated TXO. |
| `masked_amount`   | MasketAmount | The encrypted amount in the TXO referenced by this receipt.                                                                                          |

## Example

```
{
  "object": "receiver_receipt",
  "public_key": "0a20d2118a065192f11e228e0fce39e90a878b5aa628b7613a4556c193461ebd4f67",
  "confirmation": "0a205e5ca2fa40f837d7aff6d37e9314329d21bad03d5fac2ec1fc844a09368c33e5",
  "tombstone_block": "154512",
  "amount": {
    "commitment": "782c575ed7d893245d10d7dd49dcffc3515a7ed252bcade74e719a17d639092d",
    "masked_value": "12052895925511073331",
    "masked_token_id": "123589105786482",
  }
}
```
