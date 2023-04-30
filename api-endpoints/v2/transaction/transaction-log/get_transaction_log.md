# Get Transaction Log

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param       | Purpose                        | Requirement                               |
|----------------------|--------------------------------|-------------------------------------------|
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}

```
{
  "method": "get_transaction_log",
  "params": {
    "transaction_log_id": "914e703b5b7bc44b61bb3657b4ee8a184d00e87a728e2fe6754a77a38598a800"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}

{% tab title="Response" %}

```
{
  "method":"get_transaction_log",
  "result":{
    "transaction_log":{
      "id":"01cf3c1a5ac2a6b884ef81c1bdd2191a3860d59158118b08f1f8f61ec3e09567",
      "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
      "input_txos":[
        {
          "txo_id":"fa737a8e65e480fc7f75dbc17e6875b75cf4b14f3cde02b49b8cd8921fdf7dbb",
          "amount":{
            "value":"5999600000000",
            "token_id":"0"
          }
        }
      ],
      "output_txos":[
        {
          "txo_id":"454c511ddab33edccc4b686b67d1f9a6c4eb101c28386e0f4e21c994ea35aa2f",
          "public_key":"728e73bd8675562ab44dea5c2b0edd4bfdf037a73d4afd42267442337c60f73b",
          "amount":{
            "value":"1234600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk"
        }
      ],
      "change_txos":[
        {
          "txo_id":"34f8a29a2fdd2446694bf175e533c6bf0cd4ecac9d52cd793ef06fc011661b89",
          "public_key":"3c0225fab2d6df245887b7acebf22c238ffafa54842ab2663ac27833975a2212",
          "amount":{
            "value":"4764600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h"
        }
      ],
      "value_map":{
        "0":"1234600000000"
      },
      "fee_amount":{
        "value":"400000000",
        "token_id":"0"
      },
      "submitted_block_index":"1352852",
      "tombstone_block_index":"1352860",
      "finalized_block_index":"1352852",
      "status":"succeeded",
      "sent_time":null,
      "comment":""
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}
{% endtabs %}
