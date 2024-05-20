---
description: >-
  Sending a transaction is a convenience method that first builds and then
  submits a transaction.
---

# Build And Submit Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L44-L55)

### Required Params

| Param        | Type   | Description                                                            |
| ------------ | ------ | ---------------------------------------------------------------------- |
| `account_id` | string | The account on which to perform this action. Must exist in the wallet. |

### Optional Params

| Param                                     | Type                                                                                                                               | Description                                                                                                                                                                         |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `addresses_and_amounts`                   | (string, [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs))\[] | An array of public addresses and Amount object tuples                                                                                                                               |
| `recipient_public_address`                | string                                                                                                                             | b58-encoded public address bytes of the recipient for this transaction.                                                                                                             |
| `amount`                                  | [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs)              | The Amount to send in this transaction                                                                                                                                              |
| `input_txo_ids`                           | string\[]                                                                                                                          | Specific TXOs to use as inputs to this transaction                                                                                                                                  |
| `fee_value`                               | string(u64)                                                                                                                        | The fee value to submit with this transaction. If not provided, uses `MINIMUM_FEE` of the first outputs token\_id, if available, or defaults to MOB                                 |
| `fee_token_id`                            | string(u64)                                                                                                                        | The fee token to submit with this transaction. If not provided, uses token\_id of first output, if available, or defaults to MOB                                                    |
| `tombstone_block`                         | string(u64)                                                                                                                        | The block after which this transaction expires. If not provided, uses `cur_height` + 10                                                                                             |
| `max_spendable_value`                     | string(u64)                                                                                                                        | The maximum amount for an input TXO selected for this transaction                                                                                                                   |
| `block_version`                           | string(u64)                                                                                                                        | The block version to build this transaction for. Defaults to the network block version                                                                                              |
| `sender_memo_credential_subaddress_index` | string(u64)                                                                                                                        | The subaddress to generate the SenderMemoCredentials from. Defaults to the default subaddress for the account.                                                                      |
| `payment_request_id`                      | string(u64)                                                                                                                        | The payment request id to set in the RTH Memo.                                                                                                                                      |
| `comment`                                 | string                                                                                                                             | Comment to annotate this transaction in the transaction log                                                                                                                         |
| `spend_only_from_subaddress`              | string                                                                                                                             | If specified, the subaddress from which to spend funds. If sufficient funds are not availble that have been received only at this subaddress, an InsufficientFunds error is thrown. |

