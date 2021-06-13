---
description: 收取红包到钱包内的指定账户。
---

# 收取红包

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |
| `account_id` | 要收取红包的账户 ID | 指定的账户必须存在在钱包中。 |

## 示例

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

