---
description: Check the status of a receiver receipt.
---

# Check Receiver Receipt Status

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L78)

| Required Param     | Purpose                                    | Requirements                     |
|--------------------|--------------------------------------------|----------------------------------|
| `address`          | The account's public address.              | Must be a valid account address. |
| `receiver_receipt` | The receipt whose status is being checked. |                                  |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L61)

## Example

{% tabs %}
{% tab title="Request Body" %}

```
{
  "method":"check_receiver_receipt_status",
  "params":{
    "address":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
    "receiver_receipt":{
      "public_key":"0a20728e73bd8675562ab44dea5c2b0edd4bfdf037a73d4afd42267442337c60f73b",
      "confirmation":"0a20013e277d63f9223f37dace93974b5cff87257b7d413d66638155af89345016d0",
      "tombstone_block":"1352860",
      "amount":{
        "commitment":"b0eec3e4ac1605ebd32e74598ce8ae6a7730e6b159918df2d77537d5f349e43c",
        "masked_value":"15435919858782335364",
        "masked_token_id":"3dcdec57c18e114b",
        "version":"V1"
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}

{% tab title="Response" %}

```
{
  "method":"check_receiver_receipt_status",
  "result":{
    "receipt_transaction_status":"TransactionSuccess",
    "txo":{
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
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}
{% endtabs %}
