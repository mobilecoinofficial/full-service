---
description: Get assigned addresses for an account.
---

# Get Address

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `public_address_b58` | The public address b58 string to query for. |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_address",
  "params": {
    "public_address_b58": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_address",
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

