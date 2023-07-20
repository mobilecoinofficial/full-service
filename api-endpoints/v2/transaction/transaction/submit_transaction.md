---
description: >-
  Submit a transaction for an account with or without recording it in the
  transaction log.
---

# Submit Transaction

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                        | Requirements                     |
| -------------- | ------------------------------ | -------------------------------- |
| `tx_proposal`  | Transaction proposal to submit | Created with `build_transaction` |

| Optional Param | Purpose                                                                                                                                                                                                                                                | Requirements |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ |
| `account_id`   | Account ID for which to log the transaction. If omitted, the transaction is not logged and therefor the txos used will not be set to pending, if they exist. This could inadvertently cause an attempt to spend the same txo in multiple transactions. |              |
| `comment`      | Comment to annotate this transaction in the transaction log                                                                                                                                                                                            |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Examples

### Submit with Log

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "submit_transaction",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "tx_proposal": {
      "input_txos": [
        {
          "tx_out_proto": "32370a220a20a8b9496bfe9a95a3cfbae1fda980ce2a1fa7e2827da6916de204ae12d094210a11d3771dafeecf86471a083db375a7f674e6cc12220a20bcaa42886171e60c50f0a4527663507a890fbecb5016f6d9042ce6be1cd7fb521a220a20cecc879afd79153210ff79b58947416a883d4f68253d415533c0e8898e09f04522560a54643db209825ced0df98a277c989b9d1876ac4009397137af1fabd3856c7c97dd629be47752cd532aa1f4bb1412d4dac9a76d50e67b4b99da017dc3a40caa99b4933ef6b4b51c56a338fc8648244eba5a22d901002a440a4213d173c60b40cfe99de248d38166f99e5cfcd45327b03dc46b1dd0147e78f4c19f881afe2f56e50da8743597d6eec8c6e44336e606dd235e8b7edca15a5d7a0c9c08",
          "amount": {
            "value": "470400000000",
            "token_id": "0"
          },
          "subaddress_index": "18446744073709551614",
          "key_image": "fafbf66b4da787c3a7d0c6a12d67620efcb47c3299ab4382627e468c718d4d1e"
        }
      ],
      "payload_txos": [
        {
          "tx_out_proto": "32370a220a20ea71b9404e5dd41be2dfb7c2692fa667551fb8384f583b920ee77440d7cc4c27112446099e9c4d73d31a08529039b52d15e8ca12220a20f0eb6416c6da0dfd22c16f4d94de0a7606556b556ed7f5d080baa34a0714f67f1a220a20167628bd36b6c70aed289cdb3d61d22eb4b40a48f304c484a8f8de781ab5456522560a54c6a878bc8d6da36a47903332336f59b5af7fcfec635c4b914051e762141f5060b52b4e634533904675a289f870faf70dd75f012cafeec0e809fee8d71e831369077d4fd028d7a3f4b9b540f8abe19c62936b01002a440a42f6e678abdcf001450b706ab2ac855969bf7abecd1da2ecebad018eb6443cdff4872f44bc5f53beabeaa2ae332c696cf30a4c1ce1cc45ef5092c07466bca3e02ad32c",
          "amount": {
            "value": "229200000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778",
          "confirmation_number": "d0257c93a691dba8e9aa136e9edb7d6882470e92645ed3e08ea43d8570f0182e"
        }
      ],
      "change_txos": [
        {
          "tx_out_proto": "32370a220a20d007cecbca4ea2a2b6d0fb0d95470470a75c07d6c4496a9c8a065d170b07fd3a1111766004d70ac6271a084927c0f6ce61b60012220a20c441698796282c4147dc3f7e6ec07c9479ed8dbfcafb71929adc97bafb38080e1a220a20aadf8bd1437b52177d290c33ce5602e63ba3efc0cc006cb55545d333cded9f0b22560a544df264ab0cb8c9490774fe6242cd2e2951dafe92f976e0ec84698d39b5ce1b19c68be029c5aed4c327fde66917e8e907a19643c1c3d37bca5b0a460b59829d236f2aeafaa924184cbd4637b0af8dd408885e01002a440a424eb126875a5430560d942da3995aea5fc1a4feb68b19d8742519894d38bf6ccd50f4b321a7816ff5ce651971380af7e2b943a17c35a9a34280ee85b1b04617a41ee8",
          "amount": {
            "value": "240800000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "2vdjN4LDGbxhrpQ4jT777Wc2jaCszLZ98kAwf8jvonb6NjBdoWTBMnNTZfBw3LK9NGA4uAUkcBmQAHXZHV54sVN9bc8Te7pnnR1YtQpwcU8",
          "confirmation_number": "7fad0212bc57c75731e247930ba1fd6f3c6b08181171eb55243d721f9c96e3cd"
        }
      ],
      "fee_amount": {
        "value": "400000000",
        "token_id": "0"
      },
      "tombstone_block_index": "1769546",
      "tx_proto": "0abc81010aeb7c0a9f020a370a220a20fe7aed88b2e6..."
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
  "method": "submit_transaction",
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
      "finalized_block_index": null,
      "status": "pending",
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
