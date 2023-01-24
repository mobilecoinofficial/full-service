---
description: >-
  Build a unsigned burn transaction for use with the offline transaction signer
---

# Build Unsigned Transaction

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L67-L74)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action | Account must exist in the wallet |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `amount` | The [Amount](../../../full-service/src/json_rpc/v2/models/amount.rs) to send in this transaction |  |
| `redemption_memo_hex` | An external protocol dependent value that allows the entity responsible for the burn to claim credit |  |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction | TXO IDs \(obtain from `get_txos_for_account`\) |
| `fee_value` | The fee value to submit with this transaction | If not provided, uses `MINIMUM_FEE` of the first outputs token_id, if available, or defaults to MOB |
| `fee_token_id` | The fee token_id to submit with this transaction | If not provided, uses token_id of first output, if available, or defaults to MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 10 |
| `block_version` | string(u64) | The block version to build this transaction for. Defaults to the network block version |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L52-L56)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_unsigned_burn_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
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
  "method":"build_unsigned_burn_transaction",
  "result":{
    "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
    "unsigned_tx_proposal":{
      "unsigned_tx_proto_bytes_hex":"0ac77b0af6760acf0112220a20642814347652a3b1307cc5887e....",
      "unsigned_input_txos":[
        {
          "tx_out_proto":"0a370a220a20742a88da7cc2652473c159e22c037da4c9758f356c9968c1acd5fe2a2...",
          "amount":{
            "value":"3529600000000",
            "token_id":"0"
          },
          "subaddress_index":"18446744073709551614"
        }
      ],
      "payload_txos":[
        {
          "tx_out_proto":"0a370a220a20046cabffdf4d01d9391edd411b250c986a138205801975f2d548d39e0...",
          "amount":{
            "value":"12346000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"3cn4Y8V6p5u51z8AEEQsdUvFWcQKYwv25q6SaXeiXyz8kp19g7rLkuxu6rgefYWdZzun2RNrVPsMkM4djfhNzxC8LKKFmZXptcsxqndvbd9",
          "confirmation_number":"c9cdd9b7d5dcc753d1ec977512a963410e8c5a6c1efa497ab0ef25fac37110f6"
        }
      ],
      "change_txos":[
        {
          "tx_out_proto":"0a370a220a201494b66804787f13c526f5fa79d209908097f03a5fd5c5c0bdb6c7a3...",
          "amount":{
            "value":"3529187654000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h",
          "confirmation_number":"28fd8cfa53dc9f1246179206e70521866d02fecfb81201f9567f039bf13bb8a3"
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
  "method": "build_unsigned_burn_transaction",
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