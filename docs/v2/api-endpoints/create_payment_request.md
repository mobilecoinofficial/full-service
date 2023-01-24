---
description: Create a payment request b58 code to give to someone else
---

# Create Payment Request

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L86)

| Required Param | Purpose | Requirements |
|:---------------| :--- | :--- |
| `account_id`   | The account on which to perform this action. | Account must exist in the wallet. |
| `amount` | The [Amount](../../../full-service/src/json_rpc/v2/models/amount.rs) to send in this transaction |  |

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
    "account_id": "d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
    "amount": { "value": "1234600000000", "token_id": "0" },
    "subaddress_index": 1,
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
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

