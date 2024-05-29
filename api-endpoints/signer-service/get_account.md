---
description: Get account info from an existing mnemonic
---

# Get Account

## Request

| Param           | Requirements                                                                                                                                                                                                                                                                                          |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mnemonic`      | Must be a valid 24 word mnemonic from which account keys will be derived.                                                                                                                                                                                                                             |
| `bip39_entropy` | 64 character hex-encoded string containing a 32 byte (256 bit) account secret from which account keys will be derived. Can optionally be provided instead of the above mnemonic. When using the bip39 standard, one can derive the same account keys using the mnemonic or its corresponding entropy. |

## Response

{% tabs %}
{% tab title="Request (mnemonic)" %}
```json
{
  "method": "get_account",
  "params": {
    "mnemonic": "divorce tortoise note draw forest strike replace cost also crowd front unusual demand south again rather pencil next remind future rally carry keen artefact"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Request (bip39_entropy)" %}
```json
{
  "method": "get_account",
  "params": {
    "bip39_entropy": "401cb25aa135b3ae2db185074689757723a3a0412d92a2b2aad72f4b1445de68"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```json
{
    "method": "get_account",
    "result": {
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
