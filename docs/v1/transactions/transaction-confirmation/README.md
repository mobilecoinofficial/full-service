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
| `object` | string, value is "confirmation" | String representing the object's type. Objects of the same type share the same value. |
| `txo_id` | string | Unique identifier for the TXO. |
| `txo_index` | string | The index of the TXO in the ledger. |
| `confirmation` | string | A string with a confirmation number that can be validated to confirm that another party constructed or had knowledge of the construction of the associated TXO. |

## Example

```text
{
  "object": "confirmation",
  "txo_id": "873dfb8c...",
  "txo_index": "1276",
  "confirmation": "984eacd..."
}
```

