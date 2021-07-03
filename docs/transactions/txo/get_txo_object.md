---
description: 获取账簿中的 TXO 对象的 JSON 表示。
---

# 获取指定地址的全部 TXO

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `address` | 要查询的地址。 |  |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_all_txos_for_address",
  "params": {
    "address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_txo_object",
  "result": {
    "txo": ...
  }
}
```
{% endtab %}
{% endtabs %}

