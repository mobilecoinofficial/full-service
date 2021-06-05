---
description: 获取指定账户的全部已分配地址。
---

# 获取全部账户地址

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 要查询地址的账户。 | 指定的账户必须存在在钱包中。 |

## 示例

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
verify_a
```
{% endtab %}
{% endtabs %}

