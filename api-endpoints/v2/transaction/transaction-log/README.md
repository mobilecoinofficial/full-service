---
description: >-
  A Transaction Log is a record of a MobileCoin transaction that was constructed
  and sent from this wallet.
---

# Transaction Log

## Transaction Log

Due to the privacy properties of the MobileCoin ledger, transactions are ephemeral. Once they have been created, they
only exist until they are validated, and then only the outputs are written to the ledger. For this reason, the
Full-service Wallet stores outgoing transactions in the `transaction_log` table in order to preserve transaction
history. Received transactions are instead saved as txos.

### Attributes

| _Name_                  | _Type_               | _Description_                                                                                                                                                                                                    |
|-------------------------|----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `id`                    | integer              | Unique identifier for the transaction log. This value is not associated to the ledger.                                                                                                                           |
| `account_id`            | string               | Unique identifier for the assigned associated account. If the transaction is outgoing, this account is from whence the TXO came. If received, this is the receiving account.                                     |
| `value_map`             | map (string, uint64) | Total value per token associated to this transaction log.                                                                                                                                                        |
| `fee_value`             | string (uint64)      | Fee value associated to this transaction log.                                                                                                                                                                    |
| `fee_token_id`          | string (uint64)      | Fee token id associated to this transaction log.                                                                                                                                                                 |
| `submitted_block_index` | string (uint64)      | The block index of the highest block on the network at the time the transaction was submitted.                                                                                                                   |
| `tombstone_block_index` | string (uint64)      | The tombstone block index.                                                                                                                                                                                       |
| `finalized_block_index` | string (uint64)      | The scanned block block index in which this transaction occurred.                                                                                                                                                |
| `status`                | string               | String representing the transaction log status. Valid statuses are "built", "pending", "succeeded", "failed".                                                                                                    |
| `input_txos`            | \[InputTxo]          | A list of the TXOs which were inputs to this transaction.                                                                                                                                                        |
| `payload_txos`          | \[OutputTxo]         | A list of the TXOs which were payloads of this transaction.                                                                                                                                                      |
| `change_txos`           | \[OutputTxo]         | A list of the TXOs which were change in this transaction.                                                                                                                                                        |
| `sent_time`             | Timestamp            | Time at which sent transaction log was created. Only available if direction is "sent". This value is null if "received" or if the sent transactions were recovered from the ledger (`is_sent_recovered = true`). |
| `comment`               | string               | An arbitrary string attached to the object.                                                                                                                                                                      |
| `failure_code`          | integer              | Code representing the cause of "failed" status.                                                                                                                                                                  |
| `failure_message`       | string               | Human parsable explanation of "failed" status.                                                                                                                                                                   |

## Input Txo

### Attributes

| _Name_   | _Type_                                                                                                               | _Description_                  |
|----------|----------------------------------------------------------------------------------------------------------------------|--------------------------------|
| `txo_id` | string                                                                                                               | Unique identifier for the txo. |
| `amount` | [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/models/amount.rs) | Amount of this txo.            |

## Output Txo

### Attributes

| _Name_                         | _Type_                                                                                                               | _Description_                                    |
|--------------------------------|----------------------------------------------------------------------------------------------------------------------|--------------------------------------------------|
| `txo_id`                       | string                                                                                                               | Unique identifier for the txo.                   |
| `amount`                       | [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/models/amount.rs) | Amount of this txo.                              |
| `recipient_public_address_b58` | string                                                                                                               | Public address b58 of the recipient of this txo. |

### Example

{% tabs %}
{% tab title="Sent - Pending" %}

```
{
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
}
```

{% endtab %}

{% tab title="Sent - Failed" %}

```
{
  "id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
  "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
  "value_map": {
    "0": "42000000000000"
  },
  "fee_value": "10000000000",
  "fee_token_id": "0",
  "submitted_block_index": "152950",
  "finalized_block_index": null,
  "status": "failed",
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
  "failure_message:": "Contains spent key image."
}
```

{% endtab %}

{% tab title="Sent - Success" %}

```
{
  "id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
  "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
  "value_map": {
    "0": "42000000000000"
  },
  "fee_value": "10000000000",
  "fee_token_id": "0",
  "submitted_block_index": "152950",
  "finalized_block_index": "152951",
  "status": "succeeded",
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
}
```

{% endtab %}
{% endtabs %}
