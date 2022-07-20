---
description: >-
  A MobileCoin Transaction consists of inputs which are spent in order to mint
  new outputs for the recipient.
---

# Transaction

Due to the privacy properties of the MobileCoin ledger, transactions are ephemeral. Once they have been created, they only exist until they are validated, and then only the outputs are written to the ledger. For this reason, the Full Service wallet stores transactions in the `transaction_log` table in order to preserve transaction history.

## Attributes

### Transaction Log

| Name | Type | Description |
| :--- | :--- | :--- |
| object | string | String representing the object's type. Objects of the same type share the same value |
| `transaction_log_id` | string | Unique identifier for the transaction log. This value is not associated to the ledger |
| `direction` | string | A string that identifies if this transaction log was sent or received. Valid values are "sent" or "received" |
| `is_sent_recovered` | bool \(optional\) | Flag that indicates if the sent transaction log was recovered from the ledger. This is "null" for received transaction logs. If true, some information may not be available on the transaction log and its txos without user input. |
| `account_id` | string | Unique identifier for the assigned associated account. If the transaction is outgoing, this account is from whence the txo came. If received, this is the receiving account. |
| `input_txos` | \[TxoAbbrev\] | A list of txos which were inputs to this transaction |
| `output_txos` | \[TxoAbbrev\] | A list of txos which were outputs to this transaction |
| `change_txos` | \[TxoAbbrev\] | A list of txos which were change in this transaction |
| `assigned_address_id` | string \(optional\) | Unique identifier for the assigned associated account. Only available if direction is "received" |
| `value_pmob` | string \(optional\) | Value in pico MOB associated to this transaction log |
| `fee_pmob` | string \(optional\) | Fee in pico MOB associated to this transaction log. Only on outgoing transaction log . Only available if direction is "sent" |
| `submitted_block_index` | string \(optional\) | The block index of the highest block on the network at the time the transaction was submitted |
| `finalized_block_index` | string \(optional\) | The scanned block index in which this transaction occurred |
| `status` | string | String representing the transaction log status. On `send`, valid statuses are `built`, `pending`, `succeeded`, `failed`. On `received`, the status is `succeeded`. |
| `sent_time` | string \(optional\) | Time at which sent transaction log was created. Only available if direction is `sent`. This value is `null` if received or if the sent transactions were recovered from the ledger`is_sent_recovered = true` |
| `comment` | string | An arbitrary string attached to the object |
| `failure_code` | i32 \(optional\) | Code representing the cause of `failed` status |
| `failure_message` | string \(optional\) | Human parsable explanation of `failed` status |

### TxoAbbrev

| Name | Type | Description |
| :--- | :--- | :--- |
| `txo_id_hex` | string | Unique identifier for the txo |
| `recipient_address_id` | string | Unique identifier for the recipient associated account. Blank unless direction is `sent` |
| `value_pmob` | string | Available pico MOB for this Txo. If the account is syncing, this value may change |
