---
description: 账户地址是通过账户密钥创建的公开地址。一个账户地址通常包括一个只读（View）公钥和一个可花（Spend）公钥。在启用了移动模式的时候，还会包括雾服务（Fog）的信息。

---

# 账户地址

Full Service 钱包中的账户地址可以用来区分交易的发款方。由于 MobileCoin 的隐私属性，在不使用子地址的情况下，钱包将无法区分每笔交易的发款方。唯一能够用来区分发款方的方式是为每个联系人创建一个独立的子地址，并只将该子地址告知对方。

子地址的原理是通过子地址索引（Subaddress Index）通过一系列的密码学操作来生成一个新的地址。

注意：如果您通过一个并未分配的子地址接受了转账，在您分配该子地址之前您将无法使用这笔资金。这被称为“孤立”资金，直到您通过分配对应的子地址来“找回”它们。

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "address" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `public_address` | 字符串 | Base 58 编码的地址。 |
| `account_id` | 字符串 | 该地址被分配的关联账户的唯一标识符。 |
| `metadata` | 字符串 | 该对象附带的任意字符串。 |
| `subaddress_index` | 字符串，内容为 64 位无符号整数 | 在关联账户中该地址的子地址索引（Subaddress Index）。 |
| `offset_count` | 整数 | 从列表 assigned\_address 中请求的分页偏移量。请求将只会返回当前对象之后的内容（不包括当前对象）。 The value to offset pagination requests for assigned\_address list. Requests will exclude all list items up to and including this object. |

## 示例

```text
{
  "object": "address",
  "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
  "metadata": "",
  "subaddress_index": "2",
  "offset_count": "7"
}
```

