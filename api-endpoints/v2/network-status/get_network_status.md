---
description: Get the current status of the network.
---

# Get Network Status

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "get_network_status",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_network_status",
  "result":{
    "network_status":{
      "network_block_height":"1352942",
      "local_block_height":"1352942",
      "fees":{
        "0":"400000000",
        "1":"2560",
        "8192":"2560"
      },
      "block_version":"2"
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
