---
description: >-
  The balance of an account, which includes additional information about the
  syncing status needed to interpret the balance correctly.
---

# Balance

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "balance" | String representing the object's type. Objects of the same type share the same value. |
| `network_block_index` | string \(uint64\) | The block height of MobileCoin's distributed ledger. The `local_block_index` is synced when it reaches the `network_block_index`. |
| `local_block_index` | string \(uint64\) | The local block height downloaded from the ledger. The local database will sync up to the `network_block_index`. The `account_block_index` can only sync up to `local_block_index`. |
| `account_block_index` | string \(uint64\) | The scanned local block height for this account. This value will never be greater than the `local_block_index`. At fully synced, it will match `network_block_index`. |
| `is_synced` | boolean | Whether the account is synced with the `network_block_index`. Balances may not appear correct if the account is still syncing. |
| `unspent_pmob` | string \(uint64\) | Unspent pico MOB for this account at the current `account_block_index`. If the account is syncing, this value may change. |
| `pending_pmob` | string \(uint64\) | Pending, out-going pico MOB. The pending value will clear once the ledger processes the outgoing TXOs. The `pending_pmob` will reflect the change. |
| `spent_pmob` | string \(uint64\) | Spent pico MOB. This is the sum of all the TXOs in the wallet which have been spent. |
| `secreted_pmob` | string \(uint64\) | Secreted \(minted\) pico MOB. This is the sum of all the TXOs which have been created in the wallet for outgoing transactions. |
| `orphaned_pmob` | string \(uint64\) | Orphaned pico MOB. The orphaned value represents the TXOs which were view-key matched, but which can not be spent until their subaddress index is recovered. |

## Example

```text
{
  "account_block_index": "152003",
  "is_synced": false,
  "local_block_index": "152918",
  "network_block_index": "152918",
  "object": "balance",
  "orphaned_pmob": "0",
  "pending_pmob": "0",
  "secreted_pmob": "0",
  "spent_pmob": "0",
  "unspent_pmob": "110000000000000000"
}
```

## Methods

### `get_balance_for_account`

Get the current balance for a given account.

{% tabs %}
{% tab title="get\_balance\_for\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_balance_for_account",
        "params": {
           "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_balance_for_account",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152918",
      "local_block_index": "152918",
      "account_block_index": "152003",
      "is_synced": false,
      "unspent_pmob": "110000000000000000",
      "pending_pmob": "0",
      "spent_pmob": "0",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | String | The unique identifier for the account. |

### `get_balance_for_address`

Get the current balance for a given address.

{% tabs %}
{% tab title="get\_balance\_for\_address" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_balance_for_address",
        "params": {
           "address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
        },
        "jsonrpc": "2.0",
        "api_version": "2",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_balance_for_address",
  "result": {
    "balance": {
      "object": "balance",
      "network_block_index": "152961",
      "local_block_index": "152961",
      "account_block_index": "152961",
      "is_synced": true,
      "unspent_pmob": "11881402222024",
      "pending_pmob": "0",
      "spent_pmob": "84493835554166",
      "secreted_pmob": "0",
      "orphaned_pmob": "0"
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

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | String | The unique identifier for the account. |

