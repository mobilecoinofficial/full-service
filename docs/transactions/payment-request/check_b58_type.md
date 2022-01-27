---
description: Check the type of the b58 code
---

# Check B58 Type

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `b58_code` | The code to check | `String` |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "check_b58_type",
  "params": {
    "b58_code": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "check_b58_code",
  "result": {
    "b58_type": "PaymentRequest",
    "data": {
      "value": "1000000000000",
      "public_address_b58": "4BfAQbahn9Bs8on7RrWkpargtVUiGNnLrbsmCVFyeqFHHATbwV4CRtjQvhhzpyrkbWBU2HqWK8Fg6boZ235YLEzkGJNFBEVGTKAnCN6vNGV",
      "memo": "testing testing"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

