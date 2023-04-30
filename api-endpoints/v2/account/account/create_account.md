---
description: Create a new account in the wallet.
---

# Create Account

## Request

| Optional Param | Purpose                                                                                                                                                            | Requirements                                            |
|----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------|
| `name`         | A label for this account.                                                                                                                                          | A label can have duplicates, but it is not recommended. |
| `fog_info`     | The [Fog Info](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/models/account_key.rs#L67) to include in public addresses |                                                         |

## Response

{% tabs %}
{% tab title="Request Body" %}

```json
{
  "method": "create_account",
  "params": {
    "name": "Alice"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}

{% tab title="Response" %}

```json
{
  "method": "create_account",
  "result": {
    "account": {
      "id": "b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
      "name": "Alice",
      "key_derivation_version": "2",
      "main_address": "41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
      "next_subaddress_index": "2",
      "first_block_index": "1352037",
      "next_block_index": "1352037",
      "recovery_mode": false,
      "fog_enabled": false,
      "view_only": false
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}
{% endtabs %}
