---
description: Get the balance of a given account.
---

# Get Balance

## Get Balance

### Request

| Required Param | Purpose                           | Requirements                      |
| -------------- | --------------------------------- | --------------------------------- |
| `account_id`   | The account to check balance for. | Account must exist in the wallet. |

### Response

### Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_balance",
  "params": {
     "account_id": "b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_balance",
  "result":{
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
    },
    "account_block_height": "1747773",
    "network_block_height": "1747773"
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
