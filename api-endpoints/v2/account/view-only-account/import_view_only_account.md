---
description: >-
  Create a view-only account by importing the private key from an existing
  account. Note: a single wallet cannot have both the regular and view-only
  versions of an account.
---

# Import View Only Account

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param     | Purpose                              | Requirements |
| ------------------ | ------------------------------------ | ------------ |
| `view_private_key` | The view private key of this account |              |
| `spend_public_key` | The spend public key of this account |              |

| Optional Param          | Purpose | Requirements |
| ----------------------- | ------- | ------------ |
| `name`                  |         |              |
| `first_block_index`     |         |              |
| `next_subaddress_index` |         |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "import_view_only_account",
    "params": {
        "view_private_key": "0a2078062debfa72270373d13d52e228b2acc7e3d55790447e7a58905b986fc3780a",
        "spend_public_key": "0a208007986832d9269e62d9c0b2a33478ec761f8b6f6c32316bc8a993ed02964d51",
        "name": "Bob",
        "first_block_index": "1352037",
        "next_subaddress_index": "2"
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"import_view_only_account",
  "result":{
    "account":{
      "id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
      "name":"Bob",
      "key_derivation_version":"2",
      "main_address":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
      "next_subaddress_index":"2",
      "first_block_index":"1352037",
      "next_block_index":"1352037",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":true
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
