---
description: >-
  Build a burn transaction to confirm its contents before submitting it to the
  network.
---

# Build Burn Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L56-L66)

| Required Param | Purpose                                     | Requirements                     |
| -------------- | ------------------------------------------- | -------------------------------- |
| `account_id`   | The account on which to perform this action | Account must exist in the wallet |

| Optional Param        | Purpose                                                                                                                                               | Requirements                                                                                                                                                                        |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `amount`              | The [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) to send in this transaction |                                                                                                                                                                                     |
| `redemption_memo_hex` | An external protocol dependent value that allows the entity responsible for the burn to claim credit                                                  |                                                                                                                                                                                     |
| `input_txo_ids`       | Specific TXOs to use as inputs to this transaction                                                                                                    | TXO IDs (obtain from `get_txos_for_account`)                                                                                                                                        |
| `fee_value`           | The fee value to submit with this transaction                                                                                                         | If not provided, uses `MINIMUM_FEE` of the first outputs token\_id, if available, or defaults to MOB                                                                                |
| `fee_token_id`        | The fee token\_id to submit with this transaction                                                                                                     | If not provided, uses token\_id of first output, if available, or defaults to MOB                                                                                                   |
| `tombstone_block`     | The block after which this transaction expires                                                                                                        | If not provided, uses `cur_height` + 10                                                                                                                                             |
| `block_version`       | string(u64)                                                                                                                                           | The block version to build this transaction for. Defaults to the network block version                                                                                              |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction                                                                                     |                                                                                                                                                                                     |
| `spend_subaddress`    | string                                                                                                                                                | If specified, the subaddress from which to spend funds. If sufficient funds are not availble that have been received only at this subaddress, an InsufficientFunds error is thrown. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L48-51)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_burn_transaction",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
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
  "method": "build_burn_transaction",
  "result": {
    "tx_proposal": {
      "input_txos": [
        {
          "tx_out_proto": "32370a220a200c21e6de963668c8dfa356efdb6b58c476d8d0e25ab22f9dff1998ecad412c421111021f3fdff6b5e41a086b0e4e4d26c5a9dc12220a20743844a7c483b7b64eb40ee44edef6ff7d484b2a7b03f44d4e8011ccccab8d051a220a203076178eb5375ee208fc84cfe607d246469d8ee620496e6549257205ef3cbd7522560a5418058053e57fec618c40e7ad8ee4874399fefb59750a789e056dacccdcfbed9087c1f2e5b52799e8cea6155f3bac6d6d60d9c53f7d35e7daefee87d4e74bb74eb023ff6a5a4e05e596b9354671a26ebc716701002a440a423389a34769820bcaf7accbd66b2a96d02a07a18e8786abaf1baab67589f5b217c0964c27c7b0237ad1f48e7049b41f19fdf4f0fb7951b9755a8dc72de601b75faf35",
          "amount": {
            "value": "999600000000",
            "token_id": "0"
          },
          "subaddress_index": "18446744073709551614",
          "key_image": "2a68cd87e3492e84ef609dbe9bd1587f7099220cedc73f1bfcab99638352f806"
        }
      ],
      "payload_txos": [
        {
          "tx_out_proto": "32370a220a205cbf3636d105f34e3ed1d20f21a5a031a963cd39c185865651d324cd72fd5713116b7def6eb76f24a71a087c82f57bd66b404f12220a208274f05caf2e5440994af2e19cf0d337f98d7da4245580335da362c25e801f4c1a220a204a9ddb9347d55bd22459b3cbb7cc861b86447dd682733f61ff853bcb0429fb6222560a54d2bdde5b4878ff1dc012c128522eecbb5733b47ed7d53d443cc17009856e91429fde681f6289c38645371a53450307d2f42c80e7c7dd197ba399bb71b68843e6a752fbc50bf3828425a5d8eb6e6b63283eac01002a440a4266ec2a3896b23a2b9a21c74ee5b04d68f20e2ba37a9ba8f36495f55148e564a679ea4d4a6cd8e93da59e43388dd25d9b61f35d5afa1fcd80c33465868c3bba2139f7",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "3cn4Y8V6p5u51z8AEEQsdUvFWcQKYwv25q6SaXeiXyz8kp19g7rLkuxu6rgefYWdZzun2RNrVPsMkM4djfhNzxC8LKKFmZXptcsxqndvbd9",
          "confirmation_number": "2271d6753951926eb9a8caa7a46363be6a2dcd21e451f71bc52c6dd0e8de52f9"
        }
      ],
      "change_txos": [
        {
          "tx_out_proto": "32370a220a200a8f925a4d72e23fb2cdfc04bc6f75842e7c17e9da97d5786b0157fcfe7cb40211bcdde3a91df6053b1a08ac28b6e1d4c4392112220a208afecccb7c7a4934d8879995b9407bb46cc46ff93a23d32f910e6e43f7fb906c1a220a20cce568654604eb34bdc3794908b166ee577bd7ae0702c22c8165be4b2dd1de7822560a54ee8390c86c31fad75c5ba97ad1b7680a300fb69cec0d0f3fe54e50920256601f479bc00a6f161146850e1fbe600296f6c68e3e7962572572579f6bd461596929b7366173e93a4bc2eda8574267fcbfb251fd01002a440a424f0b7dae785c97238c7fe99f849cd0ebb9ff66bc9ea827bbb3e9082886992cbbc7a90d0c19eced01dc8a2ba7bdebeacb1af385a228f360b7ed8d2299232f5709d454",
          "amount": {
            "value": "758400000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8",
          "confirmation_number": "360abfc745d094ea3bf052b4712d43aa5957ce3785a8e7694fa4553c886d411a"
        }
      ],
      "fee_amount": {
        "value": "400000000",
        "token_id": "0"
      },
      "tombstone_block_index": "1769559",
      "tx_proto": "0ae681010a957d0a9f020a370a220a208e10d14..."
    },
    "transaction_log_id": "aff47d0eba40c2a4e63c68d47e3e0a6b7e29e9e84159b760c9f027ac72c8c602"
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
  "method": "build_burn_transaction",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "amount": { "value": "240800000000", "token_id": "0" }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endhint %}
