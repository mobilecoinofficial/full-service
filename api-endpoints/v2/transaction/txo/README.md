---
description: >-
  A TXO is a "Transaction Output." MobileCoin is a ledger built on the "Unspent
  Transaction Output" model (UTXO).
---

# Transaction Output TXO

## Transaction Output TXO

In order to construct a transaction, the wallet will select "Unspent Transaction Outputs" and perform a cryptographic operation to mark them as "spent" in the ledger. Then, it will mint new TXOs for the recipient.

### Attributes <a href="#object_method" id="object_method"></a>

| _Name_                              | _Type_          | _Description_                                                                                                                                                                                                                                                                                                                                                                                                                  |
| ----------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `id`                                | string          |                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `value`                             | string (uint64) | Available value for this account at the current `account_block_height`. If the account is syncing, this value may change.                                                                                                                                                                                                                                                                                                      |
| `token_id`                          | string (uint64) |                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `received_block_index`              | string (uint64) | Block index in which the TXO was received by an account.                                                                                                                                                                                                                                                                                                                                                                       |
| `spent_block_index`                 | string (uint64) | Block index in which the TXO was spent by an account.                                                                                                                                                                                                                                                                                                                                                                          |
| `account_id`                        | string          | The `account_id` for the account which has received this TXO. This account has spend authority.                                                                                                                                                                                                                                                                                                                                |
| `status`                            | string (enum)   | With respect to this account, the TXO may be "unverified", "unspent", "pending", "spent", "secreted" or "orphaned". For received TXOs received as an assigned address, the lifecycle is "unspent" -> "pending" -> "spent", the TXO is considered "orphaned" until its address is calculated -- in this case, there are manual ways to discover the missing assigned address for orphaned TXOs or to recover an entire account. |
| `target_key`                        | string (hex)    | A cryptographic key for this TXO.                                                                                                                                                                                                                                                                                                                                                                                              |
| `public_key`                        | string (hex)    | The public key for this TXO, can be used as an identifier to find the TXO in the ledger.                                                                                                                                                                                                                                                                                                                                       |
| `e_fog_hint`                        | string (hex)    | The encrypted fog hint for this TXO.                                                                                                                                                                                                                                                                                                                                                                                           |
| `subaddress_index`                  | string (uint64) | The assigned subaddress index for this TXO with respect to its received account.                                                                                                                                                                                                                                                                                                                                               |
| `key_image` (only on pending/spent) | string (hex)    | A fingerprint of the TXO derived from your private spend key materials, required to spend a TXO                                                                                                                                                                                                                                                                                                                                |
| `confirmation`                      | string (hex)    | A confirmation that the sender of the TXO can provide to validate that they participated in the construction of this TXO.                                                                                                                                                                                                                                                                                                      |
| `shared_secret`                     | string (hex)    | The proto-encoded shared secret used to decode the masked amount in the TXO                                                                                                                                                                                                                                                                                                                                                    |
| `memo`                              | Memo            | The decrypted memo that is a part of this txo, if available.                                                                                                                                                                                                                                                                                                                                                                   |

### Example <a href="#object_method" id="object_method"></a>

#### Received and Spent TXO

```
{
  "object": "txo",
  "txo_id": "14ad2f88...",
  "value_pmob": "8500000000000",
  "received_block_index": "14152",
  "spent_block_index": "20982",
  "is_spent_recovered": false,
  "received_account_id": "1916a9b3...",
  "minted_account_id": null,
  "account_status_map": {
    "1916a9b3...": {
      "txo_status": "spent",
      "txo_type": "received"
    }
  },
  "target_key": "6d6f6f6e...",
  "public_key": "6f20776f...",
  "e_fog_hint": "726c6421...",
  "subaddress_index": "20",
  "assigned_subaddress": "7BeDc5jpZ...",
  "key_image": "6d6f6269...",
  "confirmation": "23fd34a...",
  "shared_secret": "0a20eaa9...",
  "memo": {
    "AuthenticatedSender": {
      "sender_address_hash": "9b8e8d98...",
      "payment_request_id": null,
      "payment_intent_id": null
    }
  }
}
```

#### TXO Spent Between Accounts in the Same Wallet

```
{
  "object": "txo",
  "txo_id": "84f3023...",
  "value_pmob": "200",
  "received_block_index": null,
  "spent_block_index": null,
  "is_spent_recovered": false,
  "received_account_id": "36fdf8...",
  "minted_account_id": "a4db032...",
  "account_status_map": {
    "36fdf8...": {
      "txo_status": "unspent",
      "txo_type": "received"
    },
    "a4db03...": {
      "txo_status": "secreted",
      "txo_type": "minted"
    }
  },
  "target_key": "0a2076...",
  "public_key": "0a20e6...",
  "e_fog_hint": "0a5472...",
  "subaddress_index": null,
  "assigned_subaddress": null,
  "key_image": null,
  "confirmation": "0a2044...",
  "shared_secret": "0a20eaa9..."
}
```

## View Only Transaction Output ViewOnlyTXO

a minimal txo entity useful for view-only-accounts

### Attributes <a href="#object_method" id="object_method"></a>

| _Name_                     | _Type_                             | _Description_                                                                                                                |
| -------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `object`                   | string, value is "view\_only\_txo" | String representing the object's type. Objects of the same type share the same value.                                        |
| `public_key`               | string (hex)                       | The public key for this TXO, can be used as an identifier to find the TXO in the ledger.                                     |
| `value_pmob`               | string (uint64)                    | Available pico MOB for this account at the current `account_block_height`. If the account is syncing, this value may change. |
| `view_only_account_id_hex` | string                             | The local ID for view only account that has the private view key capable of decrypting this txo.                             |
| `spent`                    | string                             | Whether or not this txo has been manually marked as spent.                                                                   |
| `txo_id_hex`               | string                             | A synthetic ID created from properties of the TXO. This will be the same for a given TXO across systems.                     |

### Example <a href="#object_method" id="object_method"></a>
