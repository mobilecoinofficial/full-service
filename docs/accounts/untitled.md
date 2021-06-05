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
| `next_subaddress_index` | 字符串，内容为 64 位无符号整数 | 指向下一个可以被分配的地址的索引。主要在帐户是从其他地方导入的情况下使用。 |
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

## 方法

### `create_account`

在钱包中创建一个新的账户。

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `name` | 账户名称。 | 账户名称可以重复，但是我们并不建议您这样做。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "create_account",
  "params": {
    "name": "Alice"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "create_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Alice",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  }
}
```
{% endtab %}
{% endtabs %}

### `import_account`

通过助记词来导入一个既存账户。

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "import_account",
  "params": {
    "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
    "key_derivation_version": "2",
    "name": "Bob"
    "next_subaddress_index": 2,
    "first_block_index": "3500"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `mnemonic` | 用来找回账户的助记词组。 | 助记词必须为 24 个英文单词。 |
| `key_derivation_version` | 通过助记词生成账户密钥的算法的版本号。当前版本为 2。 |  |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `name` | 账户名称。 | 账户名称可以重复，但是我们并不建议您这样做。 |
| `next_subaddress_index` | 该账户已知的下一个可用子地址索引。  |  |
| `first_block_index` | 账簿扫描的起始区块。 |  |


### `import_account_from_legacy_root_entropy` \(已废除\)

根据账户备份密钥（Secret Entropy）导入既存账户。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `entropy` | 备份根密钥（root entropy） 。 | 十六进制编码的 32 位随机数。 |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `name` | 账户名称。 | 账户名称可以重复，但是我们并不建议您这样做。 |
| `next_subaddress_index` | 该账户已知的下一个可用子地址索引。  |  |
| `first_block_index` | 账簿扫描的起始区块。 |  |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "import_account_from_legacy_root_entropy",
  "params": {
    "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
    "name": "Bob"
    "next_subaddress_index": 2,
    "first_block_index": "3500",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
`如果您尝试导入一个已经在钱包内的账户，您会收到如下错误信息：`

```text
{"error": "Database(Diesel(DatabaseError(UniqueViolation, "UNIQUE constraint failed: accounts.account_id_hex")))"}
```
{% endhint %}

### `get_account`

获取指定账户的详细信息。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用来查询状态的账户。 | 指定的账户必须存在在钱包中。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_account",
  "params": {
    "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_account",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "key_derivation_version:": "2",
      "name": "Alice",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
如果指定账户不存在在数据库里，您会收到如下报错：

```text
{
  "error": "Database(AccountNotFound(\"a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10\"))",
  "details": "Error interacting with the database: Account Not Found: a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
}
```
{% endhint %}

### `get_all_accounts`

获取指定钱包中的全部账户信息。

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_all_accounts",
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_all_accounts",
  "result": {
    "account_ids": [
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0"
    ],
    "account_map": {
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52": {
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "key_derivation_version:": "1",
        "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "name": "Alice",
        "next_subaddress_index": "2",
        "first_block_index": "3500",
        "object": "account",
        "recovery_mode": false
      },
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0": {
        "account_id": "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0",
        "key_derivation_version:": "2",
        "main_address": "7EqduSDpM1R5AfQejbjAqFxpuCoh6zJECtvJB9AZFwjK13dCzZgYbyfLf4TfHcE8LVPjzDdpcxYLkdMBh694mHfftJmsFZuz6xUeRtmsUdc",
        "name": "Alice",
        "next_subaddress_index": "2",
        "first_block_index": "3500",
        "object": "account",
        "recovery_mode": false
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_account_status`

获取一个指定账户的当前状态，包括账户对象和余额对象。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用来查询状态的账户。 | 指定的账户必须存在在钱包中。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_account_status",
  "params": {
     "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_account_status",
  "result": {
    "account": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "balance": {
      "account_block_index": "152918",
      "is_synced": true,
      "local_block_index": "152918",
      "network_block_index": "152918",
      "object": "balance",
      "orphaned_pmob": "0",
      "pending_pmob": "2040016523222112112",
      "secreted_pmob": "204273415999956272",
      "spent_pmob": "0",
      "unspent_pmob": "51080511222211091"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `update_account_name`

重命名一个账户。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要重命名的账户 ID。  | 指定的账户必须存在在钱包中。  |
| `name` |  账户的新名字。 |  |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "update_account_name",
  "params": {
    "acount_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
    "name": "Carol"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "update_account_name",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "name": "Carol",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `remove_account`

从钱包内移除指定账户。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要移除的账户 ID | 指定的账户必须存在在钱包中。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "remove_account",
  "params": {
    "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "remove_account",
  "result": {
    "removed": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

