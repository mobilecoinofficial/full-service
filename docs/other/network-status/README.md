---
description: The current network fee and total number of blocks.
---

# Network Status

## Attributes

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | string, value is "network_status" | String representing the object's type. Objects of the same type share the same value. |
| `network_block_height` | string \(uint64\) | The block count of MobileCoin's distributed ledger. |
| `local_block_height` | string \(uint64\) | The local block count downloaded from the ledger. The local database is synced when the `local_block_height` reaches the `network_block_height`. |
| `fee_pmob` | string \(optional\) | Default fee in pico MOB required to send a transaction. |
