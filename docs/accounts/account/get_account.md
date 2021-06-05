---
description: 获取指定账户的详细信息。
---

# 获取账户详情

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用来查询状态的账户。 | 指定的账户必须存在在钱包中。 |

## Example

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

