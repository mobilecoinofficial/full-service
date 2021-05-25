---
description: >-
  Submit a transaction for an account with or without recording it in the
  transaction log.
---

# Submit Transaction

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `tx_proposal` | Transaction proposal to submit | Created with `build_transaction` |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | Account ID for which to log the transaction. If omitted, the transaction is not logged. |  |
| `comment` | Comment to annotate this transaction in the transaction log |  |

## Examples

### Submit with Log

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "submit_transaction",
  "params": {
    "tx_proposal": '$(cat test-tx-proposal.json)',
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "submit_transaction",
  "result": {
    "transaction_log": {
      "object": "transaction_log",
      "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
      "direction": "tx_direction_sent",
      "is_sent_recovered": null,
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "assigned_address_id": null,
      "value_pmob": "42000000000000",
      "fee_pmob": "10000000000",
      "submitted_block_index": "152950",
      "finalized_block_index": null,
      "status": "tx_status_pending",
      "input_txo_ids": [
        "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d"
      ],
      "output_txo_ids": [
        "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c"
      ],
      "change_txo_ids": [
        "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4"
      ],
      "sent_time": "2021-02-28 01:42:28 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null,
      "offset_count": 2252
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### Submit without Log

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "submit_transaction",
  "params": {
    "tx_proposal": '$(cat test-tx-proposal.json)'
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "submit_transaction",
  "result": {
    "transaction_log": null
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

