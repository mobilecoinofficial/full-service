---
description: Build a unsigned transaction for use with the offline transaction signer
---

# Build Unsigned Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L67-L74)

| Required Param | Purpose                                     | Requirements                     |
| -------------- | ------------------------------------------- | -------------------------------- |
| `account_id`   | The account on which to perform this action | Account must exist in the wallet |

| Optional Param               | Purpose                                                                                                                                                            | Requirements                                                                                                                                                                        |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `addresses_and_amounts`      | An array of public addresses and [Amounts](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) as a tuple | addresses are b58-encoded public addresses                                                                                                                                          |
| `recipient_public_address`   | The recipient for this transaction                                                                                                                                 | b58-encoded public address bytes                                                                                                                                                    |
| `amount`                     | The [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) to send in this transaction              |                                                                                                                                                                                     |
| `input_txo_ids`              | Specific TXOs to use as inputs to this transaction                                                                                                                 | TXO IDs (obtain from `get_txos_for_account`)                                                                                                                                        |
| `fee_value`                  | The fee value to submit with this transaction                                                                                                                      | If not provided, uses `MINIMUM_FEE` of the first outputs token\_id, if available, or defaults to MOB                                                                                |
| `fee_token_id`               | The fee token\_id to submit with this transaction                                                                                                                  | If not provided, uses token\_id of first output, if available, or defaults to MOB                                                                                                   |
| `tombstone_block`            | The block after which this transaction expires                                                                                                                     | If not provided, uses `cur_height` + 10                                                                                                                                             |
| `block_version`              | string(u64)                                                                                                                                                        | The block version to build this transaction for. Defaults to the network block version                                                                                              |
| `max_spendable_value`        | The maximum amount for an input TXO selected for this transaction                                                                                                  |                                                                                                                                                                                     |
| `spend_only_from_subaddress` | string                                                                                                                                                             | If specified, the subaddress from which to spend funds. If sufficient funds are not availble that have been received only at this subaddress, an InsufficientFunds error is thrown. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L52-L56)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_unsigned_transaction",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "recipient_public_address": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778",
    "amount": { "value": "240800000000", "token_id": "0" }
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
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "unsigned_tx_proposal": {
      "unsigned_tx_proto_bytes_hex": "0ab682010ae57d0a9f0212220a20a0...",
      "unsigned_input_txos": [
        {
          "tx_out_proto": "32370a220a200c21e6de963668c8dfa356efdb6b58c476d8d0e25ab22f9dff1998ecad412c421111021f3fdff6b5e41a086b0e4e4d26c5a9dc12220a20743844a7c483b7b64eb40ee44edef6ff7d484b2a7b03f44d4e8011ccccab8d051a220a203076178eb5375ee208fc84cfe607d246469d8ee620496e6549257205ef3cbd7522560a5418058053e57fec618c40e7ad8ee4874399fefb59750a789e056dacccdcfbed9087c1f2e5b52799e8cea6155f3bac6d6d60d9c53f7d35e7daefee87d4e74bb74eb023ff6a5a4e05e596b9354671a26ebc716701002a440a423389a34769820bcaf7accbd66b2a96d02a07a18e8786abaf1baab67589f5b217c0964c27c7b0237ad1f48e7049b41f19fdf4f0fb7951b9755a8dc72de601b75faf35",
          "amount": {
            "value": "999600000000",
            "token_id": "0"
          },
          "subaddress_index": "18446744073709551614"
        }
      ],
      "payload_txos": [
        {
          "tx_out_proto": "32370a220a20327c6c28e89dc32a97bb641454d50aa1c85ebe0dfb61847432bdd2880ffef71c11ee31eab27530eebe1a08a63b72141153f76012220a20d0b1e9a35058e75907d3d72e1ee68deff45dbf0b29a315df609c2ddb470589481a220a200037ba765e11f026c7a07a46ccfd9d4233edbe837a50cbb88dcb03261fb2263a22560a54886dddb0c668e0c38a7ed7744648b50d79027ae39efc7983535f52b7bd28e29c413cce4c798bffdb3f3be99bd59ee8be77b973c3922cdfd11ddb97c2609ce3d2f15d2a01b12b806637a8894371ef2ba634fa01002a440a4278a03d210f36423f5cb7c967a5eb196887c1722f2e4b78cafb03bfa9de1786b946ed1dbff8d6ce9a3287c97c0af9ffbaa1d256565a54d81033695f9dd8062bfc8703",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778",
          "confirmation_number": "9272192398ebdb67c4575d637f06cc778923e8e02864f93bd9a3e22d46af20dd"
        }
      ],
      "change_txos": [
        {
          "tx_out_proto": "32370a220a200cc33ff46f47e117a13fe888010c08b1809a3f0f285fae2d8bc5237bb25e575e117b253616668d0e721a08abc8a185b46aea7f12220a209ced3359c1ea745ed9112e0da8f566f093530c34f91149030fe3a71d1527d5101a220a20b84add492e9a508825b608ad81e7369119ab9cc858f7db7c955e2a825bd65d0f22560a54aa5082599ed734e3f7d83e4a1d16d6cf383fae823833c417e971768a388d80defc4da4c1c2d60b23e656e9c1f99fc5c36846dfc22f25d96d938d24138eb6d3221d1d20b1b373ce40c659fbdfd95c64d2735701002a440a4225ca1145216874f3ef24f1bda1771139f01dea87d72f07dcc78fe7fa7126103d820c7aa14b28c18d1d83604a49c683743d1f328c16e520bf91f2895f3fa27e7f5b8d",
          "amount": {
            "value": "758400000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8",
          "confirmation_number": "02414acb167825c943957386f47b131dfb8a445be275eb43b7d35eff5070562d"
        }
      ]
    }
  },
  "jsonrpc": "2.0",
  "id": 1
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
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "recipient_public_address": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778",
    "amount": { "value": "240800000000", "token_id": "0" }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endhint %}
