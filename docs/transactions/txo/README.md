---
description: TXO 是“交易输出（Transaction Output）”的缩写。MobileCoin 是一个基于 UTXO （未花交易输出）模型的账簿。
---

# 交易输出 TXO

在构造交易时，钱包会选择未花的 TXO 并通过一些密码学操作将它们在账簿内标记为“已花”。在此之后为收款方铸（Mint）新的 TXO。

## 属性 <a id="object_method"></a>

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "txo" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `value_pmob` | 字符串，内容为 64 位无符号整数 | 当前账户在当前 `account_block_index` 位置的可使用的 pico MOB。 在账户完成同步后本数值可能会发生变化。 |
| `received_block_index` | 字符串，内容为 64 位无符号整数 | TXO 被接收的区块索引。 |
| `spent_block_index` | 字符串，内容为 64 位无符号整数 | TXO 被使用（消费）的区块索引。 |
| `is_spent_recovered` | 布尔型 | 指示 `spent_block_index` 是否被从账簿恢复。当 TXO 未被花费时未 `null`。 为 `true` 时，在没有用户输入的情况下一些信息（比如确认代码）可能不可用。|
| `received_account_id` | 字符串 | 接收本 TXO 的账户的 `account_id`。该账户拥有使用本 TXO 的权力。 |
| `minted_account_id` | 字符串 | 本 TXO 被铸（Mint）的账户的 `account_id`。 |
| `account_status_map` | 散列表 | 一个规范化的从 `account_id` 到账户对象的散列映射。键值包括 "type" 和 "status"。 |
| `txo_type` | 由字符串表示的枚举类型 | 对于当前账户，一个 TXO 可能为 "minted" 或 "received"。 |
| `txo_status` | 由字符串表示的枚举类型  | 对于当前账户, 一个 TXO 可能为 "unspent", "pending", "spent", "secreted" 或 "orphaned"。 通过指定地址接收到的 TXO 的生命周期为："unspent" -&gt; "pending" -&gt; "spent"。 对于花出的 TXO，我们并不能监控其接收状态，因此记为 "secreted"。如果一个 TXO 是通过未分配的地址接收的（可能是由于账户信息恢复不完整导致），在对应的子地址被重新计算之前，TXO 的状态将为 "orphaned"，在这种情况下，您可以通过手动途径找回该子地址，或是恢复整个账户。 |
| `target_key` | 字符串 \(hex\) | 本 TXO 的密码学键值。 A cryptographic key for this TXO. |
| `public_key` | 字符串 \(hex\) | 本 TXO 的公钥，可以作为本 TXO 在账簿上的标识符。|
| `e_fog_hint` | 字符串 \(hex\) | 本 TXO 的加密的雾信息（Fog hint）。  |
| `subaddress_index` | 字符串，内容为 64 位无符号整数 | 为本 TXO 指定的接收方的子地址索引。|
| `assigned_address` | 字符串，内容为 64 位无符号整数 | 为本 TXO 指定的发送方的地址。|
| `key_image` \(只会出现在发送中/已花费 TXO 中\) | 字符串 \(hex\) | 从可花（Spend）私钥计算出的 TXO 指纹，是发送 TXO 的必要信息。|
| `confirmation` | 字符串 \(hex\) | 发送者参与了 TXO 构造的证明。|
| `offset_count` | 整型 | 请求的分页偏移量。请求将只会返回当前对象之后的内容（不包括当前对象）。 |

## 示例 <a id="object_method"></a>

### 接收的并已花费的 TXO

```text
{
  "object": "txo",
  "txo_id": "14ad2f88...",
  "value_pmob": "8500000000000",
  "received_block_index": "14152",
  "spent_block_index": "20982",
  "is_spent_recovered": false,
  "received_account_id": "1916a9b3...",
  "minted_account_id": null,
  "account_status_map": {
    "1916a9b3...": {
      "txo_status": "spent",
      "txo_type": "received"
    }
  },
  "target_key": "6d6f6f6e...",
  "public_key": "6f20776f...",
  "e_fog_hint": "726c6421...",
  "subaddress_index": "20",
  "assigned_subaddress": "7BeDc5jpZ...",
  "key_image": "6d6f6269...",
  "confirmation": "23fd34a...",
  "offset_count": 284
}
```

### 在同一个钱包内的两个账户间花费的 TXO

```text
{
  "object": "txo",
  "txo_id": "84f3023...",
  "value_pmob": "200",
  "received_block_index": null,
  "spent_block_index": null,
  "is_spent_recovered": false,
  "received_account_id": "36fdf8...",
  "minted_account_id": "a4db032...",
  "account_status_map": {
    "36fdf8...": {
      "txo_status": "unspent",
      "txo_type": "received"
    },
    "a4db03...": {
      "txo_status": "secreted",
      "txo_type": "minted"
    }
  },
  "target_key": "0a2076...",
  "public_key": "0a20e6...",
  "e_fog_hint": "0a5472...",
  "subaddress_index": null,
  "assigned_subaddress": null,
  "key_image": null,
  "confirmation": "0a2044...",
  "offset_count": 501
}
```

