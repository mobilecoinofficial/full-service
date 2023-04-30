---
description: The current network fee and total number of blocks.
---

# Network Status

## Attributes

| _Name_                 | _Type_               | _Description_                                                                                                                                    |
|------------------------|----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| `network_block_height` | string (string)      | The block count of MobileCoin's distributed ledger.                                                                                              |
| `local_block_height`   | string (string)      | The local block count downloaded from the ledger. The local database is synced when the `local_block_height` reaches the `network_block_height`. |
| `fees`                 | Map (string, string) | Default fee for each token required to send a transaction.                                                                                       |
| `block_version`        | string (optional)    | The current block version of MobileCoin's blockchain.                                                                                            |
