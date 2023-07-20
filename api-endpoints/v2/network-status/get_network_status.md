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
  "method": "get_network_status",
  "result": {
    "network_status": {
      "network_block_height": "1769514",
      "local_block_height": "1769514",
      "local_num_txos": "5367111",
      "fees": {
        "0": "400000000",
        "1": "2560",
        "8192": "2560"
      },
      "block_version": "3",
      "max_tombstone_blocks": "20160",
      "network_info": {
        "offline": false,
        "chain_id": "",
        "peers": [
          "mc://node1.test.mobilecoin.com/",
          "mc://node2.test.mobilecoin.com/"
        ],
        "tx_sources": [
          "https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/",
          "https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/"
        ]
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
