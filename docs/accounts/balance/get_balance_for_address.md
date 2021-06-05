---
description: 获取指定地址的当前余额。
---

# 获取地址余额

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `address` | 要查询余额的账户地址。 | 地址必须已分配给钱包内的账户。 |

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_balance_for_address",
  "params": {
     "address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
  },
  "jsonrpc": "2.0",
  "api_version": "2",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_balance_for_address",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152961",
      "local_block_index": "152961",
      "account_block_index": "152961",
      "is_synced": true,
      "unspent_pmob": "11881402222024",
      "pending_pmob": "0",
      "spent_pmob": "84493835554166",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  "api_version": "2"
}
```
{% endtab %}
{% endtabs %}

