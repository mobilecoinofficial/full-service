---
description: >-
  The secret keys for an account. The account secrets are returned separately
  from other account information, to enable more careful handling of
  cryptographically sensitive information.
---

# Account Secrets

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "account\_secrets" | String representing the object's type. Objects of the same type share the same value. |
| `account_id` | string | The unique identifier for the account. |
| `mnemonic` | string | A BIP39-encoded mnemonic phrase used to generate the account key. |
| `key_derivation_version` | string \(uint64\) | The version number of the key derivation path used to generate the account key from the mnemonic. |
| `account_key` | account\_key | The view and spend keys used to transact on the MobileCoin network. Also may contain keys to connect to the Fog ledger scanning service. |

## Example

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

## Methods

### `export_account_secrets`

{% tabs %}
{% tab title="export\_account\_secrets" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "export_account_secrets",
        "params": {
          "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
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

#### Outputs

If the account was generated using version 1 of the key derivation, entropy will be provided as a hex-encoded string.

If the account was generated using version 2 of the key derivation, mnemonic will be provided as a 24-word mnemonic string.

