---
description: Import an existing account from the secret entropy.
---

# Import Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `mnemonic` | The secret mnemonic to recover the account. | The mnemonic must be 24 words. |
| `key_derivation_version` | The version number of the key derivation used to derive an account key from this mnemonic. The current version is 2. |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |
| `next_subaddress_index` | The next known unused subaddress index for the account. |  |
| `first_block_index` | The block from which to start scanning the ledger. |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "import_account",
  "params": {
    "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
    "key_derivation_version": "2",
    "name": "Bob"
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
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

