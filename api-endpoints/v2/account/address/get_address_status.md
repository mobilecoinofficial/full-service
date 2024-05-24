---
description: >-
  Get the current balance for a given address. The response will have a map of
  the total values for each token_id that is present at that address.
---

# Get Address Status

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                                           |
| -------------- | -------------------------------------------- | ------------------------------------------------------ |
| `address`      | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

| Optional Param    | Purpose                                            | Requirements |
| ----------------- | -------------------------------------------------- | ------------ |
| `min_block_index` | The minimum block index to filter on txos received |              |
| `max_block_index` | The maximum block index to filter on txos received |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

**Note:**  `max_spendable`, `unspent`, and `spent` balances should be used with caution. The default behavior of full-service, is to:

1. build transactions that spend an account's utxos without regard for the subaddress to which those utxos were addressed;
2. send change (amount in the input txos that exceeds the amount being sent to the counterparty) to a designated change subaddress; and,
3. automatically defragment the funds in the account by consolidated utxos during each transaction by filling unused input slots -- each transaction can have up to 16 inputs -- with the smallest utxos in the account, reducing the total number of utxos in the account in favor of larger change utxos.

The combination of these three activities will alter balances returned by get\_address\_status without any explicit activity having occured related to the subaddress being queried.

Wallet implementors that want to track balances on per-subaddress basis can keep their own database that uses the `subaddress_index` in response to [`get_txos`](../../transaction/txo/get\_txos.md) to credit funds received to a subaddress and separately from full-service track and account for how those funds are depleted.

Wallet implementors can also override the default behavior of the transaction builder by specifying a `spend_subaddress` when composing transactions. When a `spend_subaddress` is specified

1. The transaction builder will only use utxos as inputs that were sent to that subaddress, and
2. change will be sent back to the `spend_subaddress` instead of the reserved change subaddress.

This enhances the utility of the `max_spendable`, `unspent`, and `spent` balance information reported by `get_address_status`. &#x20;

It is recommend to [set](../account/set-require-spend-subaddress.md) the `require_spend_subaddress` flag to `true` for accounts where the wallet will use balances from `get_address_status` as this will prevent inadvertently  building transactions that omit `spend_subaddress` and throw off the balances of `get_address_status` by building transactions that spend input txos without regard for the subaddress of those inputs.

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_address_status",
  "params": {
    "address": "8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_address_status",
  "result":{
    "address":{
      "public_address_b58":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "account_id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "metadata":"Main",
      "subaddress_index":"0"
    },
    "account_block_height":"1769486",
    "network_block_height":"1769486",
    "local_block_height":"1769486",
    "balance_per_token":{
      "0":{
        "max_spendable":"8039600015840",
        "unverified":"0",
        "unspent":"8040000067868",
        "pending":"0",
        "spent":"8065834220882873",
        "secreted":"0",
        "orphaned":"0"
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
