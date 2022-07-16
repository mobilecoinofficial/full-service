---
description: >-
  The balance for a token, separated by status
---

# Balance

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `unverified` | string \(uint64\) | Unverified value for this account at the current `account_block_height`. Unverified means it has a known subaddress but not a known key image \(In the case of view only accounts\) If the account is syncing, this value may change. |
| `unspent` | string \(uint64\) | Unspent value for this account at the current `account_block_height`. If the account is syncing, this value may change. |
| `pending` | string \(uint64\) | Pending, out-going value. The pending value will clear once the ledger processes the outgoing TXOs. The `pending` will reflect the change. |
| `spent` | string \(uint64\) | Spent value. This is the sum of all the TXOs in the wallet which have been spent. |
| `secreted` | string \(uint64\) | Secreted \(minted\) value. This is the sum of all the TXOs which have been created in the wallet for outgoing transactions. |
| `orphaned` | string \(uint64\) | Orphaned value. The orphaned value represents the TXOs which were view-key matched, but which can not be spent until their subaddress index is recovered. |

## Example

```text
{
  "unverified": "0",
  "unspent": "110000000000000000",
  "pending": "0",
  "spent": "0",
  "secreted": "0",
  "orphaned": "0"
}
```