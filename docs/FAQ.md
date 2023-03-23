---
description: Frequently Asked Questions
---

# FAQ

## What is the precision of MOB?

The atomic unit for MOB is picoMOB, which is 1e-12. You need u64 to represent MOB, and many frameworks, DBs, and languages top out at u32 or i64. This is why Full-Service json responses are all strings. For i64 issues there is technically no loss of precision, but you need to cast back to u64 when fetching data.



## How can I make importing of accounts faster?
When calling [`import_account`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/account/account/import_account), scanning starts at the Origin block (#1). 
By using the **first_block_index** param, a user can specify the starting block for that account if it is known. Newly created accounts are given a starting block index. Any transactions that occurred before the first_block_index will not be scanned, potentially leading to invalid balances or denied transactions.



## How-to-tell-if-full-service-is-communicating-with-the-blockchain?
Using the [`get_network_status`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/network-status/get_network_status) API is the cleanest way to determine if FullService is successfully able to communicate with the blockchain.
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


## Is there a list or mapping of Tokens?
With integration of [MCIP59](https://github.com/mobilecoinfoundation/mcips/blob/main/text/0059-token-metadata-document.md) there will be a signed, hosted file that contains the mapping and attributes of the tokens present on the MC network. Below table is for reference only.

Below table 
| Token ID  |  Token Name  | Precision  | Approx Fee |
| --------- | ------------ | ---------- | ---------- |
|     0     |     Mob      |Pico (10^-9)|   .0004    |
|     1     |     eUSD     |Micro (10^-6)|  .0025    |
