---
description: Create a new account in the wallet.
---

# Create Account

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L82)

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |
| `fog_info` | The [Fog Info](../../../full-service/src/json_rpc/v2/models/account_key.rs#L67) to include in public addresses | |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L65)

{% tabs %}
{% tab title="Request Body" %}
```text
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
```text
{
  "method": "create_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Alice",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  }
}
```
{% endtab %}
{% endtabs %}

