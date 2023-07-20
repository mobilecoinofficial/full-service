---
description: Assign an address to a given account.
---

# Assign Address For Account

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40-L43)

### Required Params

| Param        | Purpose                                      | Requirements                          |
| ------------ | -------------------------------------------- | ------------------------------------- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

### Optional Params

| Param      | Purpose                        | Requirements                          |
| ---------- | ------------------------------ | ------------------------------------- |
| `metadata` | The metadata for this address. | String; can contain stringified JSON. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41-L43)

## Examples

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "assign_address_for_account",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "metadata": "For transactions from Bob"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "assign_address_for_account",
  "result": {
    "address": {
      "public_address_b58": "5WyFFfKNKX7DTaNTg9V7wQwaSbmDjaNzBa1QcYnBmM7B8i7yvqbcaN8bnMQLACiCBS9G4QCfESRa4TQwiKLQ7zAKvDSeUcCQGbBu6BmW7XG",
      "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "metadata": "For transactions from Bob",
      "subaddress_index": "3"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
