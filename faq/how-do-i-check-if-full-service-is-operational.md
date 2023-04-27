# How do I check if Full-Service is operational?

Using the [`get_network_status`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/network-status/get\_network\_status) API is the cleanest way to determine if FullService is successfully able to communicate with the blockchain.&#x20;



### Sucess

A successful response to [`get_network_status`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/network-status/get\_network\_status) will look like this:

```
"method": "get_network_status",
    "result": {
        "network_status": {
            "network_block_height": "1482767",
            "local_block_height": "1482767",
            "local_num_txos": "4445078",
            "fees": {
                "0": "400000000",
                "1": "2560"
            },
            "block_version": "3",
            "max_tombstone_blocks": "20160",
            "network_info": {
                "offline": false,
                "chain_id": "main",
                "peers": [
                    "mc://node1.prod.mobilecoinww.com/",
                    "mc://node2.prod.mobilecoinww.com/"
                ],
                "tx_sources": [
                    "https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/",
                    "https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com/"
                    ]
```



### Error States

In the situation that Full Service is unable to decipher blocks after a mobilecoin-core update, the reponse will look like this:

```
"method": "get_network_status",
    "error": {
        "code": -32603,
        "message": "InternalError",
        "data": {
            "server_error": "NetworkBlockHeight(BlockVersion(UnsupportedBlockVersion(3, 2)))",
            "details": "Error getting network block height: Block version: Unsupported block version: 3 > 2. Try upgrading your software"
        }
    }
```
