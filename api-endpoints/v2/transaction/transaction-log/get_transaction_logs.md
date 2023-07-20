# Get Transaction Logs

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Optional Param    | Purpose                                                   | Requirement                        |
| ----------------- | --------------------------------------------------------- | ---------------------------------- |
| `account_id`      | The account id to scan for transaction logs               | Account must exist in the database |
| `min_block_index` | The minimum block index to find transaction logs from     |                                    |
| `max_block_index` | The maximum block index to find transaction logs from     |                                    |
| `offset`          | The pagination offset. Results start at the offset index. |                                    |
| `limit`           | Limit for the number of results.                          |                                    |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_transaction_logs",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "offset": 0,
    "limit": 10
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_transaction_logs",
  "result": {
    "transaction_log_ids": [
      "0eb20db5c176928fd9a9a4678eaa982ca1d5d7b34c1013148a40e538f28a34cd",
      "daf0c1439633d1d53a13b9bf086946032c20bef882d5bd7735b4a99816c24657",
      "204f812f021c0ef3d8f2121926f3e98dadae248bc7c43002d7d76c383809ba25",
      "aff47d0eba40c2a4e63c68d47e3e0a6b7e29e9e84159b760c9f027ac72c8c602"
    ],
    "transaction_log_map": {
      "0eb20db5c176928fd9a9a4678eaa982ca1d5d7b34c1013148a40e538f28a34cd": {
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
        "finalized_block_index": "1769546",
        "status": "succeeded",
        "sent_time": null,
        "comment": ""
      },
      "daf0c1439633d1d53a13b9bf086946032c20bef882d5bd7735b4a99816c24657": {
        "id": "daf0c1439633d1d53a13b9bf086946032c20bef882d5bd7735b4a99816c24657",
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "input_txos": [
          {
            "txo_id": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
            "txo_id_hex": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
            "amount": {
              "value": "470400000000",
              "token_id": "0"
            }
          }
        ],
        "output_txos": [
          {
            "txo_id": "245669e1ced312bfe5a1a7e99c77918acf7bb5b4e69eb21d8ef74961b8dcc07e",
            "txo_id_hex": "245669e1ced312bfe5a1a7e99c77918acf7bb5b4e69eb21d8ef74961b8dcc07e",
            "public_key": "167628bd36b6c70aed289cdb3d61d22eb4b40a48f304c484a8f8de781ab54565",
            "amount": {
              "value": "229200000000",
              "token_id": "0"
            },
            "recipient_public_address_b58": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778"
          }
        ],
        "change_txos": [
          {
            "txo_id": "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d",
            "txo_id_hex": "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d",
            "public_key": "aadf8bd1437b52177d290c33ce5602e63ba3efc0cc006cb55545d333cded9f0b",
            "amount": {
              "value": "240800000000",
              "token_id": "0"
            },
            "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8"
          }
        ],
        "value_map": {
          "0": "229200000000"
        },
        "fee_amount": {
          "value": "400000000",
          "token_id": "0"
        },
        "submitted_block_index": "1769541",
        "tombstone_block_index": "1769546",
        "finalized_block_index": "1769541",
        "status": "succeeded",
        "sent_time": null,
        "comment": ""
      },
      "204f812f021c0ef3d8f2121926f3e98dadae248bc7c43002d7d76c383809ba25": {
        "id": "204f812f021c0ef3d8f2121926f3e98dadae248bc7c43002d7d76c383809ba25",
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "input_txos": [
          {
            "txo_id": "6603da459bb4bb88ddfaff2ac702238de4198b67b37e93d789e7311f8978641b",
            "txo_id_hex": "6603da459bb4bb88ddfaff2ac702238de4198b67b37e93d789e7311f8978641b",
            "amount": {
              "value": "1000000000000",
              "token_id": "0"
            }
          }
        ],
        "output_txos": [
          {
            "txo_id": "c5b1f16523c9f8c8e8a043ba5b974bd38049035eed8cedbec021a6c8c537f5a2",
            "txo_id_hex": "c5b1f16523c9f8c8e8a043ba5b974bd38049035eed8cedbec021a6c8c537f5a2",
            "public_key": "685f9bc824abacab084aa9aa686e6c21fc0ad0c7552443c5b06cc3b4169ac236",
            "amount": {
              "value": "529200000000",
              "token_id": "0"
            },
            "recipient_public_address_b58": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778"
          }
        ],
        "change_txos": [
          {
            "txo_id": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
            "txo_id_hex": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
            "public_key": "cecc879afd79153210ff79b58947416a883d4f68253d415533c0e8898e09f045",
            "amount": {
              "value": "470400000000",
              "token_id": "0"
            },
            "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8"
          }
        ],
        "value_map": {
          "0": "529200000000"
        },
        "fee_amount": {
          "value": "400000000",
          "token_id": "0"
        },
        "submitted_block_index": "1769533",
        "tombstone_block_index": "1769539",
        "finalized_block_index": "1769533",
        "status": "succeeded",
        "sent_time": null,
        "comment": ""
      },
      "aff47d0eba40c2a4e63c68d47e3e0a6b7e29e9e84159b760c9f027ac72c8c602": {
        "id": "aff47d0eba40c2a4e63c68d47e3e0a6b7e29e9e84159b760c9f027ac72c8c602",
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "input_txos": [
          {
            "txo_id": "7d548234542dcc2ebd1691c694663539cca9a2a469eb7a58361d92a97bde3d24",
            "txo_id_hex": "7d548234542dcc2ebd1691c694663539cca9a2a469eb7a58361d92a97bde3d24",
            "amount": {
              "value": "999600000000",
              "token_id": "0"
            }
          }
        ],
        "output_txos": [
          {
            "txo_id": "38b17285a2b9d31deee4a903962f8c5607f3c71ef7bb5e8947c0a9b5b32f95fc",
            "txo_id_hex": "38b17285a2b9d31deee4a903962f8c5607f3c71ef7bb5e8947c0a9b5b32f95fc",
            "public_key": "4a9ddb9347d55bd22459b3cbb7cc861b86447dd682733f61ff853bcb0429fb62",
            "amount": {
              "value": "240800000000",
              "token_id": "0"
            },
            "recipient_public_address_b58": "3cn4Y8V6p5u51z8AEEQsdUvFWcQKYwv25q6SaXeiXyz8kp19g7rLkuxu6rgefYWdZzun2RNrVPsMkM4djfhNzxC8LKKFmZXptcsxqndvbd9"
          }
        ],
        "change_txos": [
          {
            "txo_id": "21787db374127da3a1fe25e91106a7319892f320e440dc6fa47f9c31f28ebc90",
            "txo_id_hex": "21787db374127da3a1fe25e91106a7319892f320e440dc6fa47f9c31f28ebc90",
            "public_key": "cce568654604eb34bdc3794908b166ee577bd7ae0702c22c8165be4b2dd1de78",
            "amount": {
              "value": "758400000000",
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
        "submitted_block_index": null,
        "tombstone_block_index": "1769559",
        "finalized_block_index": null,
        "status": "failed",
        "sent_time": null,
        "comment": ""
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
