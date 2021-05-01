---
description: Get the JSON representation of the "Block" object in the ledger.
---

# Get Block

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `block_index` | The block on which to perform this action. | Block must exist in the wallet. |

## Example

{% tabs %}
{% tab title="Body Request" %}
```text
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
```text
{
  "method": "get_block",
  "result": {
    "block": {
      "id": "7cb35994cfcddf6c1e807b178d97d26b1426b0c5e035870c1f847194d2974051",
      "version": "0",
      "parent_id": "66c93f21731a852b1c3779362b86f60c2df4569a0f3192c2606aab80abf97720",
      "index": "3204",
      "cumulative_txo_count": "9630",
      "root_element": {
        "range": {
          "from": "0",
          "to": "16383"
        },
        "hash": "13578eb43225da9ccac84054ec53a35882c11ffa91fd3fddf83a3695e3da2d34"
      },
      "contents_hash": "e6859fe30de1bdaca04da1f48c672a7efe2a20dc2f92f274beecd95335057f40"
    },
    "block_contents": {
      "key_images": [
        "0a2014af115d02757996dc9ffb9503147a1df116944bbcb7b7485d004207c3ed5148",
        "0a20f2161a1f709490ba7916f2f9b1240a8dd6ae373cf53db830bd0cf72784517733"
      ],
      "outputs": [
        {
          "amount": {
            "commitment": "3aed988182291e60592193834c1785cc461770c88a923e92c46b5e0c739f7328",
            "masked_value": "11758756470468044129"
          },
          "target_key": "08f63701a50e70dfe5f83680e417f20da0d29cfcf5a06487dea6d9b610d6531c",
          "public_key": "56fb0ba834264fff19f4228423c16f95aa48524f027e94ec95c4370ab92f4219",
          "e_fog_hint": "5b00649093c46e1f47447810d9b57885ce6d1046582f800205c9a823aec01c30dcb09e3f808ece5701b05976209d2290ba10b049e14955ab9904e9aedd5ad6957234ebc0e56a7e23eb5f1c80699a2764334c0100"
        },
        {
          "amount": {
            "commitment": "c6fe77aaf3718ee614514cb127628d067c72d7836ebdf0cf0aeb36e465b48033",
            "masked_value": "2884679206729723147"
          },
          "target_key": "343e3fd460a447e3576bdd4e7c461811693e3352da9f7db9c88ee5246f5c5a28",
          "public_key": "805daef5b1d8363c1af964f2aeb2b42f1960c780a514c3c2ead8d07230ca9303",
          "e_fog_hint": "cf544d5c7f78af198ad0cfe6ebb270d1342f9e2e9ceeadadd8c6a5a216f21c7f8989b0580d7cd73a7e32a7a4f48ad192cf9987fe4ffe734bbcf64e18fbb4f787fd62030c29274b576c68e85441b23374edb00100"
        },
        {
          "amount": {
            "commitment": "e81a0fd37fec7efa411bcf2671714d2f9653cd5de8adf0d981f807d63938716e",
            "masked_value": "8464075929622445691"
          },
          "target_key": "a48206113129e42d8c5cc1122cd76e0a06985f666f504e144e3d45d45095de5e",
          "public_key": "b633c32f91aea42de6b6cd88dc0f05af47861db24823cc485430f9bbb7a35b22",
          "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        }
      ]
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

