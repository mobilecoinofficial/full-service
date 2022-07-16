---
description: >-
  This is a convenience method for building a transaction that will split a txo
  into multiple output txos to the origin account.
---

# Build Split Txo Transaction

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `txo_id` | The TXO on which to perform this action | TXO must exist in the wallet |
| `output_values` | The output values of the generated TXOs |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `destination_subaddress_index` |  |  |
| `fee_value` |  |  |
| `fee_token_id` |  |  |
| `tombstone_block` |  |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "build_split_txo_transaction",
  "params": {
    "txo_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "output_values": ["1000000000", "1000000000", "1240275839", "1257284399532"],
    "destination_subaddress_index": "4",
    "fee": "1000000000",
    "tombstone_block": "150000"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "build_split_txo_transaction",
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

{% hint style="warning" %}
`If an account is not fully-synced, you may see the following error message:`

```text
{
  "error": "Connection(Operation { error: TransactionValidation(ContainsSpentKeyImage), total_delay: 0ns, tries: 1 })"
}
```

Call `check_balance` for the account, and note the `synced_blocks` value. If that value is less than the `local_block_height` value, then your TXOs may not all be updated to their spent status.
{% endhint %}

