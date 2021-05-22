---
description: >-
  每个钱包内的账户都关联着一个 AccountKey，一个 AccountKey 包含一组只读（View）密钥对和一组可花（Spend）密钥对。
---

# 账户 API

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "account" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `account_id` | 字符串 | 账户的唯一标识符。 |
| `name` | 字符串 | 账户的显示名称。 |
| `main_address` | 字符串 | Base 58 编码的账户主地址。账户主地址由种子地址决定。主地址不能被用作特定收款地址，而应当作为通用收款地址。 |
| `next_subaddress_index` | 字符串，内容为 64 位无符号整数 | 指向下一个可以被分配的地址的下标。主要在帐户是从其他地方导入的情况下使用。 |
| `recovery_mode` | 布尔型 | 当本字段为 `true` 时，说明此账户正在尝试回溯账户内每一个交易结果的父区块。我们建议当您在不确定交易结果的指定地址时，在回溯结束后将所有的 MOB 都转入其他账户。 |

## 示例

```text
{
  "object": "account",
  "account_id": "gdc3fd37f1903aec5a12b12a580eb837e14f87e5936f92a0af4794219f00691d",
  "name": "I love MobileCoin",
  "main_address": "8vbEtknX7zNtmN5epTYU95do3fDfsmecDu9kUbW66XGkKBX87n8AyqiiH9CMrueo5H7yiBEPXPoQHhEBLFHZJLcB2g7DZJ3tUZ9ArVgBu3a",
  "next_subaddress_index": "3",
  "first_block_index": "3500",
  "recovery_mode": false
}
```