\##[Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L44-L47)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_and_submit_transaction",
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
  "method": "build_and_submit_transaction",
  "result": {
    "transaction_log": {
      "id": "0eb20db5c176928fd9a9a4678eaa982ca1d5d7b34c1013148a40e538f28a34cd",
      "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "input_txos": [
        {
          "txo_id": "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d",
          "txo_id_hex": "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          }
        },
        {
          "txo_id": "a47a4f72efe0ec3234608d4671586b0c8777a1104fe6ee23050eefee76060496",
          "txo_id_hex": "a47a4f72efe0ec3234608d4671586b0c8777a1104fe6ee23050eefee76060496",
          "amount": {
            "value": "1000000000000",
            "token_id": "0"
          }
        }
      ],
      "output_txos": [
        {
          "txo_id": "8344dd2ec3470241c8f721e08aa5ace49befa9d9934c7063bd326eb1a0f2355c",
          "txo_id_hex": "8344dd2ec3470241c8f721e08aa5ace49befa9d9934c7063bd326eb1a0f2355c",
          "public_key": "0230b9782c6016ee830e618efc4a1316f374296015bb738821803b0508a3433a",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778"
        }
      ],
      "change_txos": [
        {
          "txo_id": "7d548234542dcc2ebd1691c694663539cca9a2a469eb7a58361d92a97bde3d24",
          "txo_id_hex": "7d548234542dcc2ebd1691c694663539cca9a2a469eb7a58361d92a97bde3d24",
          "public_key": "3076178eb5375ee208fc84cfe607d246469d8ee620496e6549257205ef3cbd75",
          "amount": {
            "value": "999600000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8"
        }
      ],
      "value_map": {
        "0": "240800000000"
      },
      "fee_amount": {
        "value": "400000000",
        "token_id": "0"
      },
      "submitted_block_index": "1769546",
      "tombstone_block_index": "1769556",
      "finalized_block_index": null,
      "status": "pending",
      "sent_time": null,
      "comment": ""
    },
    "tx_proposal": {
      "input_txos": [
        {
          "tx_out_proto": "32370a220a20d007cecbca4ea2a2b6d0fb0d95470470a75c07d6c4496a9c8a065d170b07fd3a1111766004d70ac6271a084927c0f6ce61b60012220a20c441698796282c4147dc3f7e6ec07c9479ed8dbfcafb71929adc97bafb38080e1a220a20aadf8bd1437b52177d290c33ce5602e63ba3efc0cc006cb55545d333cded9f0b22560a544df264ab0cb8c9490774fe6242cd2e2951dafe92f976e0ec84698d39b5ce1b19c68be029c5aed4c327fde66917e8e907a19643c1c3d37bca5b0a460b59829d236f2aeafaa924184cbd4637b0af8dd408885e01002a440a424eb126875a5430560d942da3995aea5fc1a4feb68b19d8742519894d38bf6ccd50f4b321a7816ff5ce651971380af7e2b943a17c35a9a34280ee85b1b04617a41ee8",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "subaddress_index": "18446744073709551614",
          "key_image": "9ec60610aa59dee42f74e7d3b9b513614d0720a0147f91c521a841a02130f835"
        },
        {
          "tx_out_proto": "32370a220a20a89c6f4c83af39feb575b62f47670187b8d7f76a5905c71d9dfff107cfe4f46b1106ce580ea83a77151a0807c8e05cd3f3b0a512220a20c0d7fc63f261a1b80195e7a9ae1b57bf656576b543d212286f7ad626a551cf401a220a2004577594e19440b50bb54dba7adef76c10628a01eabb0421891aa6f412cd402f22560a54622cff894d0c58709d57d0194247416444057dbdf9200e66164b21aefae0d7b5f100782962d4d849a343753de03d7840e11d008eadbff942bc4ca3d349c5d0a9ed4ebc3ba35de70020796e6bfee44dfbea5301002a440a42936e5fcd0b09c265830ddc5e78c89f7f56a02e92396dccb648d5bbf6e4c18f28083baa7b149860e80dde8a84183b5f9ca7321a295d7ca01f8022f3b0056e6394b4a9",
          "amount": {
            "value": "1000000000000",
            "token_id": "0"
          },
          "subaddress_index": "0",
          "key_image": "1c04d3295901c80cf01c78864c693b46439b1a68abbb6df09fe751b629833827"
        }
      ],
      "payload_txos": [
        {
          "tx_out_proto": "32370a220a20e4126c6fd3bd62251ff99bebc1fe4f764c6959a749e176ceb9711f38a392991b117554f33b6f7b1a051a08fca4ed47066fab4712220a20acdfa71be800d5454e25c73c3f17cfa22002750cd4a04a48aedd2f98cc5934121a220a200230b9782c6016ee830e618efc4a1316f374296015bb738821803b0508a3433a22560a541796639b2da610406ac42f8bdafa0f24a00f6287b836df3f779cdc2b87e0aabda91074b4cfd0648971b2740b87077d5f953c396e664522103892f496f38d28711f2bea56234fc1c171e2e1239c4c84dc689001002a440a426e16e76a9fa89800f3b67b3897d2851a9f49d8a864af7bec98b1eda4882ac0915b0c8226c3d398484d8bad00ac0569f83129854ee6888fb61c8917306d810b91c249",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778",
          "confirmation_number": "38720ad584e5a74de613f09ab063b3a48681446032a5f9248ac7ff2d490418da"
        }
      ],
      "change_txos": [
        {
          "tx_out_proto": "32370a220a200c21e6de963668c8dfa356efdb6b58c476d8d0e25ab22f9dff1998ecad412c421111021f3fdff6b5e41a086b0e4e4d26c5a9dc12220a20743844a7c483b7b64eb40ee44edef6ff7d484b2a7b03f44d4e8011ccccab8d051a220a203076178eb5375ee208fc84cfe607d246469d8ee620496e6549257205ef3cbd7522560a5418058053e57fec618c40e7ad8ee4874399fefb59750a789e056dacccdcfbed9087c1f2e5b52799e8cea6155f3bac6d6d60d9c53f7d35e7daefee87d4e74bb74eb023ff6a5a4e05e596b9354671a26ebc716701002a440a423389a34769820bcaf7accbd66b2a96d02a07a18e8786abaf1baab67589f5b217c0964c27c7b0237ad1f48e7049b41f19fdf4f0fb7951b9755a8dc72de601b75faf35",
          "amount": {
            "value": "999600000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8",
          "confirmation_number": "a4731755c93da4bb22c07564099c575277c34a0f886cfd2a87b3d62eda7813fd"
        }
      ],
      "fee_amount": {
        "value": "400000000",
        "token_id": "0"
      },
      "tombstone_block_index": "1769556",
      "tx_proto": "0a9ff8010af8790acf010a2d0a220a20ac5fa52e722893ea60....."
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
`If an account is not fully-synced, you may see the following error message:`

```
{
  "error": "Connection(Operation { error: TransactionValidation(ContainsSpentKeyImage), total_delay: 0ns, tries: 1 })"
}
```

Call `check_balance` for the account, and note the `synced_blocks` value. If that value is less than the `local_block_height` value, then your TXOs may not all be updated to their spent status.
{% endhint %}
