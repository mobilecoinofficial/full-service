---
description: Get TXOs for a given account with offset and limit parameters
---

# Get TXOs

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L40)

| Optional Param             | Purpose                                                                                                  | Requirements                      |
|----------------------------|----------------------------------------------------------------------------------------------------------|-----------------------------------|
| `account_id`               | The account on which to perform this action.                                                             | Account must exist in the wallet. |
| `address`                  | The address b58 on which to perform this action.                                                         | Address must exist in the wallet. |
| `status`                   | Txo status filer. Available status: "unverified", "unspent", "spent", "orphaned", "pending", "secreted", |                                   |
| `min_received_block_index` | The minimum block index to query for received txos, inclusive                                            |                                   |
| `max_received_block_index` | The maximum block index to query for received txos, inclusive                                            |                                   |
| `offset`                   | The pagination offset. Results start at the offset index.                                                |                                   |
| `limit`                    | Limit for the number of results.                                                                         |                                   |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}

```
{
  "method": "get_txos",
  "params": {
    "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
    "min_received_block_index": "128567",
    "max_received_block_index": "128567",
    "offset": 2,
    "limit": 8
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}

{% tab title="Response" %}

```
{
  "method":"get_txos",
  "result":{
    "txo_ids":[
      "fa9b95605688898f2d6bca52fb39608bd80eca74a342e3033f6dc0eef1c4e542",
      "454c511ddab33edccc4b686b67d1f9a6c4eb101c28386e0f4e21c994ea35aa2f",
      "490dd001d240f9c9fddcbeef0790eaf78d5732fa96e3e11a3f7dd94e994eeb84"
    ],
    "txo_map":{
      "fa9b95605688898f2d6bca52fb39608bd80eca74a342e3033f6dc0eef1c4e542":{
        "id":"fa9b95605688898f2d6bca52fb39608bd80eca74a342e3033f6dc0eef1c4e542",
        "value":"1234600000000",
        "token_id":"0",
        "received_block_index":"1352857",
        "spent_block_index":null,
        "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
        "status":"unspent",
        "target_key":"0a20269a33ccc56fc4e17e8be492638d4160e2a20e067e876f31068ae8ac7ddcda73",
        "public_key":"0a2052d89fc3cfd035bf2162f03bbf44139613fab7151d7cddc6d0ef44910edbd975",
        "e_fog_hint":"0a542ed0bc15d1395f54a4077c7a08f1eb0e713aab700a18fe55a66419c6b5abf250e0be3098f317b1f948e8f3c5dcfe800870c8150f386f732a850e4ef8bd2fd0bd5124ad78f2d4799fbb1e0e82a05d6002e2c30100",
        "subaddress_index":"0",
        "key_image":"0a204269ee4734aa5fa9b029b39fdb39f0f493c538aaefabb7e6290be56e402a2174",
        "confirmation":"0a20bf46e135b4eeb5c45fcc8ee69a5e564469fd3985269010f6738a96f832992afe"
      },
      "454c511ddab33edccc4b686b67d1f9a6c4eb101c28386e0f4e21c994ea35aa2f":{
        "id":"454c511ddab33edccc4b686b67d1f9a6c4eb101c28386e0f4e21c994ea35aa2f",
        "value":"1234600000000",
        "token_id":"0",
        "received_block_index":"1352852",
        "spent_block_index":null,
        "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
        "status":"unspent",
        "target_key":"0a20807982b36edcb72ff9b61630ffdb7949bfe1778798708f80d6c349fc0672e011",
        "public_key":"0a20728e73bd8675562ab44dea5c2b0edd4bfdf037a73d4afd42267442337c60f73b",
        "e_fog_hint":"0a54c4f56d207fbd86401c6f8bcf2dfb344aba7f8f8dcf542da046c92ed62f9582b281068872044ca71b8c70e9a8c5b3e2c134fb36a570293ceff55d3555eb8710fbb6635cc58242ff9b2383ae832881dca8698f0100",
        "subaddress_index":"0",
        "key_image":"0a2062a2cd3c08f4ff68f46ce7ba50afb8e1e441ac21e5e9d3ae9f7016c89a2cac23",
        "confirmation":"0a20013e277d63f9223f37dace93974b5cff87257b7d413d66638155af89345016d0"
      },
      "490dd001d240f9c9fddcbeef0790eaf78d5732fa96e3e11a3f7dd94e994eeb84":{
        "id":"490dd001d240f9c9fddcbeef0790eaf78d5732fa96e3e11a3f7dd94e994eeb84",
        "value":"1929999999798",
        "token_id":"0",
        "received_block_index":"1352847",
        "spent_block_index":null,
        "account_id":"b504409093f5707d63f24c9ce64ca461101478757d691f2e949fa2d87a35d02c",
        "status":"unspent",
        "target_key":"0a20dce16f8febe0dc3b4d7f7e47755030fe7204b03eb070b4dd0ccb6aead37a4d33",
        "public_key":"0a20ea6f11280167408088cff1aa92eecf8e3268bc2480609681f07569c1ac5e8c79",
        "e_fog_hint":"0a544df6b851a3d7684e0a88e9b1be2b1df9b69afc9af44310bed49750c5482cc860d87cdc7eb18187807f9288e43b5ce9aad1b08ba7472d639f1abbe1ab3e713cb6f509367787687bcf9e2a1b72ed0e944e42c70100",
        "subaddress_index":"0",
        "key_image":"0a206683694c35b4126247079d31811a7ff0b5fe7e5a5412b85a20b9d48595788321",
        "confirmation":"0a2057cdcc39a113f09e40968e9b5ff18060288bf55c5be16b4e798370a8432f64dc"
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}
{% endtabs %}
