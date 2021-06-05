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

## 方法

### `assign_address_for_account`

为指定账户分配地址。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要分配地址的账户。 | 指定的账户必须存在在钱包中。 |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| ​`metadata` | 这个地址的元数据。| 字符串。可以包含字符串化的 JSON。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "assign_address_for_account",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "metadata": "为了和 Carol 进行交易"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "assign_address_for_account",
  "result": {
    "address": {
      "object": "address",
      "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "metadata": "",
      "subaddress_index": "2",
      "offset_count": "7"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_all_addresses_for_account`

获取指定账户的全部已分配地址。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要查询地址的账户。 | 指定的账户必须存在在钱包中。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_all_addresses_for_account",
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
  "method": "get_all_addresses_for_account",
  "result": {
    "public_addresses": [
      "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG",
      "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
    ],
    "address_map": {
      "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3": {
        "object": "address",
        "public_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Main",
        "subaddress_index": "0",
        "offset_count": "5"
      },
      "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG": {
        "object": "address",
        "public_address": "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Change",
        "subaddress_index": "1",
        "offset_count": "6"
      },
      "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z": {
        "object": "address",
        "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "",
        "subaddress_index": "2",
        "offset_count": "7"
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

### `verify_address`

验证一个地址是不是正确的 Base 58 编码。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `address` | 要验证的地址。 | 地址必须已分配给钱包内的账户。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "verify_address",
  "params": {
    "address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "verify_address",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

