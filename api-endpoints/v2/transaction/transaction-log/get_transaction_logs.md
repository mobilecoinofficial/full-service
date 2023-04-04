# Get Transaction Logs

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Optional Param    | Purpose                                                   | Requirement                        |
| ----------------- | --------------------------------------------------------- | ---------------------------------- |
| `account_id`      | The account id to scan for transaction logs               | Account must exist in the database |
| `min_block_index` | The minimum block index to find transaction logs from     |                                    |
| `max_block_index` | The maximum block index to find transaction logs from     |                                    |
| `offset`          | The pagination offset. Results start at the offset index. |                                    |
| `limit`           | Limit for the number of results.                          |                                    |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_transaction_logs",
  "params": {
    "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
    "offset": 2,
    "limit": 1
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"get_transaction_logs",
  "result":{
    "transaction_log_ids":[
      "987c84c38351321572151e3fdd0643f1531fa536c531310bfd4840aed9dd4f75",
      "01cf3c1a5ac2a6b884ef81c1bdd2191a3860d59158118b08f1f8f61ec3e09567",
      "830d59e6562562df0791b9434cb2cda867c5387e0d89bd4b487929ec764182e3",
      "7a26fed20c9d43f2626022b97ba998360ed7da7c82d733cde765e6afd63563c8",
      "4e89d5a3641452d394c13a87aae13a57a836b16104e394f89a5c743b00771b81"
    ],
    "transaction_log_map":{
      "987c84c38351321572151e3fdd0643f1531fa536c531310bfd4840aed9dd4f75":{
        "id":"987c84c38351321572151e3fdd0643f1531fa536c531310bfd4840aed9dd4f75",
        "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
        "input_txos":[
          {
            "txo_id":"34f8a29a2fdd2446694bf175e533c6bf0cd4ecac9d52cd793ef06fc011661b89",
            "amount":{
              "value":"4764600000000",
              "token_id":"0"
            }
          }
        ],
        "output_txos":[
          {
            "txo_id":"fa9b95605688898f2d6bca52fb39608bd80eca74a342e3033f6dc0eef1c4e542",
            "public_key":"52d89fc3cfd035bf2162f03bbf44139613fab7151d7cddc6d0ef44910edbd975",
            "amount":{
              "value":"1234600000000",
              "token_id":"0"
            },
            "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk"
          }
        ],
        "change_txos":[
          {
            "txo_id":"63bc8d402b68241a1274162420607f0040523e0973cf1d6cb50fa0e5156dac1a",
            "public_key":"2096698ed95eb52caa4932e73085efa9f74adafdbf48001019882e1484714f3b",
            "amount":{
              "value":"3529600000000",
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
        "submitted_block_index":"1352857",
        "tombstone_block_index":"1352867",
        "finalized_block_index":"1352857",
        "status":"succeeded",
        "sent_time":null,
        "comment":""
      },
      "01cf3c1a5ac2a6b884ef81c1bdd2191a3860d59158118b08f1f8f61ec3e09567":{
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
      },
      "830d59e6562562df0791b9434cb2cda867c5387e0d89bd4b487929ec764182e3":{
        "id":"830d59e6562562df0791b9434cb2cda867c5387e0d89bd4b487929ec764182e3",
        "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
        "input_txos":[
          {
            "txo_id":"6c3699b850ebbef8cb36b579b5e53b8b235e3f74943ab4d7fe9048a99926ede0",
            "amount":{
              "value":"7929999999798",
              "token_id":"0"
            }
          }
        ],
        "output_txos":[
          {
            "txo_id":"5822eb19ba672761693d2c4d33d3582e74e387f8257d16e58d04b858816c4247",
            "public_key":"3032638202641633b143236f6ebc3deeb37b35e0f792ea092bbbb0f914e6f006",
            "amount":{
              "value":"929999999798",
              "token_id":"0"
            },
            "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk"
          }
        ],
        "change_txos":[
          {
            "txo_id":"bc9ed2b93ab96504f8faf2a30aa39afee43e7fe59b6722df049b8746b8e2e54b",
            "public_key":"329263983954fce9d68284ca031089f5c706dff4d8d51144444a4b335041bf1c",
            "amount":{
              "value":"6999600000000",
              "token_id":"0"
            },
            "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h"
          }
        ],
        "value_map":{
          "0":"929999999798"
        },
        "fee_amount":{
          "value":"400000000",
          "token_id":"0"
        },
        "submitted_block_index":null,
        "tombstone_block_index":"1352830",
        "finalized_block_index":null,
        "status":"failed",
        "sent_time":null,
        "comment":""
      },
      "7a26fed20c9d43f2626022b97ba998360ed7da7c82d733cde765e6afd63563c8":{
        "id":"7a26fed20c9d43f2626022b97ba998360ed7da7c82d733cde765e6afd63563c8",
        "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
        "input_txos":[
          {
            "txo_id":"6c3699b850ebbef8cb36b579b5e53b8b235e3f74943ab4d7fe9048a99926ede0",
            "amount":{
              "value":"7929999999798",
              "token_id":"0"
            }
          }
        ],
        "output_txos":[
          {
            "txo_id":"490dd001d240f9c9fddcbeef0790eaf78d5732fa96e3e11a3f7dd94e994eeb84",
            "public_key":"ea6f11280167408088cff1aa92eecf8e3268bc2480609681f07569c1ac5e8c79",
            "amount":{
              "value":"1929999999798",
              "token_id":"0"
            },
            "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk"
          }
        ],
        "change_txos":[
          {
            "txo_id":"fa737a8e65e480fc7f75dbc17e6875b75cf4b14f3cde02b49b8cd8921fdf7dbb",
            "public_key":"c487a4e3dc82fbfed9546e50e1d631711f6ffc42546ebef0b9e1d79c088ace33",
            "amount":{
              "value":"5999600000000",
              "token_id":"0"
            },
            "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h"
          }
        ],
        "value_map":{
          "0":"1929999999798"
        },
        "fee_amount":{
          "value":"400000000",
          "token_id":"0"
        },
        "submitted_block_index":null,
        "tombstone_block_index":"1352857",
        "finalized_block_index":"1352847",
        "status":"succeeded",
        "sent_time":null,
        "comment":""
      },
      "4e89d5a3641452d394c13a87aae13a57a836b16104e394f89a5c743b00771b81":{
        "id":"4e89d5a3641452d394c13a87aae13a57a836b16104e394f89a5c743b00771b81",
        "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
        "input_txos":[
          {
            "txo_id":"63bc8d402b68241a1274162420607f0040523e0973cf1d6cb50fa0e5156dac1a",
            "amount":{
              "value":"3529600000000",
              "token_id":"0"
            }
          }
        ],
        "output_txos":[
          {
            "txo_id":"4631f7f4466a523544d579918f2bf878a94a741219a4e2496c05c4a68a319dc3",
            "public_key":"5ebaa9d4b5e8e666b162614f031d2263b3df68c40015220fafc62a882d397c19",
            "amount":{
              "value":"12346000",
              "token_id":"0"
            },
            "recipient_public_address_b58":"3cn4Y8V6p5u51z8AEEQsdUvFWcQKYwv25q6SaXeiXyz8kp19g7rLkuxu6rgefYWdZzun2RNrVPsMkM4djfhNzxC8LKKFmZXptcsxqndvbd9"
          }
        ],
        "change_txos":[
          {
            "txo_id":"d51075b42a9210280702f19786d205ba9b5cc24a3bbde53499abad5ecc1dcb70",
            "public_key":"841b9626db8c3046ce34bb74b8cc629ce9332d560198a5a513d0ea9a3a007a64",
            "amount":{
              "value":"3529187654000",
              "token_id":"0"
            },
            "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h"
          }
        ],
        "value_map":{
          "0":"12346000"
        },
        "fee_amount":{
          "value":"400000000",
          "token_id":"0"
        },
        "submitted_block_index":null,
        "tombstone_block_index":"1352876",
        "finalized_block_index":null,
        "status":"failed",
        "sent_time":null,
        "comment":""
      }
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}
