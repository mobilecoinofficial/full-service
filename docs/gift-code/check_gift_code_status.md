---
description: 'Check the status of a gift code, which may be pending, available, or claimed.'
---

# check\_gift\_code\_status

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid b58-encoded gift code. |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "check_gift_code_status",
  "params": {
    "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeAvailable",
    "gift_code_value": 100000000,
    "gift_code_memo": "Happy Birthday!"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}

{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeSubmittedPending",
    "gift_code_value": null
    "gift_code_memo": "",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

