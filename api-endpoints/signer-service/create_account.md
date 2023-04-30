---
description: Create a mnemonic and account info for a new account
---

# Create Account

## Request

No Params

## Response

{% tabs %}
{% tab title="Request Body" %}

```json
{
  "method": "create_account",
  "params": {
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
    "mnemonic": "divorce tortoise note draw forest strike replace cost also crowd front unusual demand south again rather pencil next remind future rally carry keen artefact",
    "account_info": {
      "view_private": "4adb853513669514029e2f3ff0d1340638563fa1cff31b5a0a68aa70fe9e6c04",
      "spend_public": "fe2bdfa1e3364b16bf23686f87499577c634477b553e7b3ddcaa35a9e8ec4e12",
      "account_index": 0
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}
{% endtabs %}
