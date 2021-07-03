---
description: MobileCoin 的交易包括了用来给收款方构造新的 TXO 的被消费的输入。
---

# 交易

基于 MobileCoin 的隐私属性，交易的存在时间很短。当一个交易被创建后，它在被验证后就会被销毁，只有交易输出会被写入到账簿上。因此，Full Service 把交易存储在 `transaction_log` 表中用来记录交易历史。

## 属性

### 交易日志

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

## Example

