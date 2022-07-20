---
description: Get the current balance for a given account. The response will have a map of the total values for each token_id that is present in the account. If no tokens are found at the account, the map will be empty.
---

# Get Balance For Account

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `min_block_index` | The minimum block index to filter on txos received | |
| `max_block_index` | The maximum block index to filter on txos received | |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_balance_for_account",
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
  "method": "get_balance_for_account",
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
}
```
{% endtab %}
{% endtabs %}

