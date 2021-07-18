---
description: 获取交易日志中 TXO 对象的 JSON 表示。
---

# 获取交易对象 

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `transaction_log_id` | 要查询的交易日志 ID。 | 交易日志必须存在在钱包中。 |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_transaction_object",
  "params": {
    "transaction_log_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_transaction_object",
  "result": {
    "transaction": ...
  }
}
```
{% endtab %}
{% endtabs %}

