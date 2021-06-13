---
description: 钱包状态提供了钱包内容的概览。
---

# 钱包状态

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "wallet\_status" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `network_block_index` | 字符串，内容为 64 位无符号整数 | MobileCoin 分布式账簿的区块高度，当 `local_block_index` 的值和 `network_block_index` 相等时说明本地记录已完成同步。 |
| `local_block_index` | 字符串，内容为 64 位无符号整数 | 本地已同步的区块高度。每个账户的 `account_block_index` 最高只会同步到 `local_block_index` 的高度。 |
| `is_synced_all` | 布尔型 | 标识本地数据库的**全部**账户是否已经和区块链完全同步。在账户同步还在进行时，账户余额可能不会反映最新的变化。 |
| `total_unspent_pmob` | 字符串，内容为 64 位无符号整数 | 当前 `account_block_index` 位置时所有账户的未花费 Pico MOB 的总和。在账户完成同步后本数值可能会发生变化。 |
| `total_pending_pmob` |字符串，内容为 64 位无符号整数 | 全部账户的待处理的发送中的 Pico MOB。在账簿处理完成后，待处理的数值会相应地减少。`available_pmob` 会反映该变化。 |
| `total_spent_pmob` |字符串，内容为 64 位无符号整数 | 已花费的 Pico MOB。这是在钱包内所支出的所有金额的总和。 |
| `total_secreted_pmob` |字符串，内容为 64 位无符号整数 | 已铸的 Pico MOB。这是在钱包内生成的可以被消费的金额的总和。 |
| `total_orphaned_pmob` |字符串，内容为 64 位无符号整数 | 孤立的 Pico MOB。孤立的金额在对应的子地址索引被恢复之前无法被使用。 |
| `account_ids` | 列表 | 以导入钱包的顺序排列的全部 `account_id` 列表。 |
| `account_map` | 散列表 | 一个规范化的从 `account_id` 到账户对象到散列映射。 |

## 示例

```text
{
"wallet_status": {
  "account_ids": [
    "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
    "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470"
  ],
  "account_map": {
    "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470": {
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "key_derivation_version:": "2",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "name": "Bob",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "key_derivation_version:": "2",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "is_synced_all": false,
  "local_block_index": "152918",
  "network_block_index": "152918",
  "object": "wallet_status",
  "total_orphaned_pmob": "0",
  "total_pending_pmob": "70148220000000000",
  "total_secreted_pmob": "0",
  "total_spent_pmob": "0",
  "total_unspent_pmob": "220588320000000000"
}
```

