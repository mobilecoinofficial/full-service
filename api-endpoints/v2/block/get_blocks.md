---
description: Get the JSON representation of multiple "Block" objects in the ledger.
---

# Get Blocks

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param      | Purpose                          | Requirements                    |
|---------------------|----------------------------------|---------------------------------|
| `first_block_index` | The first block index to return. | Block must exist in the ledger. |

| Optional Param | Purpose                        | Requirements |
|----------------|--------------------------------|--------------|
| `limit`        | The number of blocks to return |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Body Request" %}

```
{
  "method": "get_blocks",
  "params": {
    "block_index": "10",
    "limit": 2
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}

{% tab title="Response" %}

```
{
  "method": "get_blocks",
  "result": {
    "blocks": [
      {
        "id": "1caf5f40b3af987b7180e37fda27b22afaf1fe021c51759702a86c70ac73fb2b",
        "version": "0",
        "parent_id": "2ad80a8339a7fa4345fafcb32a4b4cf31fde19f1dbd245638b2305755f56664a",
        "index": "10",
        "cumulative_txo_count": "36",
        "root_element": {
          "range": {
            "from": "0",
            "to": "63"
          },
          "hash": "7c6ee94966a25058053a736ecf93410056f7a61e6826492b236a1d74960349ba"
        },
        "contents_hash": "364a7666e143aa0e6967124ba21bfa478c8354b5139ea654b816612bf3c32c41"
      },
      {
        "id": "c9098ce66ad40495d3347cb5f50ee8109c4a5c881f24cfb8085af24f1517af7c",
        "version": "0",
        "parent_id": "1caf5f40b3af987b7180e37fda27b22afaf1fe021c51759702a86c70ac73fb2b",
        "index": "11",
        "cumulative_txo_count": "38",
        "root_element": {
          "range": {
            "from": "0",
            "to": "63"
          },
          "hash": "29214009f53ea67d07902ba8f45fa5256609bb53b8431538b3294458979eb67a"
        },
        "contents_hash": "8637dad481d11ac504cf38554d53229b18e099dd61e4927e9754d1941dbf990b"
      }
    ],
    "block_contents": [
      {
        "key_images": [
          "0a209695c4a088e2d177b5f87976096f5deafe12e22b14b65e3cefbcf3338f4ef603"
        ],
        "outputs": [
          {
            "masked_amount": {
              "commitment": "ead9a4df10433156bf25cfc3a7b071a2ee20376a53a87a22f0893d872b1be40c",
              "masked_value": "6817199765353746279",
              "masked_token_id": "",
              "version": 1
            },
            "target_key": "d848939f37624bc3159cde523787fc7f5c30135ee16abba6126c5dc6eec63a1f",
            "public_key": "1a70e3383373c31a73c01b0987f657cd9250a2bf95a28174fb304890b0168a7c",
            "e_fog_hint": "54610778c3f6558f6f6c52e3168cc13e2e08ab520eb46c2839020e20c8f059ece37e9cfa28c4328aa0351920849d5c2d565172270e9c20e339582b53bc2a6d7c1373d536ec1a9258110f689145870cadd6ae0100",
            "e_memo": ""
          },
          {
            "masked_amount": {
              "commitment": "64654ec80d7a43cdd0f2752833d70b4e4e949af44884e219115cecd28304c916",
              "masked_value": "16842093316104514354",
              "masked_token_id": "",
              "version": 1
            },
            "target_key": "fac9dc71a33bfabb64049f9faabed6302a46e4f095edde8405d87124a4679320",
            "public_key": "7a01a348a0a9c6da070bfa70b467802e0a7ee49c7c828a814ac4a474442a0a18",
            "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "e_memo": ""
          }
        ]
      },
      {
        "key_images": [
          "0a20a82fd520e5a9245776bed4e002d63939bbc97256bc76064f7950268eab938c69"
        ],
        "outputs": [
          {
            "masked_amount": {
              "commitment": "4ebeed141d87c99d51ff09013a7aac35aca0de6455d9b7a774d31af9a425a67b",
              "masked_value": "4128812440862043022",
              "masked_token_id": "",
              "version": 1
            },
            "target_key": "96e8772a54928b4e45d158917986733034768c56871458258dc1c21afb468f11",
            "public_key": "82199be50fa18b3268ca2fd486533b4de75d687be21975b2ae661d9816be2316",
            "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "e_memo": ""
          },
          {
            "masked_amount": {
              "commitment": "d49e2ed4639211bed5d87b0572ac5cf55dbadedc0f64408024277fe5bc75cf64",
              "masked_value": "18400396735931331034",
              "masked_token_id": "",
              "version": 1
            },
            "target_key": "806efd05a0b3ac2579fe0fb3ee39cee85f4132ff0a60d40ee415d01bb765a502",
            "public_key": "fa2945ebc19d504921096830603df708f61161e55b5cf925e62bd3ce24522d4c",
            "e_fog_hint": "0c291818ff5c6d4a0d316e54c51ced28ab4828e6f461dc2c23a8eac4a15b9362d5383656c1384da40245cd51eb74244bec61cfb4ee4b53bc4da99a4987b618d970784f9009a516fc539e26a966c8768ebab80100",
            "e_memo": ""
          }
        ]
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}
{% endtabs %}
