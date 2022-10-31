---
description: 'Get the current status of the network.'
---

# Get The Network Status

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "get_network_status",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_network_status",
  "result": {
    "network_status": {
      object: "network_status",
      "network_block_height": "152918",
      "local_block_height": ""152918,
        "fees": {
            "0": "400000000",
            "1": "2560"
        },

    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

