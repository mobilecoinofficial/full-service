---
description: 为账户分配地址。
---

# 为账户分配地址

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要分配地址的账户。 | 指定的账户必须存在在钱包中。 |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| ​`metadata` | 这个地址的元数据。| 字符串。可以包含字符串化的 JSON。 |

## 示例

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

