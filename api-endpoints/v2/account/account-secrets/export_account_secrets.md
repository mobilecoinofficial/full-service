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
    "account_id": "b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"export_account_secrets",
  "result":{
    "account_secrets":{
      "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
      "name":"SAMPLE ACCOUNT",
      "mnemonic":"into text kick two bread dish air simple throw glow topic yard heavy donkey guess permit captain thank weekend daring mad helmet safe olympic",
      "key_derivation_version":"2",
      "account_key":{
        "view_private_key":"0a2078062debfa72270373d13d52e228b2acc7e3d55790447e7a58905b986fc3780a",
        "spend_private_key":"0a201f4ba0099acc804e09b011deeabc6c5e1ce6f9a8fd626dcccb0dfd4142c63209",
        "fog_report_url":"",
        "fog_report_id":"",
        "fog_authority_spki":""
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

## Outputs

If the account was generated using version 1 of the key derivation, entropy will be provided as a hex-encoded string.

If the account was generated using version 2 of the key derivation, mnemonic will be provided as a 24-word mnemonic string.
