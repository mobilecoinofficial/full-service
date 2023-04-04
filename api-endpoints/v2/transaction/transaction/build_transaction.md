---
description: >-
  Build a transaction to confirm its contents before submitting it to the
  network.
---

# Build Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L56-L66)

| Required Param | Purpose                                     | Requirements                     |
| -------------- | ------------------------------------------- | -------------------------------- |
| `account_id`   | The account on which to perform this action | Account must exist in the wallet |

| Optional Param                            | Purpose                                                                                                                                                            | Requirements                                                                                                   |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| `addresses_and_amounts`                   | An array of public addresses and [Amounts](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) as a tuple | addresses are b58-encoded public addresses                                                                     |
| `recipient_public_address`                | The recipient for this transaction                                                                                                                                 | b58-encoded public address bytes                                                                               |
| `amount`                                  | The [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) to send in this transaction              |                                                                                                                |
| `input_txo_ids`                           | Specific TXOs to use as inputs to this transaction                                                                                                                 | TXO IDs (obtain from `get_txos_for_account`)                                                                   |
| `fee_value`                               | The fee value to submit with this transaction                                                                                                                      | If not provided, uses `MINIMUM_FEE` of the first outputs token\_id, if available, or defaults to MOB           |
| `fee_token_id`                            | The fee token\_id to submit with this transaction                                                                                                                  | If not provided, uses token\_id of first output, if available, or defaults to MOB                              |
| `tombstone_block`                         | The block after which this transaction expires                                                                                                                     | If not provided, uses `cur_height` + 10                                                                        |
| `block_version`                           | string(u64)                                                                                                                                                        | The block version to build this transaction for. Defaults to the network block version                         |
| `sender_memo_credential_subaddress_index` | string(u64)                                                                                                                                                        | The subaddress to generate the SenderMemoCredentials from. Defaults to the default subaddress for the account. |
| `payment_request_id`                      | string(u64)                                                                                                                                                        | The payment request id to set in the RTH Memo.                                                                 |
| `max_spendable_value`                     | The maximum amount for an input TXO selected for this transaction                                                                                                  |                                                                                                                |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L48-51)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_transaction",
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
  "method":"build_transaction",
  "result":{
    "tx_proposal":{
      "input_txos":[
        {
          "tx_out_proto":"0a2d0a220a20ea91069208a5c017b84254c27f828adbb8...",
          "amount":{
            "value":"7929999999798",
            "token_id":"0"
          },
          "subaddress_index":"0",
          "key_image":"8ef8c4f8f4580a61516073a2e029f5bd6e776188556232e9fcea7b2e5f43bf18"
        }
      ],
      "payload_txos":[
        {
          "tx_out_proto":"0a370a220a20d6067be6475351ea719d3c5f9c026d26d1...",
          "amount":{
            "value":"929999999798",
            "token_id":"0"
          },
          "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
          "confirmation_number":"0e7de1ff74132c9bd6ae7951dabd656b1b0a1317e8b34bc6ec08d0b7d74e8aa1"
        }
      ],
      "change_txos":[
        {
          "tx_out_proto":"0a370a220a20307a1ea3b33ae13a3b23492e0f638a5d41c...",
          "amount":{
            "value":"6999600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h",
          "confirmation_number":"1cceb521744d44047caa7c0849877df8ccb1980d526fc475042eab7e9bb137da"
        }
      ],
      "fee_amount":{
        "value":"400000000",
        "token_id":"0"
      },
      "tombstone_block_index":"1352830",
      "tx_proto":"0aa27b0ad1760acf010a2d0a220a20427e72e004a81f0253d529a..."
    },
    "transaction_log_id":"830d59e6562562df0791b9434cb2cda867c5387e0d89bd4b487929ec764182e3"
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
