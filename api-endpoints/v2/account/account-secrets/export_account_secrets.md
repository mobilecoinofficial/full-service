---
description: >-
  Exporting the secret mnemonic an account is the only way to recover it when
  lost.
---

# Export Account Secrets

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                      |
| -------------- | -------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "export_account_secrets",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "export_account_secrets",
  "result": {
    "account_secrets": {
      "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name": "Carol",
      "mnemonic": "bomb canyon vibrant giant convince grid little jeans frost mail depart rib hope rebuild green gesture birth arrive carry vault artefact average silent oval",
      "key_derivation_version": "2",
      "account_key": {
        "view_private_key": "0a20952e7ea32b80f5249c8fa470913dd67b1f722e12f516d0aff8a64f95faf6cb07",
        "spend_private_key": "0a206f3460191ed6d1441350033d6a98e49b196e8815e626fbfe99a5a97c250ca303",
        "fog_report_url": "",
        "fog_report_id": "",
        "fog_authority_spki": ""
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

## Outputs

If the account was generated using version 1 of the key derivation, entropy will be provided as a hex-encoded string.

If the account was generated using version 2 of the key derivation, mnemonic will be provided as a 24-word mnemonic string.
