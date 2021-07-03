---
description: 钱包在构建 TXO 时会生成一个确认编码。
---

# 确认编码

确认编码可以由发送方提供给接收方以证明一个 TXO 确实是由发送方发出的。

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "confirmation" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `txo_id` | 字符串 | TXO 的唯一标识符。 |
| `txo_index` | 字符串 | TXO 在账簿上的索引。 |
| `confirmation` | 字符串 | 一个包含确认编码的字符串，可以被验证以证明交易的另一方确实参与了该 TXO 的构造。 |

## 示例

```text
{
  "object": "confirmation",
  "txo_id": "873dfb8c...",
  "txo_index": "1276",
  "confirmation": "984eacd..."
}
```

