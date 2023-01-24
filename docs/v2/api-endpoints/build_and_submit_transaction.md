---
description: >-
  Sending a transaction is a convenience method that first builds and then
  submits a transaction.
---

# Build And Submit Transaction

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L44-L55)

### Required Params
| Param | Type | Description |
| :--- | :--- | :--- |
| `account_id` | string | The account on which to perform this action. Must exist in the wallet. | 

### Optional Params
| Param | Type | Description |
| :--- | :--- | :--- |
| `addresses_and_amounts` | (string, [Amount](../../../full-service/src/json_rpc/v2/models/amount.rs))[] | An array of public addresses and Amount object tuples |
| `recipient_public_address` | string | b58-encoded public address bytes of the recipient for this transaction. |
| `amount` | [Amount](../../../full-service/src/json_rpc/v2/models/amount.rs) | The Amount to send in this transaction |
| `input_txo_ids` | string[] | Specific TXOs to use as inputs to this transaction |
| `fee_value` | string(u64) | The fee value to submit with this transaction. If not provided, uses `MINIMUM_FEE` of the first outputs token_id, if available, or defaults to MOB |
| `fee_token_id` | string(u64) | The fee token to submit with this transaction. If not provided, uses token_id of first output, if available, or defaults to MOB |
| `tombstone_block` | string(u64) | The block after which this transaction expires. If not provided, uses `cur_height` + 10 |
| `max_spendable_value` | string(u64) | The maximum amount for an input TXO selected for this transaction |
| `block_version` | string(u64) | The block version to build this transaction for. Defaults to the network block version |
| `sender_memo_credential_subaddress_index` | string(u64) | The subaddress to generate the SenderMemoCredentials from. Defaults to the default subaddress for the account. |
| `comment` | string | Comment to annotate this transaction in the transaction log |

##[Response](../../../full-service/src/json_rpc/v2/api/response.rs#L44-L47)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "build_and_submit_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "amount": { "value": "42000000000000", "token_id": "0" }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method":"build_and_submit_transaction",
  "result":{
    "transaction_log":{
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
      "finalized_block_index":null,
      "status":"pending",
      "sent_time":null,
      "comment":""
    },
    "tx_proposal":{
      "input_txos":[
        {
          "tx_out_proto":"0a370a220a20c29cbaee8f6e1e824bf3e4a010a4a4479b61432082c890fc7481dde...",
          "amount":{
            "value":"4764600000000",
            "token_id":"0"
          },
          "subaddress_index":"18446744073709551614",
          "key_image":"1c091d59f09c7efe6e48662f810b29d4ed4308911726e001a964fbf8e251b25a"
        }
      ],
      "payload_txos":[
        {
          "tx_out_proto":"0a370a220a20d0b53678573da663d793d6b4a6827deb53f67db9ac6a5e6148d8351b...",
          "amount":{
            "value":"1234600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk",
          "confirmation_number":"bf46e135b4eeb5c45fcc8ee69a5e564469fd3985269010f6738a96f832992afe"
        }
      ],
      "change_txos":[
        {
          "tx_out_proto":"0a370a220a20742a88da7cc2652473c159e22c037da4c9758f356c9968c1acd5fe2a24...",
          "amount":{
            "value":"3529600000000",
            "token_id":"0"
          },
          "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h",
          "confirmation_number":"b16ca36fd390bd73a09667c0795806ced1739046b0f5d2bef5040b40d22760d1"
        }
      ],
      "fee_amount":{
        "value":"400000000",
        "token_id":"0"
      },
      "tombstone_block_index":"1352867",
      "tx_proto":"0ac27c0af1770a9f020a370a220a208c737bdd608ca413e3b22489c04d9b20bdde45ee64c2cb31296bc..."
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
`If an account is not fully-synced, you may see the following error message:`

```text
{
  "error": "Connection(Operation { error: TransactionValidation(ContainsSpentKeyImage), total_delay: 0ns, tries: 1 })"
}
```

Call `check_balance` for the account, and note the `synced_blocks` value. If that value is less than the `local_block_height` value, then your TXOs may not all be updated to their spent status.
{% endhint %}

