---
description: >-
  An account in the wallet. An account is associated with one AccountKey,
  containing a View keypair and a Spend keypair.
---

# Account

A view-only-account in the wallet. An view-only-account is associated with one ViewPrivateKey. It can decode txos but it can not decode key images or create txos.

## Attributes

| Name                      | Type                                   | Description                                                                                                          |
| ------------------------- | -------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `object`                  | string, value is "view\_only\_account" | String representing the object's type. Objects of the same type share the same value.                                |
| `account_id`              | string                                 | The unique identifier for the account.                                                                               |
| `name`                    | string                                 | The display name for the account.                                                                                    |
| `first_block_index`       | string (uint64)                        | Index of the first block when this account may have received funds. Defaults to 0 if not provided on account import. |
| `next_block_index`        | string (uint64)                        | Index of the next block this account needs to sync.                                                                  |
| `main_subaddress_index`   | string (uint64)                        |                                                                                                                      |
| `change_subaddress_index` | string (uint64)                        |                                                                                                                      |
| `next_subaddress_index`   | string (uint64)                        |                                                                                                                      |

## Example

```
{
  "object": "view-only-account",
  "account_id": "gdc3fd37f1903aec5a12b12a580eb837e14f87e5936f92a0af4794219f00691d",
  "name": "I love MobileCoin",
  "first_block_index": "0",
  "next_block_index": "3500",
  "main_subaddress_index": "0",
  "change_subaddress_index": "1",
  "next_subaddress_index": "2"
}
```
