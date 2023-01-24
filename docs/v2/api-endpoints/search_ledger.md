---
description: Search the ledger for blocks based on a query string (that can be either a tx out public key or a key image)
---

# Search Ledger

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `query` | Query string to search for. | Currently the supported queries are hex representations of a tx out public key or a key image. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Body Request" %}
```text
{
  "method": "search_ledger",
  "params": {
    "query": "0a209458f8e0aa9eff40475c64bcb90407adb0da56e6889665ea23176ae28314ca4f"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "search_ledger",
  "result": {
    "results": [
      {
        "result_type": "KeyImage",
        "block": {
          "id": "e0140f99e60299842b953cd5caa8d2c8977861eba66c8987deb8d628a98cabef",
          "version": "2",
          "parent_id": "c49b11e3cc8da6ac030a5ca15e969a2a9eb6c4f466af71eb04baaf5c21b56401",
          "index": "1369672",
          "cumulative_txo_count": "4112217",
          "root_element": {
            "range": {
              "from": "0",
              "to": "4194303"
            },
            "hash": "25485b5701617714fe66fb59fe01b296ff627252eb8b9a95fe7445ee8e889263"
          },
          "contents_hash": "840c5a4e462ec19a89195f71a6731503dcaf91a3f19655c1dd89194fa367beb7"
        },
        "block_contents": {
          "key_images": [
            "0a209458f8e0aa9eff40475c64bcb90407adb0da56e6889665ea23176ae28314ca4f",
            "0a209cd92e6c177ee88bf5f0ec74ee0c689cdd86452681a97befeab2a5c66f6a4c4b"
          ],
          "outputs": [
            {
              "masked_amount": {
                "commitment": "908013979a7921c58bebe18ea52e6a0d0549357de0f91162c27918a0c1aca36a",
                "masked_value": "5021023487447205630",
                "masked_token_id": "4b91842f66e11f78",
                "version": 1
              },
              "target_key": "42ba9c95315143febafa00c491ae9c2b612b2b311f4514c11c21a692bc960716",
              "public_key": "545eb308f009216f9b2768879e70ec7db505e6d1e22d1ef9629a89a0934cf875",
              "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
              "e_memo": "2602de077ca7fa4c0199aeb6297f8db4f52e1fc73be0e358271c4ef12cd3452e952a6649b6e130bfaaf310eebf4a76afa4c9f94724c73e28c5da209668582b27ed30"
            },
            {
              "masked_amount": {
                "commitment": "aac44ae256a5ad368d34436ab0c07669743d0033020bff45fca01586ca3c4519",
                "masked_value": "9134733508405425051",
                "masked_token_id": "4c088b69b7a477de",
                "version": 1
              },
              "target_key": "7a521900c99388e6bc6d40e17cb42f95fffcb3605c881d04323157eb92ed2404",
              "public_key": "7a21df10a5af5cb0f8ece7d2d9ec92fe71ca86a4956f47930e92eb0924037e76",
              "e_fog_hint": "c58f3d8c2f06d8446d7847c30892a2a085cbe76aef4de2a01e8911a98a9c02a023d2aee25ae5756708b36ece79bcb6ffad7c121fa861a4ced11df207b64fac64581d672caa4f4fe35d46e342b2afd9b4d6950100",
              "e_memo": "4473d4483f0504a9a2866995b2d8e560b41c3c3eae7d3ee2e48357bf5771e886db93fcdbc613523bdbf693e8c894120d16285d88b65815ba434103399cc5740a9f7b"
            },
            {
              "masked_amount": {
                "commitment": "d621fc3837f94ec5efe3f391f67f8e3aa1e5ca6fa142e40554ee7401cc52e069",
                "masked_value": "7675168110278462933",
                "masked_token_id": "d2a91db0f96813de",
                "version": 1
              },
              "target_key": "2cfa011ec4c8ee576cc7dc56ae7ef5928b65f50e77fc2a9bf92822f8d66c0259",
              "public_key": "98ff4f715080417cb65165c606ab39169caa969f23643e6b4f3e5569385fed1b",
              "e_fog_hint": "c7fd6f47978f68eeeb087fa5e94ad535beb4d1a0a0255ee9af1407bda136d9e2f9b7425a001bef4c58967b1283c22e0583d034fde2db01f4cf6b0cfaaaaf9fbdd37d959a5effd6545cdea201bda4adcb21a40100",
              "e_memo": "4c692f00fdc26bea03012d7eb1e10ecde3ad3f07ef4cfab9f9490458e3668dca805c14e69fd3818ffd14fa9a1898811a5610837d818c5cabd4be397e62665b62b07c"
            }
          ]
        },
        "tx_out": null,
        "key_image": {
          "block_contents_key_image_index": "0"
        }
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

