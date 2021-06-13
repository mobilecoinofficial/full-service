---
description: 红包是一个只包含一个 TXO 的一次性的账户。在放入资金后，可以通过不同的途径（比如二维码）分享给接收方去接收红包内的资金。
---

# 红包码

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "gift_code" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `gift_code` | 字符串 | 用来分享的由 Base 58 编码的红包码。 |
| `entropy` | 字符串 | 临时红包账户的加密口令。 |
| `value_pmob` | 字符串 | 临时红包账户内包含的 MOB 数量。 |
| `memo` | 字符串 | 红包附带的备注消息。 |

## 示例

```text
{
  "object": "gift_code",
  "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
  "entropy": "41e1e794f8a2f7227fa8b5cd936f115b8799da712984c85f499e03bca43cba9c",
  "value_pmob": "60000000000",
  "memo": "新年快乐！",
  "account_id": "050d8d97aaf31c70d63c6aed828c11d3fb16b56b44910659b6724621047b81f9",
  "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
}
```

