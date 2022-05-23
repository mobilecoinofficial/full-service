---
description: >-
  Build a unsigned transaction for use with the offline transaction signer
---

# Build Unsigned Transaction

## Parameters

| Required Param | Purpose                                     | Requirements                     |
| -------------- | ------------------------------------------- | -------------------------------- |
| `account_id`   | The account on which to perform this action | Account must exist in the wallet |

| Optional Param             | Purpose                                                                                                                                                                                                                            | Requirements                                                 |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `recipient_public_address` | The recipient for this transaction                                                                                                                                                                                                 | b58-encoded public address bytes                             |
| `value_pmob`               | The amount of MOB to send in this transaction                                                                                                                                                                                      |                                                              |
| `addresses_and_values`     | An array of public addresses and value tuples                                                                                                                                                                                      | addresses are b58-encoded public addresses, value is in pmob |
| `input_txo_ids`            | Specific TXOs to use as inputs to this transaction                                                                                                                                                                                 | TXO IDs (obtain from `get_txos_for_account`)             |
| `fee`                      | The fee amount to submit with this transaction                                                                                                                                                                                     | If not provided, uses `MINIMUM_FEE` = .01 MOB                |
| `tombstone_block`          | The block after which this transaction expires                                                                                                                                                                                     | If not provided, uses `cur_height` + 10                      |                                                       |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_unsigned_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value_pmob": "42000000000000",
    "log_tx_proposal": false
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "build_unsigned_transaction",
  "result": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "unsigned_tx": ,
    "fog_resolver" ,
}
```
{% endtab %}
{% endtabs %}

{% hint style="info" %}
Since the `tx_proposal`JSON object is quite large, you may wish to write the result to a file for use in the `submit_transaction` call, such as:

```
{
  "method": "build_unsigned_transaction",
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
