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

| Optional Param          | Purpose                                                 | Requirements                                            |
| ----------------------- | ------------------------------------------------------- | ------------------------------------------------------- |
| `name`                  | A label for this account.                               | A label can have duplicates, but it is not recommended. |
| `first_block_index`     | The block from which to start scanning the ledger.      | All subaddresses below this index will be created.      |
| `next_subaddress_index` | The next known unused subaddress index for the account. |                                                         |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "import_view_only_account",
    "params": {
        "view_private_key": "0a20952e7ea32b80f5249c8fa470913dd67b1f722e12f516d0aff8a64f95faf6cb07",
        "spend_public_key": "0a20e22bcedd966ac737cbe53445dc983433072b771440e7c9f2619f775db2cc0448",
        "name": "Alice",
        "first_block_index": "1769454",
        "next_subaddress_index": "2"
        "require_spend_subaddress": false
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "import_view_only_account",
  "result": {
    "account": {
      "id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name": "Alice",
      "key_derivation_version": "2",
      "main_address": "8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index": "2",
      "first_block_index": "1769454",
      "next_block_index": "1769454",
      "recovery_mode": false,
      "fog_enabled": false,
      "view_only": true,
      "require_spend_subaddress": false
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
