---
description: >-
  A MobileCoin Transaction consists of inputs which are spent in order to mint
  new outputs for the recipient.
---

# Transaction

Due to the privacy properties of the MobileCoin ledger, transactions are ephemeral. Once they have been created, they
only exist until they are validated, and then only the outputs are written to the ledger. For this reason, the Full
Service wallet stores transactions in the `transaction_log` table in order to preserve transaction history.

## Attributes

### Transaction Proposal

| Name                    | Type         | Description                                                                                   |
|-------------------------|--------------|-----------------------------------------------------------------------------------------------|
| `input_txos`            | \[InputTxo]  | The collection of txos used as inputs                                                         |
| `payload_txos`          | \[OutputTxo] | The collection of txos used as payload outputs                                                |
| `change_txos`           | \[OutputTxo] | The collection of txos used as change outputs                                                 |
| `fee`                   | string       | Fee for this transaction                                                                      |
| `fee_token_id`          | string       | TokenId of the fee for this transaction                                                       |
| `tombstone_block_index` | string       | The tombstone block index of this transaction                                                 |
| `tx_proto`              | string       | The protobuff encoded data of the transaction that can be submitted to the mobilecoin network |

### InputTxo

| Name           | Type   | Description                   |
|----------------|--------|-------------------------------|
| `tx_out_proto` | string | Unique identifier for the txo |
| `value`        | string | The value of this txo         |
| `token_id`     | string | The tokenId of this txo       |
| `key_image`    | string | The key image of this txo     |

### OutputTxo

| Name                           | Type   | Description                                                                         |
|--------------------------------|--------|-------------------------------------------------------------------------------------|
| `tx_out_proto`                 | string | Unique identifier for the txo                                                       |
| `value`                        | string | The value of this txo                                                               |
| `token_id`                     | string | The tokenId of this txo                                                             |
| `recipient_public_address_b58` | string | The recipient that this txo belongs to                                              |
| `confirmation_number`          | string | The confirmation number of the txo that can be used to validate it by the recipient |
