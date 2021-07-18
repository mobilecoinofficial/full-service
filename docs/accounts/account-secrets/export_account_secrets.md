---
description: 导出助记词是找回丢失账户的唯一途径。
---

# 导出账户密钥

##  参数

| 参数 |  用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 指定导出助记词的账户。 | 指定的账户必须存在在钱包中。 |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "export_account_secrets",
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
  "method": "export_account_secrets",
  "result": {
    "account_secrets": {
      "object": "account_secrets",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "entropy": "c0b285cc589447c7d47f3yfdc466e7e946753fd412337bfc1a7008f0184b0479",
      "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
      "key_derivation_version": "2",
      "account_key": {
        "object": "account_key",
        "view_private_key": "0a20be48e147741246f09adb195b110c4ec39302778c4554cd3c9ff877f8392ce605",
        "spend_private_key": "0a201f33b194e13176341b4e696b70be5ba5c4e0021f5a79664ab9a8b128f0d6d40d",
        "fog_report_url": "",
        "fog_report_id": "",
        "fog_authority_spki": ""
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

## 输出

如果账户创建使用的密钥生成算法版本号是 1，那么您会得到一个 16 进制编码的字符串。

如果账户创建使用的密钥生成算法版本号是 2，那么您会得到一个由 24 个单词构成的助记词字符串。

