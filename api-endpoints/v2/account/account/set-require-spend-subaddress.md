# Set Require Spend Subaddress

The default behavior of full-service is to treat all unspent txos which belong to an account as eligible inputs for [building an outgoing transaction](../../transaction/transaction/build\_and\_submit\_transaction.md).  There are two optional parameters to the transaction builder which change this behavior.  The first is to specify an explicit list of candidate `input_txo_ids.` The second is to specify a `spend_subaddress` which the transaction builder will use to filter down the account's unspent txos to only those which were sent to the provided subaddress.

When the account has `require_spend_subaddress` set `true,`either using `set_require_spend_subaddress` ; or, by setting it with [`create_account`](create\_account.md) or [`import_account`](import\_account.md), then the `spend_subaddress` request parameter to the transaction builder becomes required and is no longer merely optional.

## Request

| Required Param             | Purpose                                                                                                                                                                          | Requirements                     |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| `account_id`               | The account on which to enable `require_spend_subaddress`                                                                                                                        | Account must exist in the wallet |
| `require_spend_subaddress` | enable (set `true`) or clear (set `false`) the requirement that the transaction builder must be provided with a `spend_subaddress` when building transactions for `account_id`.  | `bool`                           |

## Response

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "enable_require_spend_subaddress",
    "params": {
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "require_spend_subaddress": true,
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"enable_require_spend_subaddress",
  "result":{
    "account":{
      "id":"60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "name":"Carol",
      "key_derivation_version":"2",
      "main_address":"8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
      "next_subaddress_index":"2",
      "first_block_index":"1769454",
      "next_block_index":"1769454",
      "recovery_mode":false,
      "fog_enabled":false,
      "view_only":false,
      "require_spend_subaddress":true
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
