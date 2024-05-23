---
description: Import an existing account from the secret entropy.
---

# Import Account

## Request

| Required Param           | Purpose                                                                                                              | Requirements                   |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| `mnemonic`               | The secret mnemonic to recover the account.                                                                          | The mnemonic must be 24 words. |
| `key_derivation_version` | The version number of the key derivation used to derive an account key from this mnemonic. The current version is 2. |                                |

| Optional Param          | Purpose                                                                                                                                                                                            | Requirements                                                     |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| `name`                  | A label for this account.                                                                                                                                                                          | A label can have duplicates, but it is not recommended.          |
| `next_subaddress_index` | <p>The next known unused subaddress index for the account. All subaddresses below this index will be created.<br>This can be used to create a large number of subaddresses as a batch request.</p> |                                                                  |
| `first_block_index`     | The block from which to start scanning the ledger.                                                                                                                                                 |                                                                  |
| `fog_report_url`        | Fog Report server url.                                                                                                                                                                             | Applicable only if user has Fog service, empty string otherwise. |
| `fog_report_id`         | Fog Report server ID                                                                                                                                                                               | Unused                                                           |
| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                                                                                                                                             | Applicable only if user has Fog service, empty string otherwise. |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "import_account",
  "params": {
    "mnemonic": "bomb canyon vibrant giant convince grid little jeans frost mail depart rib hope rebuild green gesture birth arrive carry vault artefact average silent oval",
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
```
{
  "method":"import_account",
  "result":{
    "account":{
      "id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name":"Alice",
      "key_derivation_version":"2",
      "main_address":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index":"2",
      "first_block_index":"1769454",
      "next_block_index":"1769454",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":false,
      "require_spend_subaddress":false
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

```
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
