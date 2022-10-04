---
description: Get the Tx Out Membership Proof for a selection of Tx Outs
---

# Get Txo Membership Proof

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `outputs` | The TXOs to get the membership proofs for | TXO must exist in the ledger |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Body Request" %}
```json
{
  "method": "get_txo_membership_proofs",
  "params": {
    "outputs": [
      {
        "masked_amount": {
          "commitment": "f6207c1952489634384434c230bac7eb72427d15742e2b43ce40fa9be21f6044",
          "masked_value": "778515034541258781",
          "masked_token_id": ""
        },
        "target_key": "94f722c735c5d2ada2561717d7ce83a1ebf161d66d5ab0e13c8a189048629241",
        "public_key": "eaaf989840dba9de8f825f7d11c01523ad46f7f581bafc5f9d2a37d35b4b9e2f",
        "e_fog_hint": "7d806ff43d1b4ead24e63263932ef820e7ca5bc72c3b6a01eee42c5e814769eac6b78c72f7fe9cbe4b65dd0f3b70a63b1dcb5f3223430eb5890e388dfa6c8acf7c73f8eeeb3def9a6dd5b4b4a7d3150f8c1e0100",
        "e_memo": ""
      }
    ],
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```json
{
    "method": "get_txo_membership_proofs",
    "result": {
        "outputs": [
            {
                "masked_amount": {
                    "commitment": "f6207c1952489634384434c230bac7eb72427d15742e2b43ce40fa9be21f6044",
                    "masked_value": "778515034541258781",
                    "masked_token_id": ""
                },
                "target_key": "94f722c735c5d2ada2561717d7ce83a1ebf161d66d5ab0e13c8a189048629241",
                "public_key": "eaaf989840dba9de8f825f7d11c01523ad46f7f581bafc5f9d2a37d35b4b9e2f",
                "e_fog_hint": "7d806ff43d1b4ead24e63263932ef820e7ca5bc72c3b6a01eee42c5e814769eac6b78c72f7fe9cbe4b65dd0f3b70a63b1dcb5f3223430eb5890e388dfa6c8acf7c73f8eeeb3def9a6dd5b4b4a7d3150f8c1e0100",
                "e_memo": ""
            }
        ],
        "membership_proofs": [
            {
                "index": "9636",
                "highest_index": "2954613",
                "elements": [
                    {
                        "range": {
                            "from": "9636",
                            "to": "9636"
                        },
                        "hash": "ba5fe09724623f2fb2dd769561aa0763dbc77efdba7ad8429a724949e8ac5180"
                    },
                    {
                        "range": {
                            "from": "9637",
                            "to": "9637"
                        },
                        "hash": "6845c1a6ec4543e8e045604b0573677872965972e4717c0fdfac038482671bbf"
                    },
                    {
                        "range": {
                            "from": "9638",
                            "to": "9639"
                        },
                        "hash": "68eab9e61bd72d68889d410f87c5d00356f103e367c5b0cdfc4bb7f70d5fdaa5"
                    },
                    {
                        "range": {
                            "from": "9632",
                            "to": "9635"
                        },
                        "hash": "6fc1d18c4593192e66e25ba7027c30a9a4e9ca188041bdad29524d26adfedc1e"
                    },
                    {
                        "range": {
                            "from": "9640",
                            "to": "9647"
                        },
                        "hash": "80e49cb0cf92cc5f14849b0d75461df291d88fd8a8db6dcc380e431419056aa4"
                    },
                    {
                        "range": {
                            "from": "9648",
                            "to": "9663"
                        },
                        "hash": "2a5b5cabea35d66b99ee4d348389b2a6f67e925d28a4fca66a4ebf72bfadabe6"
                    },
                    {
                        "range": {
                            "from": "9600",
                            "to": "9631"
                        },
                        "hash": "5360fea1cd5a0a56289f37d064765642841583f643c5f02056a5dc58206a9d4d"
                    },
                    {
                        "range": {
                            "from": "9664",
                            "to": "9727"
                        },
                        "hash": "b7c4ddf7d711f5393546e275a81a5e68a130bd789f0bf978a292838902dd4215"
                    },
                    {
                        "range": {
                            "from": "9472",
                            "to": "9599"
                        },
                        "hash": "1d0222a2289a66787c52ddd8346bf89807ebe5033afa952c90a31596720a0a4f"
                    },
                    {
                        "range": {
                            "from": "9216",
                            "to": "9471"
                        },
                        "hash": "40605f54922bfb35ca707773faa92e0f93f381980944f46e4074ca39a3647088"
                    },
                    {
                        "range": {
                            "from": "9728",
                            "to": "10239"
                        },
                        "hash": "4002e276511a4a94832e2dec52ca8ddf3e01371afb4035db06d5759a13f2a365"
                    },
                    {
                        "range": {
                            "from": "8192",
                            "to": "9215"
                        },
                        "hash": "1851bd61df6fdcef280e1e0f65700e1fcba4fcf71e492a3e0812b1e33b992fe5"
                    },
                    {
                        "range": {
                            "from": "10240",
                            "to": "12287"
                        },
                        "hash": "ac98ec9700c9a55eda01ea036d207778ce203e7e9b0fc53572a94b67ae6e7406"
                    },
                    {
                        "range": {
                            "from": "12288",
                            "to": "16383"
                        },
                        "hash": "fb103b9efbb385fb972a34c2e49dc3f8befbe84280236b07a6d3c7c140535ae7"
                    },
                    {
                        "range": {
                            "from": "0",
                            "to": "8191"
                        },
                        "hash": "e3414a20e668ca283fe1cc5f49a9e883234cfcff28bce60556c3e2102f908620"
                    },
                    {
                        "range": {
                            "from": "16384",
                            "to": "32767"
                        },
                        "hash": "d73181dc373033eced433a797aceda8da2664972198cc99c0e0c52851e6f7e90"
                    },
                    {
                        "range": {
                            "from": "32768",
                            "to": "65535"
                        },
                        "hash": "ea706f9b84f872c459e0e9e316705bc3a72bc683625b1279259d48d8a1d63633"
                    },
                    {
                        "range": {
                            "from": "65536",
                            "to": "131071"
                        },
                        "hash": "1ff8fea30828f2548877cc69ba12218c7c8a38969162cbcb9dc25e5e08a1ae7f"
                    },
                    {
                        "range": {
                            "from": "131072",
                            "to": "262143"
                        },
                        "hash": "72973b7fbb93e23b67f278721c951098f630c375aaaf5e16fc04fe6271485d2d"
                    },
                    {
                        "range": {
                            "from": "262144",
                            "to": "524287"
                        },
                        "hash": "ede9af86064b5b91edf646ebe6b8f0fbaa31344894c77ad06d9f79784d536bca"
                    },
                    {
                        "range": {
                            "from": "524288",
                            "to": "1048575"
                        },
                        "hash": "5027acb6de8ac0b4e8ed35e23af60b165960a5e02fc3a5cdef0fb476e2f6ffc9"
                    },
                    {
                        "range": {
                            "from": "1048576",
                            "to": "2097151"
                        },
                        "hash": "47771f1a5984fc3243e37869733db130f174967f9c81847ea64cccef937e1c7c"
                    },
                    {
                        "range": {
                            "from": "2097152",
                            "to": "4194303"
                        },
                        "hash": "616efa08e869e62bb7ba8a04523b1fc18ec7d0c71c524d08d24f54aa7383dbd3"
                    }
                ]
            }
        ]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}

