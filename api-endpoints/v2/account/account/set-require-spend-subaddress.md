# Set Require Spend Subaddress

Some wallet providers may wish to enforce a pattern within an account so that subaddresses behave like "subaccounts." The standard behavior for Full Service is to spend funds from any TXO owned by the account, regardless of which subaddress received those funds, and subaddresses only represent the turnstile through which a TXO enters the wallet. This provides definitive clarity on the counterparty who sent the transaction.

With the `require_spend_subaddress` restriction enabled, when you build a transaction, you pass a `spend_subaddress`, which is the subaddress from which to source the funds.

This API endpoint enables a `require_spend_subaddress` mode on the account, enforcing that all build transactions must provide this parameter. If this parameter is not enabled, the wallet provider may still optionally restrict spent TXOs to come from a single subaddress in the transaction.

## Request

| Required Param             | Purpose                                                        | Requirements                     |
| -------------------------- | -------------------------------------------------------------- | -------------------------------- |
| `account_id`               | The account on which to enable `require_spend_subaddress`      | Account must exist in the wallet |
| `require_spend_subaddress` | Restricts spending to require a given subaddress to spend from | `bool`                           |

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
