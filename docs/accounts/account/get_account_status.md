---
description: >-
  Get the current status of a given account. The account status includes both
  the account object and the balance object.
---

# Get Account Status

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_account_status",
  "params": {
     "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_account_status",
  "result": {
    "account": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "balance": {
      "account_block_height": "152918",
      "is_synced": true,
      "local_block_height": "152918",
      "network_block_height": "152918",
      "object": "balance",
      "orphaned_pmob": "0",
      "pending_pmob": "2040016523222112112",
      "secreted_pmob": "204273415999956272",
      "spent_pmob": "0",
      "unspent_pmob": "51080511222211091"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

