---
description: Get the version number of the software.
---

# Get Version

## Example

{% tabs %}
{% tab title="Request Body" %}

```
{
  "method": "version",
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}

{% tab title="Response" %}

```
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
