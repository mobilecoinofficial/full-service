---
description: >-
  A TXO is a "Transaction Output." MobileCoin is a ledger built on the "Unspent
  Transaction Output" model (UTXO).
---

# Transaction Output TXO

In order to construct a transaction, the wallet will select "Unspent Transaction Outputs" and perform a cryptographic operation to mark them as "spent" in the ledger. Then, it will mint new TXOs for the recipient.

## Attributes <a id="object_method"></a>

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | string, value is "txo" | String representing the object's type. Objects of the same type share the same value. |
| `value_pmob` | string \(uint64\) | Available pico MOB for this account at the current `account_block_height`. If the account is syncing, this value may change. |
| `received_block_index` | string \(uint64\) | Block index in which the TXO was received by an account. |
| `spent_block_index` | string \(uint64\) | Block index in which the TXO was spent by an account. |
| `is_spent_recovered` | boolean | Flag that indicates if the `spent_block_index` was recovered from the ledger. This value is null if the TXO is unspent. If true, some information may not be available on the TXO without user input. If true, the confirmation number will be null without user input. |
| `received_account_id` | string | The `account_id` for the account which has received this TXO. This account has spend authority. |
| `minted_account_i` | string | The `account_id` for the account which minted this TXO. |
| `account_status_map` | hash map | A normalized hash mapping account\_id to account objects. Keys include "type" and "status". |
| `txo_type` | string \(enum\) | With respect to this account, the TXO may be "minted" or "received". |
| `txo_status` | string \(enum\) | With respect to this account, the TXO may be "unspent", "pending", "spent", "secreted" or "orphaned". For received TXOs received as an assigned address, the lifecycle is "unspent" -&gt; "pending" -&gt; "spent". For outbound, minted TXOs, we cannot monitor its received lifecycle status with respect to the minting account, we note its status as "secreted". If a TXO is received at an address unassigned \(likely due to a recovered account or using the account on another client\), the TXO is considered "orphaned" until its address is calculated -- in this case, there are manual ways to discover the missing assigned address for orphaned TXOs or to recover an entire account. |
| `target_key` | string \(hex\) | A cryptographic key for this TXO. |
| `public_key` | string \(hex\) | The public key for this TXO, can be used as an identifier to find the TXO in the ledger. |
| `e_fog_hint` | string \(hex\) | The encrypted fog hint for this TXO. |
| `subaddress_index` | string \(uint64\) | The assigned subaddress index for this TXO with respect to its received account. |
| `assigned_address` | string \(uint64\) | The address corresponding to the subaddress index which was assigned as an intended sender for this TXO. |
| `key_image` \(only on pending/spent\) | string \(hex\) | A fingerprint of the TXO derived from your private spend key materials, required to spend a TXO |
| `confirmation` | string \(hex\) | A confirmation that the sender of the TXO can provide to validate that they participated in the construction of this TXO. |
| `offset` | integer | The value to offset pagination requests. Requests will exclude all list items up to and including this object. |
| `limit` | integer | The limit of returned results. |

## Example <a id="object_method"></a>

### Received and Spent TXO

```text
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
  "confirmation": "23fd34a..."
}
```

### TXO Spent Between Accounts in the Same Wallet

```text
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
  "confirmation": "0a2044..."
}
```

