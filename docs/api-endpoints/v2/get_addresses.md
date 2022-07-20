---
description: Get assigned addresses for an account.
---

# Get Addresses

## Parameters

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |
| `offset` | The pagination offset. Results start at the offset index | |
| `limit` | Limit for the number of results | |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_addresses",
  "params": {
    "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
    "offset": 1,
    "limit": 1
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_addresses",
  "result": {
    "public_addresses": [
      "7RvvDmRa9CuB5Uf1aDeyKuyhjKtQhxHroAuDh8NFuwfRdQd1QvAhgA8E6Tg34nRo4sM6B1SbPEC8ffz86oYfDKziBw7xYVPKzZ4dvL8p961"
    ],
    "address_map": {
      "7RvvDmRa9CuB5Uf1aDeyKuyhjKtQhxHroAuDh8NFuwfRdQd1QvAhgA8E6Tg34nRo4sM6B1SbPEC8ffz86oYfDKziBw7xYVPKzZ4dvL8p961": {
        "object": "address",
        "public_address": "7RvvDmRa9CuB5Uf1aDeyKuyhjKtQhxHroAuDh8NFuwfRdQd1QvAhgA8E6Tg34nRo4sM6B1SbPEC8ffz86oYfDKziBw7xYVPKzZ4dvL8p961",
        "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
        "metadata": "Change",
        "subaddress_index": "1"
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

