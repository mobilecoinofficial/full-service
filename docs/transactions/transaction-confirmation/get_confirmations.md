---
description: 由钱包构建的 TXO 会包括一个确认编码，可以由发送方分享给接收方，接收方可以据此确认 TXO 和发送方的关联。
---

# 获取确认编码

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `transaction_log_id` | 需要获取确认编码的交易日志 ID。| 该交易日志必须存在在钱包内。|

## 示例

当调用 `get_confirmations` 时，系统只会返回对应的确认编码（而不会包括其他的交易细节）。

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_confirmations",
  "params": {
    "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_confirmations",
  "result": {
    "confirmations": [
      {
        "object": "confirmation",
        "txo_id": "9e0de29bfee9a391e520a0b9411a91f094a454ebc70122bdc0e36889ab59d466",
        "txo_index": "458865",
        "confirmation": "0a20faca10509c32845041e49e009ddc4e35b61e7982a11aced50493b4b8aaab7a1f"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

