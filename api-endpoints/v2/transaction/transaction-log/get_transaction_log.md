# Get Transaction Log

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param       | Purpose                        | Requirement                               |
| -------------------- | ------------------------------ | ----------------------------------------- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_transaction_log",
  "params": {
    "transaction_log_id": "daf0c1439633d1d53a13b9bf086946032c20bef882d5bd7735b4a99816c24657"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_transaction_log",
  "result": {
    "transaction_log": {
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
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
