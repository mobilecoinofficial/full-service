---
description: 区块是 MobileCoin 区块链的重要基础组成部分，每个区块都包括了交易输出（TXO）和密钥镜像（Key Images）。
---

# 区块

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "block" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `block` | JSON 对象 | 包含了区块的头部信息。|
| `block_contents` | JSON 对象 | 包含了区块的密钥镜像 \(key\_images\) 和交易输出 \(TXO\)。 |

## Example

