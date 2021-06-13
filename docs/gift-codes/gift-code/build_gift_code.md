---
description: 在一个 tx_proposal 内生成一个可以放入资金并提交至账簿的红包码。
---

# 生成红包码

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 红包的发出账户。 | 指定的账户必须存在在钱包中。 |
| `value_pmob` | 红包内将包括的 MOB 的数量。  |  |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `input_txo_ids` | 作为交易输入的 MOB \(TXO\) ID。 | TXO IDs \(通过 `get_all_txos_for_account` 获取\) |
| `fee` | 随本交易提交的手续费。 | 默认为 `MINIMUM_FEE` = .01 MOB. |
| `tombstone_block` | 本交易的过期区块，当网络区块高于本值时，红包即失效。 | 默认为当前高度 \(`cur_height`\) + 50. |
| `max_spendable_value` | 要选择作为交易输入的单个 TXO 最大价值。 |  |
| `memo` | 给接收方的信息。 |  |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "build_gift_code",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "value_pmob": "42000000000000",
    "memo": "生日快乐！"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "build_gift_code",
  "result": {
    "tx_proposal": "...",
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

