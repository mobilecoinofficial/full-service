---
description: Get the current balance for a given address. The response will have a map of the total values for each token_id that is present at that address. If no tokens are found at that address, the map will be empty. Orphaned will always be 0 for addresses.
---

# Get Address Status

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)
| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `min_block_index` | The minimum block index to filter on txos received | |
| `max_block_index` | The maximum block index to filter on txos received | |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_balance_for_address",
  "params": {
    "address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
  },
  "jsonrpc": "2.0",
  "api_version": "2",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_balance_for_address",
  "result": {
    "account_block_height": "154320",
    "network_block_height": "154320",
    "local_block_height": "154320",
    "balance_per_token": {
      "0": {
        "unverified": "0000000000"
        "unspent": "110000000000000000",
        "pending": "0",
        "spent": "0",
        "secreted": "0",
        "orphaned": "0"
      },
      "1": {
        "unverified": "0000000000"
        "unspent": "1100000000",
        "pending": "0",
        "spent": "0",
        "secreted": "0",
        "orphaned": "0"
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  "api_version": "2"
}
```
{% endtab %}
{% endtabs %}

