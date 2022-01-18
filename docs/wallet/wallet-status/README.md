---
description: The Wallet Status provides a quick overview of the contents of the wallet.
---

# Wallet Status

## Attributes

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | String, value is "wallet\_status" | String representing the object's type. Objects of the same type share the same value. |
| `network_block_height` | String \(uint64\) | The block count of MobileCoin's distributed ledger. |
| `local_block_height` | String \(uint64\) | The local block count downloaded from the ledger. The local database is synced when the `local_block_height` reaches the `network_block_height`. The account_block_height can only sync up to `local_block_height`. |
| `is_synced_all` | Boolean | Whether ALL accounts are synced with the `network_block_height`. Balances may not appear correct if any account is still syncing. |
| `total_unspent_pmob` | String \(uint64\) | Unspent pico mob for ALL accounts at the `account_block_height`. If the account is syncing, this value may change. |
| `total_pending_pmob` | String \(uint64\) | Pending outgoing pico mob from ALL accounts. Pending pico mobs will clear once the ledger processes the outgoing TXO. The `available_pmob` will reflect the change. |
| `total_spent_pmob` | String \(uint64\) | Spent pico MOB. This is the sum of all the TXOs in the wallet which have been spent. |
| `total_secreted_pmob` | String \(uint64\) | Secreted \(minted\) pico MOB. This is the sum of all the TXOs which have been created in the wallet for outgoing transactions. |
| `total_orphaned_pmob` | String \(uint64\) | Orphaned pico MOB. The orphaned value represents the TXOs which were view-key matched, but which can not be spent until their subaddress index is recovered. |
| `account_ids` | List | A list of all `account_ids` imported into the wallet in order of import. |
| `account_map` | Hash map | A normalized hash mapping `account_id` to account objects. |

## ​Example

```text
{
"wallet_status": {
  "account_ids": [
    "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
    "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470"
  ],
  "account_map": {
    "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470": {
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "key_derivation_version:": "2",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "name": "Bob",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "key_derivation_version:": "2",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "is_synced_all": false,
  "local_block_height": "152918",
  "network_block_height": "152918",
  "object": "wallet_status",
  "total_orphaned_pmob": "0",
  "total_pending_pmob": "70148220000000000",
  "total_secreted_pmob": "0",
  "total_spent_pmob": "0",
  "total_unspent_pmob": "220588320000000000"
}
```

