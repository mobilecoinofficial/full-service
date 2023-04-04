---
description: >-
  Build a burn transaction to confirm its contents before submitting it to the
  network.
---

# Build Burn Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L56-L66)

| Required Param | Purpose                                     | Requirements                     |
| -------------- | ------------------------------------------- | -------------------------------- |
| `account_id`   | The account on which to perform this action | Account must exist in the wallet |

| Optional Param        | Purpose                                                                                                                                               | Requirements                                                                                         |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `amount`              | The [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) to send in this transaction |                                                                                                      |
| `redemption_memo_hex` | An external protocol dependent value that allows the entity responsible for the burn to claim credit                                                  |                                                                                                      |
| `input_txo_ids`       | Specific TXOs to use as inputs to this transaction                                                                                                    | TXO IDs (obtain from `get_txos_for_account`)                                                         |
| `fee_value`           | The fee value to submit with this transaction                                                                                                         | If not provided, uses `MINIMUM_FEE` of the first outputs token\_id, if available, or defaults to MOB |
| `fee_token_id`        | The fee token\_id to submit with this transaction                                                                                                     | If not provided, uses token\_id of first output, if available, or defaults to MOB                    |
| `tombstone_block`     | The block after which this transaction expires                                                                                                        | If not provided, uses `cur_height` + 10                                                              |
| `block_version`       | string(u64)                                                                                                                                           | The block version to build this transaction for. Defaults to the network block version               |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction                                                                                     |                                                                                                      |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L48-51)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_burn_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "amount": { "value": "42000000000000", "token_id": "0" }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method":"build_burn_transaction",
  "result":{
    "tx_proposal":{
      "input_txos":[
        {
          "tx_out_proto":"0a370a220a20742a88da7cc2652473c159e22c037da4c9758f356c9968c1acd5fe2a246b...",
          "amount":{
            "value":"3529600000000",
            "token_id":"0"
          },
          "subaddress_index":"18446744073709551614",
          "key_image":"d8832af25e21e14f1705cddbc26bea46eff185040444c16d9c3d07053f775f6d"
        }
      ],
      "payload_txos":[
        {
          "tx_out_proto":"0a370a220a2084e41af24f167344ff593ca48456d6aafdab6eb2acc027302ff2a7cbaeaa...",
          "amount":{
            "value":"12346000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"3cn4Y8V6p5u51z8AEEQsdUvFWcQKYwv25q6SaXeiXyz8kp19g7rLkuxu6rgefYWdZzun2RNrVPsMkM4djfhNzxC8LKKFmZXptcsxqndvbd9",
          "confirmation_number":"13cc2b59469e30912d1e417ede7a93ae21d4fafa59605bdf2cb66e648c72dd37"
        }
      ],
      "change_txos":[
        {
          "tx_out_proto":"0a370a220a20621e3912b508a7eb5c2eb6b4cc1a0e306b19bc1429a7c2e938751a19e247...",
          "amount":{
            "value":"3529187654000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h",
          "confirmation_number":"d0f20741f440b9c8a780500f8abc381737d3b9c9d712a6520b9d7806705b031d"
        }
      ],
      "fee_amount":{
        "value":"400000000",
        "token_id":"0"
      },
      "tombstone_block_index":"1352876",
      "tx_proto":"0afc790aab750a9f020a370a220a20742a88da7cc2652473c159e22c037da4c9758f356c9968c1acd5fe...."
    },
    "transaction_log_id":"4e89d5a3641452d394c13a87aae13a57a836b16104e394f89a5c743b00771b81"
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
  "method": "build_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value_pmob": "42000000000000"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endhint %}
