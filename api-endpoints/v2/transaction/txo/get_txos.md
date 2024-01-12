---
description: Get TXOs for a given account with offset and limit parameters
---

# Get TXOs

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Optional Param             | Purpose                                                                                                  | Requirements                      |
| -------------------------- | -------------------------------------------------------------------------------------------------------- | --------------------------------- |
| `account_id`               | The account on which to perform this action.                                                             | Account must exist in the wallet. |
| `address`                  | The address b58 on which to perform this action.                                                         | Address must exist in the wallet. |
| `status`                   | Txo status filer. Available status: "unverified", "unspent", "spent", "orphaned", "pending", "secreted", |                                   |
| `min_received_block_index` | The minimum block index to query for received txos, inclusive                                            |                                   |
| `max_received_block_index` | The maximum block index to query for received txos, inclusive                                            |                                   |
| `offset`                   | The pagination offset. Results start at the offset index.                                                |                                   |
| `limit`                    | Limit for the number of results.                                                                         |                                   |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_txos",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "min_received_block_index": "1769500",
    "max_received_block_index": "1769600",
    "offset": 2,
    "limit": 8
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_txos",
  "result": {
    "txo_ids": [
      "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d",
      "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
      "6603da459bb4bb88ddfaff2ac702238de4198b67b37e93d789e7311f8978641b"
    ],
    "txo_map": {
      "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d": {
        "id": "9832a72bf474e9e7bb105a12110919be8d2f9d5ec34fc195271480935244a64d",
        "value": "240800000000",
        "token_id": "0",
        "received_block_index": "1769541",
        "spent_block_index": "1769546",
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "status": "spent",
        "target_key": "0a20c441698796282c4147dc3f7e6ec07c9479ed8dbfcafb71929adc97bafb38080e",
        "public_key": "0a20aadf8bd1437b52177d290c33ce5602e63ba3efc0cc006cb55545d333cded9f0b",
        "e_fog_hint": "0a544df264ab0cb8c9490774fe6242cd2e2951dafe92f976e0ec84698d39b5ce1b19c68be029c5aed4c327fde66917e8e907a19643c1c3d37bca5b0a460b59829d236f2aeafaa924184cbd4637b0af8dd408885e0100",
        "subaddress_index": "18446744073709551614",
        "key_image": "0a209ec60610aa59dee42f74e7d3b9b513614d0720a0147f91c521a841a02130f835",
        "confirmation": "0a207fad0212bc57c75731e247930ba1fd6f3c6b08181171eb55243d721f9c96e3cd",
        "shared_secret": "0a20d662b5dc2d2ada8b72cecde0a84d822aad7018e37c3ac58445740064fa26ba78",
        "memo": {
          "Destination": {
            "recipient_address_hash": "8a515f44149956609f75b27d214daed6",
            "num_recipients": "1",
            "fee": "400000000",
            "total_outlay": "1400000000",
            "payment_request_id": null,
            "payment_intent_id": null
          }
        }
      },
      "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb": {
        "id": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
        "value": "470400000000",
        "token_id": "0",
        "received_block_index": "1769533",
        "spent_block_index": "1769541",
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "status": "spent",
        "target_key": "0a20bcaa42886171e60c50f0a4527663507a890fbecb5016f6d9042ce6be1cd7fb52",
        "public_key": "0a20cecc879afd79153210ff79b58947416a883d4f68253d415533c0e8898e09f045",
        "e_fog_hint": "0a54643db209825ced0df98a277c989b9d1876ac4009397137af1fabd3856c7c97dd629be47752cd532aa1f4bb1412d4dac9a76d50e67b4b99da017dc3a40caa99b4933ef6b4b51c56a338fc8648244eba5a22d90100",
        "subaddress_index": "18446744073709551614",
        "key_image": "0a20fafbf66b4da787c3a7d0c6a12d67620efcb47c3299ab4382627e468c718d4d1e",
        "confirmation": "0a207e8073157c3c938cc06c10c17094ba6940ec6ea15985df4760e43aaddd9bdccb",
        "shared_secret": "0a20a6d0637fe8358b1ca1cf90ba2e8efdb52a2c369a004fc1dd5cd18557116a9b34",
        "memo": {
          "Destination": {
            "recipient_address_hash": "12df8df8ddcc6d8830f8cdd1cd00cc51",
            "num_recipients": "1",
            "fee": "400000000",
            "total_outlay": "1000400000000",
            "payment_request_id": null,
            "payment_intent_id": null
          }
        }
      },
      "6603da459bb4bb88ddfaff2ac702238de4198b67b37e93d789e7311f8978641b": {
        "id": "6603da459bb4bb88ddfaff2ac702238de4198b67b37e93d789e7311f8978641b",
        "value": "1000000000000",
        "token_id": "0",
        "received_block_index": "1769524",
        "spent_block_index": "1769533",
        "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
        "status": "spent",
        "target_key": "0a203e90849059e97196b92784842abaadd273efc21669a188949242f3f2059fb87d",
        "public_key": "0a20e84d5ecd87acba366f731c2865085e5dba2d15f8c7f69b36bfd0b4ff21c4cd02",
        "e_fog_hint": "0a54ca6975a5fafa891973e0a2ff03875b01fe891bf32be9e9480d8b0c8421679799c9944efa497a1b3b561c6df053c58351e5f8dbebe0f55b05f88986472cc3fc3fd6157e110cf47fd1c7bff07bba1bd05cd5e50100",
        "subaddress_index": "0",
        "key_image": "0a20a08be966b837b31e306de665ccc3cfb2e0b515ddc3d959e7858114bae0069d16",
        "confirmation": "0a208f9ded89e086fbf3e6d6b18cb73a797490ba4d45d182e6ba8ae907534be9dd11",
        "shared_secret": "0a20e6269296b8152741cbd679322af61117230cfc4a92e4c8e296d78eae8326ef08",
        "memo": {
          "AuthenticatedSender": {
            "sender_address_hash": "12df8df8ddcc6d8830f8cdd1cd00cc51",
            "payment_request_id": null,
            "payment_intent_id": null
          }
        }
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
