---
description: >-
  Submit a transaction for an account with or without recording it in the
  transaction log.
---

# Submit Transaction

## Parameters

| Required Param | Purpose                        | Requirements                     |
| -------------- | ------------------------------ | -------------------------------- |
| `tx_proposal`  | Transaction proposal to submit | Created with `build_transaction` |

| Optional Param | Purpose                                                                                                                                                                                                                                                | Requirements |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ |
| `account_id`   | Account ID for which to log the transaction. If omitted, the transaction is not logged and therefor the txos used will not be set to pending, if they exist. This could inadvertently cause an attempt to spend the same txo in multiple transactions. |              |
| `comment`      | Comment to annotate this transaction in the transaction log                                                                                                                                                                                            |              |

## Examples

### Submit with Log

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "submit_transaction",
  "params": {
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
    },
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "submit_transaction",
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
      "failure_message:": null
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
```
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
```
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
