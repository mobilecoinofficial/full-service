---
description: >-
  Submit a tx_proposal to the ledger that adds the gift code to the wallet_db
  once the tx_proposal has been appended to the ledger.
---

# submit\_gift\_code

## Parameters

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid b58-encoded gift code. |
| `from_account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `tx_proposal` | Transaction proposal to submit. | Created with`build_gift_code.` |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "submit_gift_code",
  "params": {
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
    "tx_proposal": '$(cat test-tx-proposal.json)',
    "from_account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "submit_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "Happy Birthday!",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

