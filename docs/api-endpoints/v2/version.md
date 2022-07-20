---
description: 'Get the version number of the software.'
---

# Get Version Number

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "version",
  "jsonrpc": "2.0",
  "api_version": "2",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "version",
  "result": {
    "string": "1.6.0",
    "number": ["1", "6", "0", ""],
    "commit": "282982fb295dbe0bf6f9df829471055f02606f10"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

