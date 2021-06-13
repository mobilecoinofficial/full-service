---
description: 从数据库中读取一个红包的加密口令，价值以及留言信息。
---

# 获取红包码

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码 | 必须为有效的 Base 58 编码的红包码。 |

## 示例

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

