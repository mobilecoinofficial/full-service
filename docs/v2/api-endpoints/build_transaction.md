---
description: >-
  Build a transaction to confirm its contents before submitting it to the
  network.
---

# Build Transaction

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L56-L66)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action | Account must exist in the wallet |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `addresses_and_amounts` | An array of public addresses and [Amounts](../../../full-service/src/json_rpc/v2/models/amount.rs) as a tuple | addresses are b58-encoded public addresses |
| `recipient_public_address` | The recipient for this transaction | b58-encoded public address bytes |
| `amount` | The [Amount](../../../full-service/src/json_rpc/v2/models/amount.rs) to send in this transaction |  |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction | TXO IDs \(obtain from `get_txos_for_account`\) |
| `fee_value` | The fee value to submit with this transaction | If not provided, uses `MINIMUM_FEE` of the first outputs token_id, if available, or defaults to MOB |
| `fee_token_id` | The fee token_id to submit with this transaction | If not provided, uses token_id of first output, if available, or defaults to MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 10 |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L48-51)

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
  "method": "build_transaction",
  "result": {
    "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
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
  }
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
