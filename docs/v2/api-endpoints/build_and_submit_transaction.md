---
description: >-
  Sending a transaction is a convenience method that first builds and then
  submits a transaction.
---

# Build And Submit Transaction

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

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
| `input_txo_ids` | string[]] | Specific TXOs to use as inputs to this transaction |
| `fee_value` | string(u64) | The fee value to submit with this transaction. If not provided, uses `MINIMUM_FEE` of the first outputs token_id, if available, or defaults to MOB |
| `fee_token_id` | string(u64) | The fee token to submit with this transaction. If not provided, uses token_id of first output, if available, or defaults to MOB |
| `tombstone_block` | string(u64) | The block after which this transaction expires. If not provided, uses `cur_height` + 10 |
| `max_spendable_value` | string(u64) | The maximum amount for an input TXO selected for this transaction |
| `comment` | string | Comment to annotate this transaction in the transaction log |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

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
  "method": "build_and_submit_transaction",
  "result": {
    "transaction_log": {
      "id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
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
      "comment": "",
      "failure_code": null,
      "failure_message": null
    },
    "tx_proposal": {
      "input_txos": [
        "tx_out_proto": "439f9843vmtbgdrv5...",
        "value": "10000000000",
        "token_id": "0",
        "key_image": "dfj42v03mn40c353v53vvjyh5tr...",
      ],
      "payload_txos": [
        "tx_out_proto": "vr243095b89nvrimwec...",
        "value": "5000000000",
        "token_id": "0",
        "recipient_public_address_b58": "ewvr3m49350c932emr3cew2...",
      ],
      "change_txos": [
        "tx_out_proto": "defvr34v5t4b6b...",
        "value": "4060000000",
        "token_id": "0",
        "recipient_public_address_b58": "n23924mtb89vck31...",
      ]
      "fee": "40000000",
      "fee_token_id": "0",
      "tombstone_block_index": "152700",
      "tx_proto": "328fi4n94902cmjinrievn49jg9439nvr3v..."
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
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

