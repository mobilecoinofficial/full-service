---
description: 在构建了交易提案（`tx_proposal`）后，您可以生成该交易对应的收据并提供给收款方，这样对方就可以通过收据来查询交易的状态。
---

# 创建收据

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `tx_proposal` |  |  |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "create_receiver_receipts",
  "params": {
    "tx_proposal": '$(cat tx_proposal.json)',
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "create_receiver_receipts",
  "result": {
    "receiver_receipts": [
      {
        "object": "receiver_receipt",
        "public_key": "0a20d2118a065192f11e228e0fce39e90a878b5aa628b7613a4556c193461ebd4f67",
        "confirmation": "0a205e5ca2fa40f837d7aff6d37e9314329d21bad03d5fac2ec1fc844a09368c33e5",
        "tombstone_block": "154512",
        "amount": {
          "object": "amount",
          "commitment": "782c575ed7d893245d10d7dd49dcffc3515a7ed252bcade74e719a17d639092d",
          "masked_value": "12052895925511073331"
        }
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

