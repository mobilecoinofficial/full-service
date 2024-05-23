---
description: Import an existing account from the secret entropy.
---

# Import Account Legacy

## Request

| Required Param | Purpose                  | Requirements                         |
| -------------- | ------------------------ | ------------------------------------ |
| `entropy`      | The secret root entropy. | 32 bytes of randomness, hex-encoded. |

| Optional Param             | Purpose                                                                                 | Requirements                                                     |
| -------------------------- | --------------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| `name`                     | A label for this account.                                                               | A label can have duplicates, but it is not recommended.          |
| `next_subaddress_index`    | The next known unused subaddress index for the account.                                 |                                                                  |
| `first_block_index`        | The block from which to start scanning the ledger.                                      |                                                                  |
| `fog_report_url`           | Fog Report server url.                                                                  | Applicable only if user has Fog service, empty string otherwise. |
| `fog_report_id`            | Fog Report server ID                                                                    | Unused                                                           |
| `fog_authority_spki`       | Fog Authority Subject Public Key Info.                                                  | Applicable only if user has Fog service, empty string otherwise. |
| `require_spend_subaddress` | Indicate if all transactions built using this account must specify a `spend_subaddress` | boolean that defaults to false if not included.                  |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "import_account_from_legacy_root_entropy",
  "params": {
    "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
    "name": "Alice",
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
      "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name": "Alice",
      "main_address": "8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false,
      "require_spend_subaddress": false
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
