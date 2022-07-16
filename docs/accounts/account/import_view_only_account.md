---
description: >-
  Create a view-only account by importing the private key from an existing
  account. Note: a single wallet cannot have both the regular and view-only versions of an account.
---

# Import

## Parameters

| Required Param | Purpose                                                                            | Requirements |
| -------------- | ---------------------------------------------------------------------------------- | ------------ |
| `view_private_key`      | The view private key of this account |              |
| `spend_public_key`      | The spend public key of this account |              |

| Optional Param | Purpose                                                                            | Requirements |
| -------------- | ---------------------------------------------------------------------------------- | ------------ |
| `name`      |  |              |
| `first_block_index`      |  |              |
| `next_subaddress_index`      |  |              |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "import_view_only_account",
  "result": {
    "account": {
      "view_private_key": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "spend_public_key": "fcewc434g5353v535323f43f43f43g5342v3b67n8576453f4dcv56b77n857b46",
      "name": "Coins for cats",
      "first_block_index": "3500",
      "next_block_index": "4000",
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}

{% tab title="Response" %}
```
{
    "method": "import_view_only_account",
    "params": {
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
    "jsonrpc": "2.0",
    "api_version": "2",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
