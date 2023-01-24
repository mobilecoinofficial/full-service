---
description: Import an existing account from the secret entropy.
---

# Import Account

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `mnemonic` | The secret mnemonic to recover the account. | The mnemonic must be 24 words. |
| `key_derivation_version` | The version number of the key derivation used to derive an account key from this mnemonic. The current version is 2. |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |
| `next_subaddress_index` | The next known unused subaddress index for the account. |  |
| `first_block_index` | The block from which to start scanning the ledger. |  |
| `fog_report_url` |  |  |
| `fog_report_id` |  |  |
| `fog_authority_spki` |  |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "import_account",
  "params": {
    "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
    "key_derivation_version": "2",
    "name": "Bob",
    "next_subaddress_index": 2,
    "first_block_index": "3500"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"import_account",
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
      "view_only":false
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
`If you attempt to import an account already in the wallet, you will see the following error message:`

```text
{
  "method":"import_account",
  "error":{
    "code":-32603,
    "message":"InternalError",
    "data":{
      "server_error":"Database(AccountAlreadyExists(\"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c\"))",
      "details":"Error interacting& with the database: Account already exists: b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c"
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endhint %}