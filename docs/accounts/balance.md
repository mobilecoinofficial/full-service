---
description: 账户余额，还包含了用来正确解读余额的同步状态信息。
---

# 账户余额

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "balance" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `network_block_index` | 字符串，内容为 64 位无符号整数 | MobileCoin 分布式账簿的区块高度，当 `local_block_index` 的值和 `network_block_index` 相等时说明本地记录已完成同步。 |
| `local_block_index` |字符串，内容为 64 位无符号整数 | 本地已同步的区块高度。`account_block_index` 最高只会同步到 `local_block_index` 的高度。 |
| `account_block_index` |字符串，内容为 64 位无符号整数 | 已扫描的关于当前账户的本地区块高度，永远不会大于 `local_block_index`，当完全同步时，将等于 `network_block_index`（或区块链上关于当前账户的最大区块高度）。 |
| `is_synced` | 布尔型 | 标识本地数据库是否已经和区块链完全同步。在账户同步还在进行时，账户余额可能不会反映最新的变化。 |
| `unspent_pmob` |字符串，内容为 64 位无符号整数 | 在当前 `account_block_index` 时当前账户内未使用的 Pico MOB。在账户完成同步后本数值可能会发生变化。 |
| `pending_pmob` |字符串，内容为 64 位无符号整数 | 待处理的发送中的 Pico MOB。在账簿处理完成后，待处理的数值会相应地减少。 |
| `spent_pmob` |字符串，内容为 64 位无符号整数 | 已花费的 Pico MOB。这是在钱包内所支出的所有金额的总和。 |
| `secreted_pmob` |字符串，内容为 64 位无符号整数 | 已铸的 Pico MOB。这是在钱包内生成的可以被消费的金额的总和。 |
| `orphaned_pmob` |字符串，内容为 64 位无符号整数 | 孤立的 Pico MOB。孤立的金额在对应的子地址索引被恢复之前无法被使用。 |

## 示例


```text
{
  "account_block_index": "152003",
  "is_synced": false,
  "local_block_index": "152918",
  "network_block_index": "152918",
  "object": "balance",
  "orphaned_pmob": "0",
  "pending_pmob": "0",
  "secreted_pmob": "0",
  "spent_pmob": "0",
  "unspent_pmob": "110000000000000000"
}
```

## 方法

### `get_balance_for_account`

获取指定账户的当前余额。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要查询余额的账户。 | 指定的账户必须存在在钱包中。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_balance_for_account",
  "params": {
     "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_balance_for_account",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152918",
      "local_block_index": "152918",
      "account_block_index": "152003",
      "is_synced": false,
      "unspent_pmob": "110000000000000000",
      "pending_pmob": "0",
      "spent_pmob": "0",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_balance_for_address`

获取指定地址的当前余额。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `address` | 要查询余额的账户地址。 | 地址必须已分配给钱包内的账户。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_balance_for_address",
  "params": {
     "address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
  },
  "jsonrpc": "2.0",
  "api_version": "2",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_balance_for_address",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152961",
      "local_block_index": "152961",
      "account_block_index": "152961",
      "is_synced": true,
      "unspent_pmob": "11881402222024",
      "pending_pmob": "0",
      "spent_pmob": "84493835554166",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  "api_version": "2"
}
```
{% endtab %}
{% endtabs %}

