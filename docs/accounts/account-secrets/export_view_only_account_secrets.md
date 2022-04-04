---
description: >-
  Exporting the view private key for a view only account
---

# Export Account Secrets

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "export_view_only_account_secrets",
  "params": {
    "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "export_view_only_account_secrets",
  "result": {
    "view_only_account_secrets": {
      "object": "view_only_account_secrets",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "view_private_key": "c0b285cc589447c7d47f3yfdc466e7e946753fd412337bfc1a7008f0184b0479",
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

## Outputs

If the account was generated using version 1 of the key derivation, entropy will be provided as a hex-encoded string.

If the account was generated using version 2 of the key derivation, mnemonic will be provided as a 24-word mnemonic string.

