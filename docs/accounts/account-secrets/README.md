---
description: 出于安全考虑，账户密钥和其他的账户信息并不能同时获取。
---

# 账户密钥

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "account\_secrets" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `account_id` | 字符串 | 账户的唯一标识符。 |
| `mnemonic` | 字符串 | 以 BIP39 编码的助记词序列，用来生成账户密钥。  |
| `key_derivation_version` | 字符串，内容为 64 位无符号整数 | 用于从助记词序列生成账户密钥的路径版本。 |
| `account_key` | account\_key | 账户的只读（View）密钥和可花（Spend）密钥。也会包括连接到雾账簿服务所需要的 URL 和密钥。 |

## 示例

```text
{
  "object": "account_secrets",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
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
```

