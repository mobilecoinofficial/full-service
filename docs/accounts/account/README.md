---
description: >-
  An account in the wallet. An account is associated with one AccountKey,
  containing a View keypair and a Spend keypair.
---

# Account

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "account" | String representing the object's type. Objects of the same type share the same value. |
| `account_id` | string | The unique identifier for the account. |
| `name` | string | The display name for the account. |
| `main_address` | string | The b58 address code for the account's main address. The main address is determined by the seed subaddress. It is not assigned to a single recipient and should be considered a free-for-all address. |
| `next_subaddress_index` | string \(uint64\) | This index represents the next subaddress to be assigned as an address. This is useful information in case the account is imported elsewhere. |
| `first_block_index` | string \(uint64\) | Index of the first block when this account may have received funds. Defaults to 0 if not provided on account import |
| `next_block_index` | string \(uint64\) | Index of the next block this account needs to sync. |
| `fog_enabled` | boolean | A flag that indicates whether or not this account has a fog address. |
| `recovery_mode` | boolean | A flag that indicates this imported account is attempting to un-orphan found TXOs. It is recommended to move all MOB to another account after recovery if the user is unsure of the assigned addresses. |

## Example

```text
{
  "object": "account",
  "account_id": "gdc3fd37f1903aec5a12b12a580eb837e14f87e5936f92a0af4794219f00691d",
  "name": "I love MobileCoin",
  "main_address": "8vbEtknX7zNtmN5epTYU95do3fDfsmecDu9kUbW66XGkKBX87n8AyqiiH9CMrueo5H7yiBEPXPoQHhEBLFHZJLcB2g7DZJ3tUZ9ArVgBu3a",
  "next_subaddress_index": "3",
  "first_block_index": "3500",
  "recovery_mode": false
}
```

# ViewOnlyAccount
A view-only-account in the wallet. An view-only-account is associated with one ViewPrivateKey. It can decode txos but it can not decode key images or create txos.

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "view_only_account" | String representing the object's type. Objects of the same type share the same value. |
| `account_id` | string | The unique identifier for the account. |
| `name` | string | The display name for the account. |
| `first_block_index` | string \(uint64\) | Index of the first block when this account may have received funds. Defaults to 0 if not provided on account import. |
| `next_block_index` | string \(uint64\) | Index of the next block this account needs to sync. |

## Example

```text
{
  "object": "view-only-account",
  "account_id": "gdc3fd37f1903aec5a12b12a580eb837e14f87e5936f92a0af4794219f00691d",
  "name": "I love MobileCoin",
  "first_block_index": "0",
  "next_block_index": "3500",
}
```

