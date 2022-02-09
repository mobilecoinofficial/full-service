---
description: Build a gift code in a tx_proposal that you can fund and submit to the ledger.
---

# Build Gift Code

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `value_pmob` | The amount of MOB to send in this transaction. |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `input_txo_ids` | The specific TXOs to use as inputs to this transaction. | TXO IDs \(obtain from `get_all_txos_for_account`\) |
| `fee` | The fee amount to submit with this transaction. | If not provided, uses `MINIMUM_FEE` = .01 MOB. |
| `tombstone_block` | The block after which this transaction expires. | If not provided, uses `cur_height` + 10. |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction. |  |
| `memo` | Memo for whoever claims the gift code. |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "build_gift_code",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "value_pmob": "42000000000000",
    "memo": "Happy Birthday!"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "build_gift_code",
  "result": {
    "tx_proposal": "...",
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

