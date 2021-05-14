---
description: 通过助记词来导入一个既存账户。
---

# 导入账户

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `mnemonic` | 用来找回账户的助记词组。 | 助记词必须为 24 个英文单词。 |
| `key_derivation_version` | 通过助记词生成账户密钥的算法的版本号。当前版本为 2。 |  |

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

