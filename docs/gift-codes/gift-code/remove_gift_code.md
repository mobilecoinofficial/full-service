---
description: 从数据库中移除一个红包码。
---

# 移除红包码

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |

## 示例

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

