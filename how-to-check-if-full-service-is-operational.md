# How to check if Full-Service is operational

Using the [`get_network_status`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/network-status/get\_network\_status) API is the cleanest way to determine if FullService is successfully able to communicate with the blockchain.&#x20;

For instance, if Full Service is unable to decipher blocks after a mobilecoin-core update, the reponse will look like this:

```
"method": "get_network_status",
    "error": {
        "code": -32603,
        "message": "InternalError",
        "data": {
            "server_error": "NetworkBlockHeight(BlockVersion(UnsupportedBlockVersion(3, 2)))",
            "details": "Error getting network block height: Block version: Unsupported block version: 3 > 2. Try upgrading your software"
        }
    },
    "jsonrpc": "2.0",
    "id": 1
```
