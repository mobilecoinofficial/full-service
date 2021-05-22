---
description: 根据账户备份密钥（Secret Entrop）导入既存账户。
---

# 导入账户（旧版）

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `entropy` | 备份根密钥（root entropy） 。 | 十六进制编码的 32 位随机数。 |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `name` | 账户名称。 | 账户名称可以重复，但是我们并不建议您这样做。 |
| `next_subaddress_index` | 该账户已知的下一个可用子地址下标。  |  |
| `first_block_index` | 账簿扫描的起始区块。 |  |
| `fog_report_url` |  |  |
| `fog_report_id` |  |  |
| `fog_authority_spki` |  |  |

## 示例

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

