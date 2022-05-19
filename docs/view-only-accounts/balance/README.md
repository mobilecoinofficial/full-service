---
description: >-
  The balance of an account, which includes additional information about the
  syncing status needed to interpret the balance correctly.
---

# View Only Balance
The balance for a view-only-account.

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "balance" | String representing the object's type. Objects of the same type share the same value. |
| `network_block_height` | string \(uint64\) | The block count of MobileCoin's distributed ledger. |
| `local_block_height` | string \(uint64\) | The local block count downloaded from the ledger. The local database is synced when the `local_block_height` reaches the `network_block_height`. The `account_block_height` can only sync up to `local_block_height`. | 
| `account_block_height` | string \(uint64\) | The scanned local block count for this account. This value will never be greater than `local_block_height`. At fully synced, it will match `network_block_height`.
| `is_synced` | boolean | Whether the account is synced with the `network_block_height`. Balances may not appear correct if the account is still syncing. |
| `balance` | string \(uint64\) | total pico MOB for this account minus the pico MOB marked as spent for this account |

## Example

```text
{
  "object": "balance",
  "balance": "10000000000000",
  "network_block_height": "468847",
  "local_block_height": "468847",
  "account_block_height": "468847",
  "is_synced": true
}
```
