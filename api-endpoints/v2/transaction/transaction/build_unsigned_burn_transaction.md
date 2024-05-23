---
description: Build a unsigned burn transaction for use with the offline transaction signer
---

# Build Unsigned Burn Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L67-L74)

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

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L52-L56)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_unsigned_burn_transaction",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "amount": {
      "value": "240800000000",
      "token_id": "0"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "build_unsigned_burn_transaction",
  "result": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "unsigned_tx_proposal": {
      "unsigned_tx_proto_bytes_hex": "0ab882010ae77d0a9f0212220a2...",
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
          "tx_out_proto": "32370a220a208eb51f1e69837e1c8a25fedc6d0e19ec1cd526a2b25facebd0a4cb030b67632e119111fe2872edd82f1a089c44ec6a10be84b412220a20e4a4f5e597d9c4b216b98ef9529a896138775d796cf843cee4e9327532f2fd7a1a220a2070c79d6285ca7f609a7262f031b5a29ed15587dfc081150caba7a13e4f3eaf5c22560a54b2a7d7f5b2d5de1f914946534d1abdbcfb56412ae8dd490de2f1ee6639dc3f580d8ece7b917b9924c8185211a0619120a17cabc1ec7a32a40434193b878b5511044de4603aa779deddc645e61bebd4019f1301002a440a42bfa3ff3a08615c02f0422cbb9e6241946c53e4a85c5372190c87c5e295b9b186ac05e94af94dc78183da92b99219a19503196feb00dc8a758b8d06bc1ca85583856b",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "3cn4Y8V6p5u51z8AEEQsdUvFWcQKYwv25q6SaXeiXyz8kp19g7rLkuxu6rgefYWdZzun2RNrVPsMkM4djfhNzxC8LKKFmZXptcsxqndvbd9",
          "confirmation_number": "1629ef37dc7a8c4707a7536804b17c4dfc2bf85c9a3a72b76e9a60791100b7ba"
        }
      ],
      "change_txos": [
        {
          "tx_out_proto": "32370a220a20c6e884ef3aff93c56df5c2c554607fab17eb82da5e26e13104f6875c6ac91457116e1245de9a4fafee1a083afd153c7f5fa8d212220a20188d035a5f109c48a90c5dd0dee94182b1ee7eb9dfa62b4af13222633025f8501a220a2032488f4e82c1f16a3f6ff37b591bee9eb0c46119aaf1a545d65138568c6e937922560a54337662b1120c5a88d123f9a4e6590ab6862809cc41789cc7d2bd7201c2c6e3fd045d081ce8abd4c1436b491772f6d0cf6a146c8b80a9e5d08579ffaa585352694914f9a307c92e68499eb105ea09909bfebb01002a440a42eae1a5351d63cd5914aff62c77a41db89c0a5a0d4f07343e59e7b314313fde26fd8a04f9f7cd372566d0ed47245b10caec6334c2445d19887dd59d7dbce66f8c6f9d",
          "amount": {
            "value": "758400000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8",
          "confirmation_number": "a28024cf2b224173ab2f54e55a4a246dc88a0c92ae1efcfada3332572ef68a5a"
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
  "method": "build_unsigned_burn_transaction",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "amount": {
      "value": "240800000000",
      "token_id": "0"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endhint %}
