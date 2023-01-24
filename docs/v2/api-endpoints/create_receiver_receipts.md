---
description: >-
  After building a tx_proposal, you can get the receipts for that transaction
  and provide it to the recipient so they can poll for the transaction status.
---

# Create Receiver Receipts

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `tx_proposal` |  |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method":"create_receiver_receipts",
  "params":{
    "tx_proposal":{
      "input_txos":[
        {
          "tx_out_proto":"0a370a220a20648f47e11467e58e66db1bf8aef625248dae8da988eadf2852a065e1d64...",
          "amount":{
            "value":"5999600000000",
            "token_id":"0"
          },
          "subaddress_index":"18446744073709551614",
          "key_image":"f6e1c9e28d98278cddf0e16b4885f95b7d5aa6e791689920d7e1ed724bc79d0d"
        }
      ],
      "payload_txos":[
        {
          "tx_out_proto":"0a370a220a20b0eec3e4ac1605ebd32e74598ce8ae6a7730e6b159918df2d77537d5f349...",
          "amount":{
            "value":"1234600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
          "confirmation_number":"013e277d63f9223f37dace93974b5cff87257b7d413d66638155af89345016d0"
        }
      ],
      "change_txos":[
        {
          "tx_out_proto":"0a370a220a20c29cbaee8f6e1e824bf3e4a010a4a4479b61432082c890fc7481ddecff....",
          "amount":{
            "value":"4764600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h",
          "confirmation_number":"c1a3d0ced6b25dbd1d9110aeb7e99ba899129fcc5d7064fcc3a8626b245ae7e5"
        }
      ],
      "fee_amount":{
        "value":"400000000",
        "token_id":"0"
      },
      "tombstone_block_index":"1352860",
      "tx_proto":"0a9c7b0acb760a9f020a370a220a20124eb6e51b173727d7b51126d72f1f77fcd7ebc0655ba..."
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"create_receiver_receipts",
  "result":{
    "receiver_receipts":[
      {
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
    ]
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

