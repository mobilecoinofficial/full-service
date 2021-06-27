---
description: 创建一个 Base 58 编码的可以发给别人的支付请求。
---

# 创建支付请求

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用来创建支付请求的账户。 | 账户必须存在于钱包内。 |
| `amount_pmob` | 要发送的 pMOB 数量。 | 类型为 64 位无符号整型。 |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `subaddress_index` | 用来创建支付请求的子地址索引。| 类型为 64 位有符号整型。|
| `memo` | 支付请求附带的信息。 |  |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "create_payment_request",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "amount_pmob": 42000000000000,
    "subaddress_index": 4,
    "memo": "家庭聚餐"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "create_payment_request",
  "result": {
    "payment_request_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

