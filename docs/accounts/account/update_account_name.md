---
description: 重命名一个账户
---

# 重命名账户

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要重命名的账户 ID。  | 指定的账户必须存在在钱包中。  |
| `name` |  账户的新名字。 |  |

## 示例

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

