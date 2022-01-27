---
description: >-
  A Transaction Log is a record of a MobileCoin transaction that was constructed
  and sent from this wallet, or that was received at an address belonging to an
  account in this wallet.
---

# Transaction Log

Due to the privacy properties of the MobileCoin ledger, transactions are ephemeral. Once they have been created, they only exist until they are validated, and then only the outputs are written to the ledger. For this reason, the Full-service Wallet stores transactions in the `transaction_log` table in order to preserve transaction history.

## Attributes

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | String, value is "transaction\_log" | String representing the object's type. Objects of the same type share the same value. |
| `transaction_log_id` | Integer | Unique identifier for the transaction log. This value is not associated to the ledger. |
| `direction` | String | A string that identifies if this transaction log was sent or received. Valid values are "sent" or "received". |
| `is_sent_recovered` | Boolean | Flag that indicates if the sent transaction log was recovered from the ledger. This value is null for "received" transaction logs. If true, some information may not be available on the transaction log and its TXOs without user input. If true, the fee `receipient_address_id`, fee, and `sent_time` will be null without user input. |
| `account_id` | String | Unique identifier for the assigned associated account. If the transaction is outgoing, this account is from whence the TXO came. If received, this is the receiving account. |
| `recipient_address_id` | String | Unique identifier for the recipient associated account. Only available if direction is "sent". |
| `assigned_address_id` | String | Unique identifier for the assigned associated account. Only available if direction is "received". |
| `value_pmob` | String \(uint64\) | Value in pico MOB associated to this transaction log. |
| `fee_pmob` | String \(uint64\) | Fee in pico MOB associated to this transaction log. Only on outgoing transaction logs. Only available if direction is "sent". |
| `submitted_block_index` | String \(uint64\) | The block index of the highest block on the network at the time the transaction was submitted. |
| `finalized_block_index` | String \(uint64\) | The scanned block block index in which this transaction occurred. |
| `status` | String | String representing the transaction log status. On "sent", valid statuses are "built", "pending", "succeeded", "failed". On "received", the status is "succeeded". |
| `input_txo_ids` | List | A list of the IDs of the TXOs which were inputs to this transaction. |
| `output_txo_ids` | List | A list of the IDs of the TXOs which were outputs of this transaction. |
| `change_txo_ids` | List | A list of the IDs of the TXOs which were change in this transaction. |
| `sent_time` | Timestamp | Time at which sent transaction log was created. Only available if direction is "sent". This value is null if "received" or if the sent transactions were recovered from the ledger \(`is_sent_recovered = true`\). |
| `comment` | String | An arbitrary string attached to the object. |
| `failure_code` | Integer | Code representing the cause of "failed" status. |
| `failure_message` | String | Human parsable explanation of "failed" status. |
| `offset` | Integer | The value to offset pagination requests for `transaction_log` list. Requests will exclude all list items up to and including this object. |

## Example

{% tabs %}
{% tab title="Received" %}
```text
{
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
  "failure_message": null
}
```
{% endtab %}

{% tab title="Sent - Failed" %}
```text
{
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
  "status": "failed",
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
  "comment": "This is an example of a failed sent transaction log of 1.288 MOB and 0.01 MOB fee!",
  "failure_code": 3,
  "failure_message:": "Contains sent key image."
}
```
{% endtab %}

{% tab title="Sent - Success, Recovered" %}
```text
{
  "object": "transaction_log",
  "transaction_log_id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
  "direction": "tx_direction_sent",
  "is_sent_recovered": true,
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
  "failure_message": null
}
```
{% endtab %}
{% endtabs %}

