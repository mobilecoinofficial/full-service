---
description: 查看一个红包码的状态，可能为待处理（Pending），可用（Available）或已收取（Claimed）。
---

# 查看红包码状态

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `gift_code_b58` | Base 58 编码的红包码。 | 必须为有效的 Base 58 编码的红包码。 |

## 示例

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

