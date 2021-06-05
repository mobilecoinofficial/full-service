---
description: 验证一个地址是不是正确的 Base 58 编码。
---

# 验证地址

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `address` | 要验证的地址。 | 地址必须已分配给钱包内的账户。 |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "verify_address",
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
  "method": "verify_address",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

