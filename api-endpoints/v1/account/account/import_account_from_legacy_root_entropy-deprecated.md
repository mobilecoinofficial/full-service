---
description: Import an existing account from the secret entropy.
---

# Import Account Legacy

## Parameters

| Required Param | Purpose                  | Requirements                         |
| -------------- | ------------------------ | ------------------------------------ |
| `entropy`      | The secret root entropy. | 32 bytes of randomness, hex-encoded. |

| Optional Param          | Purpose                                                 | Requirements                                            |
| ----------------------- | ------------------------------------------------------- | ------------------------------------------------------- |
| `name`                  | A label for this account.                               | A label can have duplicates, but it is not recommended. |
| `next_subaddress_index` | The next known unused subaddress index for the account. |                                                         |
| `first_block_index`     | The block from which to start scanning the ledger.      |                                                         |
| `fog_report_url`        |                                                         |                                                         |
| `fog_report_id`         |                                                         |                                                         |
| `fog_authority_spki`    |                                                         |                                                         |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "import_account_from_legacy_root_entropy",
  "params": {
    "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
    "name": "Bob"
    "next_subaddress_index": 2,
    "first_block_index": "3500",
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

{% hint style="warning" %}
`If you attempt to import an account already in the wallet, you will see the following error message:`

```
{"error": "Database(Diesel(DatabaseError(UniqueViolation, "UNIQUE constraint failed: accounts.account_id_hex")))"}
```
{% endhint %}
