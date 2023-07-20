---
description: Get the JSON representation of the "Block" object in the ledger.
---

# Get Block

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Optional Param   | Purpose                                                              | Requirements                         |
| ---------------- | -------------------------------------------------------------------- | ------------------------------------ |
| `block_index`    | The block on which to perform this action.                           | Block must exist in the ledger.      |
| `txo_public_key` | The public key on which to perform this action, as hex encoded bytes | Public key must exist in the ledger. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Body Request" %}
```
{
  "method": "get_block",
  "params": {
    "block_index": "3204",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_block",
  "result": {
    "block": {
      "id": "f9e61c4c62d2b7f4741ae219c88e88a6446549b3ec6d223665dd5290b6e4f41e",
      "version": "0",
      "parent_id": "bf971608329e17c2fb1a08995580459fc158df9d628f05a32c951b83b4656307",
      "index": "3204",
      "cumulative_txo_count": "9638",
      "root_element": {
        "range": {
          "from": "0",
          "to": "16383"
        },
        "hash": "cc6e295d51947fad0c114325924d87ce247b96e541ca877642a1012ed697ec32"
      },
      "contents_hash": "c7933db034fe4916a2aff689e5396ca95dee080255071073f6aa574d11433b4c"
    },
    "block_contents": {
      "key_images": [
        "0a209a458509e1b4d2bdc9f508efdd763d0b2f93e6208167d66b409e5340ed966b0f"
      ],
      "outputs": [
        {
          "masked_amount": {
            "commitment": "702425de53fb2beaa3a9f6ae55d448fb76e3731706321bec2e2b14f971a61d53",
            "masked_value": "1805309211121738718",
            "masked_token_id": "",
            "version": null
          },
          "target_key": "e082729801cc958f0fcda3b2041408015ac07ca041552e4f6c56a4286f659f2f",
          "public_key": "ea0d6a2f47cc54dcd47aa52a58fe392b11607c43467a6f45d49dd0fc551ec520",
          "e_fog_hint": "cbc9d140db329ab0f533c5878b0dc47f42cec840c78614688df1860a3c7c840070503e4661164324929ae4777bbc821b80cc8fdb0ee0b222a750b8ed2a226fc25b1795d20b031ffe2d2f225d9f0074cb38d10100",
          "e_memo": ""
        },
        {
          "masked_amount": {
            "commitment": "f6207c1952489634384434c230bac7eb72427d15742e2b43ce40fa9be21f6044",
            "masked_value": "778515034541258781",
            "masked_token_id": "",
            "version": null
          },
          "target_key": "94f722c735c5d2ada2561717d7ce83a1ebf161d66d5ab0e13c8a189048629241",
          "public_key": "eaaf989840dba9de8f825f7d11c01523ad46f7f581bafc5f9d2a37d35b4b9e2f",
          "e_fog_hint": "7d806ff43d1b4ead24e63263932ef820e7ca5bc72c3b6a01eee42c5e814769eac6b78c72f7fe9cbe4b65dd0f3b70a63b1dcb5f3223430eb5890e388dfa6c8acf7c73f8eeeb3def9a6dd5b4b4a7d3150f8c1e0100",
          "e_memo": ""
        },
        {
          "masked_amount": {
            "commitment": "d09561d73bff74599c989400ee219875f06690a258f76bbafa1caf7d0e0cea0c",
            "masked_value": "3925568865368868231",
            "masked_token_id": "",
            "version": null
          },
          "target_key": "9eb851a8ef81670b5d290039c979a16b675b465590178efcc19fdfdc2566547e",
          "public_key": "faee99e718fc35a43dfcd640f43abd20f0960f5bf7c5c4a72dd10c2868d52949",
          "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
          "e_memo": ""
        }
      ]
    },
    "watcher_info": null
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
