---
description: 交易收据包含了确认编码，收款方可以通过收据来查询交易状态。
---

# 交易收据

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "receiver\_receipt"  | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `public_key` | 字符串 | 16 进制编码的 TXO 的公钥。 |
| `tombstone_block` | 字符串 | TXO 的有效期（在区块链高度大于此指定区块后当前 TXO 将会被共识系统拒绝）。|
| `confirmation` | 字符串 | 16 进制编码的确认信息，可以用来验证交易的另一方参与了该 TXO 的构建。 |
| `amount` | 字符串 | 此收据所指向的 TXO 的总值。 |

## 示例

```text
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
```

