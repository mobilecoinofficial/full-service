---
description: Get all the gift codes currently in the database.
---

# Get Gift Codes

## Parameters

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `offset` | The pagination offset. Results start at the offset index | |
| `limit` | Limit for the number of results | |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "get_gift_codes",
    "jsonrpc": "2.0",
    "id": 1,
    "params": {}
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_gift_codes",
  "result": {
    "gift_codes": [
      {
        "object": "gift_code",
        "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
        "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
        "value_pmob": "80000000000",
        "memo": "Happy New Year!",
        "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
        "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
      },
      {
        "object": "gift_code",
        "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY",
        "entropy": "14aa16d9d4000628c82826d9c43bbc17414f8677e74882bf21e44db75d4c2b87",
        "value_pmob": "20000000000",
        "memo": "Happy Birthday!",
        "account_id": "dba3d3b99fe9ce6bc666490b8176be91ace0f4166853b0327ea39928640ea840",
        "txo_id": "ab917ed9e69fa97bd9422452b1a2f615c2405301b220f7a81eb091f75eba3f54"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

