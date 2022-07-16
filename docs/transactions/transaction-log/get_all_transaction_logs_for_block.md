---
description: Get all transaction logs for a given block.
---

# Get All Transaction Logs For Block

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `block_index` | The block on which to perform this action. | Block must exist in the wallet. |

## Example

In the below example, the account in the wallet sent a transaction to itself. Therefore, there is one sent `transaction_log` in the block, and two received \(one for the change, and one for the output TXO sent to the same account that constructed the transaction\).

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_all_transaction_logs_for_block",
  "params": {
    "block_index": "152951"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_all_transaction_logs_for_block",
  "result": {
    "transaction_log_ids": [
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
      "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc",
      "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
    ],
    "transaction_log_map": {
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1": {
        "id": "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
        "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
        "value_map": {
          "0": "42000000000000"
        },
        "fee_value": "10000000000",
        "fee_token_id": "0",
        "submitted_block_index": "152950",
        "finalized_block_index": null,
        "status": "pending",
        "input_txos": [
          {
            "id": "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d",
            "value": "50000000000",
            "token_id": "0"
          }
        ],
        "payload_txos": [
          {
            "id": "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c",
            "value": "30000000000",
            "token_id": "0"
            "recipient_public_address_b58": "vrewh94jfm43m430nmv2084j3k230j3mfm4i3mv39nffrwv43"
          }
        ],
        "change_txos": [
          {
            "id": "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4",
            "value": "10000000000",
            "token_id": "0"
            "recipient_public_address_b58": "grewmvn3990435vm032492v43mgkvocdajcl2icas"
          }
        ],
        "sent_time": "2021-02-28 01:42:28 UTC",
        "comment": "This is an example of a failed sent transaction log of 1.288 MOB and 0.01 MOB fee!",
        "failure_code": 3,
        "failure_message:": "Contains sent key image."
      },
      "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc": {
        "id": "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc",
        "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
        "value_map": {
          "0": "42000000000000"
        },
        "fee_value": "10000000000",
        "fee_token_id": "0",
        "submitted_block_index": "152950",
        "finalized_block_index": null,
        "status": "pending",
        "input_txos": [
          {
            "id": "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d",
            "value": "50000000000",
            "token_id": "0"
          }
        ],
        "payload_txos": [
          {
            "id": "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c",
            "value": "30000000000",
            "token_id": "0"
            "recipient_public_address_b58": "vrewh94jfm43m430nmv2084j3k230j3mfm4i3mv39nffrwv43"
          }
        ],
        "change_txos": [
          {
            "id": "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4",
            "value": "10000000000",
            "token_id": "0"
            "recipient_public_address_b58": "grewmvn3990435vm032492v43mgkvocdajcl2icas"
          }
        ],
        "sent_time": "2021-02-28 01:42:28 UTC",
        "comment": "This is an example of a failed sent transaction log of 1.288 MOB and 0.01 MOB fee!",
        "failure_code": 3,
        "failure_message:": "Contains sent key image."
      },
      "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab":{
        "id": "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab",
        "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
        "value_map": {
          "0": "42000000000000"
        },
        "fee_value": "10000000000",
        "fee_token_id": "0",
        "submitted_block_index": "152950",
        "finalized_block_index": null,
        "status": "pending",
        "input_txos": [
          {
            "id": "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d",
            "value": "50000000000",
            "token_id": "0"
          }
        ],
        "payload_txos": [
          {
            "id": "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c",
            "value": "30000000000",
            "token_id": "0"
            "recipient_public_address_b58": "vrewh94jfm43m430nmv2084j3k230j3mfm4i3mv39nffrwv43"
          }
        ],
        "change_txos": [
          {
            "id": "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4",
            "value": "10000000000",
            "token_id": "0"
            "recipient_public_address_b58": "grewmvn3990435vm032492v43mgkvocdajcl2icas"
          }
        ],
        "sent_time": "2021-02-28 01:42:28 UTC",
        "comment": "This is an example of a failed sent transaction log of 1.288 MOB and 0.01 MOB fee!",
        "failure_code": 3,
        "failure_message:": "Contains sent key image."
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

