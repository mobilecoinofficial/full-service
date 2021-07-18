---
description: MobileCoin 的交易包括了用来给收款方构造新的 TXO 的被消费的输入。
---

# 交易

基于 MobileCoin 的隐私属性，交易的存在时间很短。当一个交易被创建后，它在被验证后就会被销毁，只有交易输出会被写入到账簿上。因此，Full Service 把交易存储在 `transaction_log` 表中用来记录交易历史。

## 属性

### 交易日志

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为  "transaction\_log" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `transaction_log_id` | 整数 | 交易日志的唯一标识符。该标识符与账簿无关。 |
| `direction` | 字符串 | 标记交易日志的来源，为接收（"received"）或发送（"sent"）。 |
| `is_sent_recovered` | 布尔型 | 指示一个发送的交易是否被从账簿上恢复。如果一个交易是被钱包接收的，本字段为 null。当本字段为真时，在没有用户输入的情况下，一些信息在交易日志和该交易的 TXOs 上可能会不可用，而且字段 `receipient_address_id`，`fee` 和 `sent_time` 在没有用户输入的情况下会为 null。 |
| `account_id` | 字符串 | 与交易关联的账户 ID。如果交易是发出的，那么本字段为 TXO 的来源；如果交易是接收的，那么本字段为接收交易的账户。 |
| `recipient_address_id` | 字符串 | 与交易关联的接收方账户，只有当该交易为发送的交易时有效。 |
| `assigned_address_id` | 字符串 | 与交易关联的指定账户，只有当该交易为接收的交易时有效。 |
| `value_pmob` | 字符串 \(uint64\) | 交易日志对应的交易价值。单位为 Pico Mob。 |
| `fee_pmob` | 字符串 \(uint64\) | 交易日志对应的交易手续费，只有当该交易为发送的交易时有效。单位为 Pico Mob。|
| `submitted_block_index` | 字符串 \(uint64\) | 当交易被提交时网络上的最高的区块高度。 |
| `finalized_block_index` | 字符串 \(uint64\) | 交易被记录在区块网络上的区块索引。 |
| `status` | 字符串 | 表示交易状态的字符串。当交易是发出的时，全部有效的状态为： "built"， "pending"， "succeeded" 或 "failed"。当交易是接收的时，状态为 "succeeded"。 |
| `input_txo_ids` | TxoAbbrev 列表 | 交易输入的 TXO ID 列表。 |
| `output_txo_ids` | TxoAbbrev 列表 | 交易输出的 TXO ID 列表。  |
| `change_txo_ids` | TxoAbbrev 列表 | 在该交易中产生的找零 TXO ID 列表。 |
| `sent_time` | 时间戳 | 交易日志被创建的时间戳。只有交易是发送的时有效。当交易为接收的，或是交易被从账簿上恢复时（`is_sent_recovered = true`）为 null。|
| `comment` | 字符串 | 对象附带的任意字符串。 |
| `failure_code` | 整数 | "failed" 状态的错误码。 |
| `failure_message` | 字符串 | 人类可读的 "failed" 状态解释。 |

### TxoAbbrev

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `txo_id_hex` | 字符串 | TXO 的唯一标识符。|
| `recipient_address_id` | 字符串 | 接收方的账户。只有当交易为发送时才生效。|
| `value_pmob` | 字符串 | 本 TXO 的价值，单位为 Pico Mob。如果账户尚未完全同步，本字段可能会发生变化。|

## Example

