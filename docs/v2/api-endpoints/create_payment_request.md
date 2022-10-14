---
description: Create a payment request b58 code to give to someone else
---

# Create Payment Request

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L86)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `amount_pmob` | The amount of pMOB to send in this transaction. | `u64` |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `subaddress_index` | The subaddress index on the account to generate the request with | `i64` |
| `memo` | Memo for the payment request |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "create_payment_request",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "amount_pmob": 42000000000000,
    "subaddress_index": 4,
    "memo": "Payment for dinner with family"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "create_payment_request",
  "result": {
    "payment_request_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

