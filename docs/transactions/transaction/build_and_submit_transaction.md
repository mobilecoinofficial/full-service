---
description: >-
  Sending a transaction is a convenience method that first builds and then
  submits a transaction.
---

# build\_and\_submit\_transaction

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action | Account must exist in the wallet |
| `recipient_public_address` | The recipient for this transaction | b58-encoded public address bytes |
| `value_pmob` | The amount of MOB to send in this transaction |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction | TXO IDs \(obtain from `get_all_txos_for_account`\) |
| `fee` | The fee amount to submit with this transaction | If not provided, uses `MINIMUM_FEE` = .01 MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 50 |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |
| `comment` | Comment to annotate this transaction in the transaction log |  |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "build_and_submit_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value_pmob": "42000000000000"
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
      "object": "transaction_log",
      "transaction_log_id": "937f102052500525ff0f54aa4f7d94234bd824260bfd7ba40d0561166dda7780",
      "direction": "tx_direction_sent",
      "is_sent_recovered": null,
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "assigned_address_id": null,
      "value_pmob": "42000000000000",
      "fee_pmob": "10000000000",
      "submitted_block_index": "152948",
      "finalized_block_index": null,
      "status": "tx_status_pending",
      "input_txo_ids": [
        "8432bb4e25f1bde68e4759b27ec72d290252cb99943f2f38a9035dba230895b7"
      ],
      "output_txo_ids": [
        "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb"
      ],
      "change_txo_ids": [
        "44c03ddbccb33e5c37365d7b263568a49e6f608e5e818db604541cc09389b762"
      ],
      "sent_time": "2021-02-28 01:27:52 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null,
      "offset_count": 2199
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

Call `check_balance` for the account, and note the `synced_blocks` value. If that value is less than the `local_block_index` value, then your TXOs may not all be updated to their spent status.
{% endhint %}

