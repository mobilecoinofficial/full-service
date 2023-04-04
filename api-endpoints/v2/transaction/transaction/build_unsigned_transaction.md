---
description: Build a unsigned transaction for use with the offline transaction signer
---

# Build Unsigned Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L67-L74)

| Required Param | Purpose                                     | Requirements                     |
| -------------- | ------------------------------------------- | -------------------------------- |
| `account_id`   | The account on which to perform this action | Account must exist in the wallet |

| Optional Param             | Purpose                                                                                                                                                            | Requirements                                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------- |
| `addresses_and_amounts`    | An array of public addresses and [Amounts](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) as a tuple | addresses are b58-encoded public addresses                                                           |
| `recipient_public_address` | The recipient for this transaction                                                                                                                                 | b58-encoded public address bytes                                                                     |
| `amount`                   | The [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) to send in this transaction              |                                                                                                      |
| `input_txo_ids`            | Specific TXOs to use as inputs to this transaction                                                                                                                 | TXO IDs (obtain from `get_txos_for_account`)                                                         |
| `fee_value`                | The fee value to submit with this transaction                                                                                                                      | If not provided, uses `MINIMUM_FEE` of the first outputs token\_id, if available, or defaults to MOB |
| `fee_token_id`             | The fee token\_id to submit with this transaction                                                                                                                  | If not provided, uses token\_id of first output, if available, or defaults to MOB                    |
| `tombstone_block`          | The block after which this transaction expires                                                                                                                     | If not provided, uses `cur_height` + 10                                                              |
| `block_version`            | string(u64)                                                                                                                                                        | The block version to build this transaction for. Defaults to the network block version               |
| `max_spendable_value`      | The maximum amount for an input TXO selected for this transaction                                                                                                  |                                                                                                      |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L52-L56)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_unsigned_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "amount": { "value": "42000000000000", "token_id": "0" },
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"build_unsigned_transaction",
  "result":{
    "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
    "unsigned_tx_proposal":{
      "unsigned_tx_proto_bytes_hex":"0acc7a0afb750acf0112220a20748912ab4cf0b50b2a83340433fd8232319b....",
      "unsigned_input_txos":[
        {
          "tx_out_proto":"0a370a220a20742a88da7cc2652473c159e22c037da4c9758f356c9968c1acd5fe2a246......",
          "amount":{
            "value":"3529600000000",
            "token_id":"0"
          },
          "subaddress_index":"18446744073709551614"
        }
      ],
      "payload_txos":[
        {
          "tx_out_proto":"0a370a220a20566f25a667250236cfcf1352a1671fb3d035e3258e29200b270f4c5471f.....",
          "amount":{
            "value":"123460000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
          "confirmation_number":"3b9fd9d6debd56ad7a5cc62539478a94a0c946cfd9c5c996eeddc2142886ffb5"
        }
      ],
      "change_txos":[
        {
          "tx_out_proto":"0a370a220a202ceba8e05b56eddd22f4f15fa7bc6abd78cdee77b4796e7ec9389318a16....",
          "amount":{
            "value":"3405740000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h",
          "confirmation_number":"38a7aa0a882364327c293b7ff1c1cc9b67d64131ebad3b0a7c93ff91c0976a68"
        }
      ]
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

{% hint style="info" %}
Since the `tx_proposal`JSON object is quite large, you may wish to write the result to a file for use in the `submit_transaction` call, such as:

```
{
  "method": "build_unsigned_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value": ["42000000000000", "0"]
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endhint %}
