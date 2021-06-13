---
description: 红包是一个只包含一个 TXO 的一次性的账户。在放入资金后，可以通过不同的途径（比如二维码）分享给接收方去接收红包内的资金。
---

# 红包码

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "gift_code" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `gift_code` | 字符串 | 用来分享的由 Base 58 编码的红包码。 |
| `entropy` | 字符串 | 临时红包账户的加密口令。 |
| `value_pmob` | 字符串 | 临时红包账户内包含的 MOB 数量。 |
| `memo` | 字符串 | 红包附带的备注消息。 |


## 示例

```text
{
  "object": "gift_code",
  "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
  "entropy": "41e1e794f8a2f7227fa8b5cd936f115b8799da712984c85f499e03bca43cba9c",
  "value_pmob": "60000000000",
  "memo": "新年快乐！",
  "account_id": "050d8d97aaf31c70d63c6aed828c11d3fb16b56b44910659b6724621047b81f9",
  "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
}
```

##  方法

### `build_gift_code`

在一个 tx_proposal 内生成一个可以放入资金并提交至账簿的红包码。

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

### `submit_gift_code`

向账簿提交红包的 tx_proposal 并在账簿接收后将红包码加入钱包的数据库 \(wallet_db\)。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |
| `from_account_id` | 用来提交 tx_proposal 的账户 ID | 指定的账户必须存在在钱包中。 |
| `tx_proposal` | 要提交的交易提案 \(tx_proposal\) | 由 `build_gift_code` 创建。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "submit_gift_code",
  "params": {
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
    "tx_proposal": '$(cat test-tx-proposal.json)',
    "from_account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "submit_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "生日快乐！",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_gift_code`

从数据库中读取一个红包的加密口令，价值以及留言信息。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_gift_code",
  "params": {
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "生日快乐！",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `get_all_gift_codes`

获取当前数据库中的全部红包码。

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_all_gift_codes",
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_all_gift_codes",
  "result": {
    "gift_codes": [
      {
        "object": "gift_code",
        "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
        "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
        "value_pmob": "80000000000",
        "memo": "新年快乐！",
        "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
        "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
      },
      {
        "object": "gift_code",
        "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY",
        "entropy": "14aa16d9d4000628c82826d9c43bbc17414f8677e74882bf21e44db75d4c2b87",
        "value_pmob": "20000000000",
        "memo": "生日快乐！",
        "account_id": "dba3d3b99fe9ce6bc666490b8176be91ace0f4166853b0327ea39928640ea840",
        "txo_id": "ab917ed9e69fa97bd9422452b1a2f615c2405301b220f7a81eb091f75eba3f54"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `check_gift_code_status`

查看一个红包码的状态，可能为待处理（Pending），可用（Available）或已收取（Claimed）。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码。 | 必须为有效的 Base 58 编码的红包码。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "check_gift_code_status",
  "params": {
    "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeAvailable",
    "gift_code_value": 100000000,
    "gift_code_memo": "生日快乐！"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}

{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeSubmittedPending",
    "gift_code_value": null
    "gift_code_memo": "",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

#### 输出

* `GiftCodeSubmittedPending` - 红包正在处理中，还没有显示在账簿上。
* `GiftCodeAvailable` - 红包可以被收取。
* `GiftCodeClaimed` - 红包以及被收取。

### `claim_gift_code`

收取红包到钱包内的指定账户。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |
| `account_id` | 要收取红包的账户 ID | 指定的账户必须存在在钱包中。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "claim_gift_code",
  "params": {
    "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
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
  "method": "claim_gift_code",
  "result": {
    "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `remove_gift_code`

从数据库中移除一个红包码。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "remove_gift_code",
  "params": {
    "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "remove_gift_code",
  "result": {
    "removed": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

