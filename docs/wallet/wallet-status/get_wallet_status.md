---
description: Get the current status of a wallet.
---

# Get Wallet Status

## Example

{% tabs %}
{% tab title="Body Request" %}
```text
{
    "method": "get_wallet_status",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_wallet_status",
  "result": {
    "wallet_status": {
      "account_ids": [
        "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
        "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470"
      ],
      "account_map": {
        "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470": {
          "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
          "key_derivation_version:": "2",
          "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
          "name": "Bob",
          "next_subaddress_index": "2",
          "first_block_index": "3500",
          "object": "account",
          "recovery_mode": false
        },
        "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17": {
          "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
          "key_derivation_version:": "2",
          "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
          "name": "Carol",
          "next_subaddress_index": "2",
          "first_block_index": "3500",
          "object": "account",
          "recovery_mode": false
        }
      },
      "is_synced_all": false,
      "local_block_height": "152918",
      "network_block_height": "152918",
      "object": "wallet_status",
      "total_orphaned_pmob": "0",
      "total_pending_pmob": "70148220000000000",
      "total_secreted_pmob": "0",
      "total_spent_pmob": "0",
      "total_unspent_pmob": "220588320000000000"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

