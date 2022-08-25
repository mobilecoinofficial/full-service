---
description: Sample a desired number of mixins from the ledger, excluding a list of tx outs
---

# Sample Mixins

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
    "method": "sample_mixins",
    "params": {
        "num_mixins": 10,
        "excluded_outputs": [{
                    "masked_amount": {
                        "commitment": "f6207c1952489634384434c230bac7eb72427d15742e2b43ce40fa9be21f6044",
                        "masked_value": "778515034541258781",
                        "masked_token_id": ""
                    },
                    "target_key": "94f722c735c5d2ada2561717d7ce83a1ebf161d66d5ab0e13c8a189048629241",
                    "public_key": "eaaf989840dba9de8f825f7d11c01523ad46f7f581bafc5f9d2a37d35b4b9e2f",
                    "e_fog_hint": "7d806ff43d1b4ead24e63263932ef820e7ca5bc72c3b6a01eee42c5e814769eac6b78c72f7fe9cbe4b65dd0f3b70a63b1dcb5f3223430eb5890e388dfa6c8acf7c73f8eeeb3def9a6dd5b4b4a7d3150f8c1e0100",
                    "e_memo": ""
                }]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```json
{
    "method": "sample_mixins",
    "result": {
        "mixins": [
            [
                {
                    "masked_amount": {
                        "commitment": "64ea2fd88ec8072007eb6c7dd2adf3d4541b79e7b94c19b310acc36b86936b07",
                        "masked_value": "4472708061428872055",
                        "masked_token_id": ""
                    },
                    "target_key": "36be07e9a9ef9e11103a5f604e48a0874b8fc2bbf86db50d143a01c1fc126301",
                    "public_key": "6ece9daf601107927cb95ce375fcb86059554b69f4a776e61cbefefc49727f1f",
                    "e_fog_hint": "701f428be921aa9be00eab10324ca914addf5d23dd4c98e3baeb6b5bf37a70dcd16982c38030d8d482b8418a7daf9cc49988f9fb383c07a5a0255525f1e54630372ba162a4c85289780e3d0822d2194c5fb20100",
                    "e_memo": ""
                },
                {
                    "index": "636079",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "636079",
                                "to": "636079"
                            },
                            "hash": "6d5cece0ac49a8fbddc184739da05a8d3f6df340ffca901519573e4b24802b0e"
                        },
                        {
                            "range": {
                                "from": "636078",
                                "to": "636078"
                            },
                            "hash": "5591b0ddad1c8fe6da5f320e913a0ebde526a991715a8b63b455ec57644b47f4"
                        },
                        {
                            "range": {
                                "from": "636076",
                                "to": "636077"
                            },
                            "hash": "3e3ce2c10ddbc374ff5d9f32baa56cb1da4998d7f17287eec79794be33653ebc"
                        },
                        {
                            "range": {
                                "from": "636072",
                                "to": "636075"
                            },
                            "hash": "c32dff79bd52d07d55bc838e3326e24920cb8fa4f28ffefe65564d63ae1e98d6"
                        },
                        {
                            "range": {
                                "from": "636064",
                                "to": "636071"
                            },
                            "hash": "d4f4dd2c3f8ed0f31a2904a920afa195d5783d7a755d3bb4db46fcf13bbae988"
                        },
                        {
                            "range": {
                                "from": "636080",
                                "to": "636095"
                            },
                            "hash": "f6fcbbbfbb9bedefd0dbd69fc8bbfd429bf4d714367d3514b9d4f9a5c1a497b6"
                        },
                        {
                            "range": {
                                "from": "636032",
                                "to": "636063"
                            },
                            "hash": "8990e3039f469e8e56a8a06d93cff5a74134854ef7acd7e3033868fb34cbc8f9"
                        },
                        {
                            "range": {
                                "from": "636096",
                                "to": "636159"
                            },
                            "hash": "618f2102cc1ef5f8515c256dcbfdbf3b54e3a5e757454bbf39d8bdd1830968c7"
                        },
                        {
                            "range": {
                                "from": "635904",
                                "to": "636031"
                            },
                            "hash": "f7d345789a5834d742063cd720ae8520e130bc2f064ae2e14c5339738229b47b"
                        },
                        {
                            "range": {
                                "from": "636160",
                                "to": "636415"
                            },
                            "hash": "387772cbcf196364f3fa5802e968b74612d84db352b97a1ac1214478a9ea329b"
                        },
                        {
                            "range": {
                                "from": "636416",
                                "to": "636927"
                            },
                            "hash": "acf7ca3cde6e9f14fdf57dc1429f4d9dd610f24fe65b3864d039de3b3c98d3f1"
                        },
                        {
                            "range": {
                                "from": "634880",
                                "to": "635903"
                            },
                            "hash": "7f7b39e17e57f939ea40ce1343cdc0c949c91148ab2ee4fe5416e2e97269495c"
                        },
                        {
                            "range": {
                                "from": "636928",
                                "to": "638975"
                            },
                            "hash": "96c1541116f340bb506af07320748b05696ae4ea3794ea4d7ac0513dbc3ca163"
                        },
                        {
                            "range": {
                                "from": "630784",
                                "to": "634879"
                            },
                            "hash": "eae2f4a4b8631badcb011b256866af8d07bf852e501053eaac2f5018cda2e268"
                        },
                        {
                            "range": {
                                "from": "622592",
                                "to": "630783"
                            },
                            "hash": "8dc7e4871a75c511b7e17cfd5050b50a5f31e6bfffe4c78041d7bfa0b5b53944"
                        },
                        {
                            "range": {
                                "from": "638976",
                                "to": "655359"
                            },
                            "hash": "72de9e21252b7e45a31eb39959b478a2b5b5f40bce40cf060bffcb7c5755cfec"
                        },
                        {
                            "range": {
                                "from": "589824",
                                "to": "622591"
                            },
                            "hash": "4ae17019426c6648db5d618c7b8cd9597b98efb35b7fe0991bdc174ab7c4fe47"
                        },
                        {
                            "range": {
                                "from": "524288",
                                "to": "589823"
                            },
                            "hash": "91ac1d42a7874b1fa0cba1450ba5de987215d690d128aeda9b6841421149001a"
                        },
                        {
                            "range": {
                                "from": "655360",
                                "to": "786431"
                            },
                            "hash": "0e9ef9d0354896086a7943b076aa7ddd0d0dd94d30542b82d90735e9a080a32a"
                        },
                        {
                            "range": {
                                "from": "786432",
                                "to": "1048575"
                            },
                            "hash": "4d4dec78fc598560811d42adc2744d7f2499c0b684414c355f6f3b577bfe74fe"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "524287"
                            },
                            "hash": "deee6f72a764e18887a16d96006b2c35f23c411d273805e4f10827151cba7a2a"
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
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "08c785048732cb8783a82115a521d181448317844c1c0b02750a29da94c47547",
                        "masked_value": "9114463916170722919",
                        "masked_token_id": ""
                    },
                    "target_key": "be1a926ee71f4cf97feebf8a9d4dd75d0eb4b374062c42e231b9ffc57cbc532e",
                    "public_key": "7aa0986a6ce4a40a5a5f9120fbb8ff61d97bd63824c48ec76584bed369a5a441",
                    "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "e_memo": ""
                },
                {
                    "index": "1386764",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "1386764",
                                "to": "1386764"
                            },
                            "hash": "f77e9e0fdda264b6df5cfbabe0dcb56e5016f7639ee734ffc3de09ee18d1b6f6"
                        },
                        {
                            "range": {
                                "from": "1386765",
                                "to": "1386765"
                            },
                            "hash": "13364b18c4b42c9f19c196b3fc072a513ecd677c26726d8f3132410aeca45d05"
                        },
                        {
                            "range": {
                                "from": "1386766",
                                "to": "1386767"
                            },
                            "hash": "8d359903b65f79f87bd31b1fc937659dea0d134dcbec00fb577ebaa1dc8a0504"
                        },
                        {
                            "range": {
                                "from": "1386760",
                                "to": "1386763"
                            },
                            "hash": "64bac16e8216c1f22254644a74910ca9ecf40362b2eb37f3c865e88da304a5cf"
                        },
                        {
                            "range": {
                                "from": "1386752",
                                "to": "1386759"
                            },
                            "hash": "d8ead582a2f3c910218161c1e968b72a4412d70db594d36183a20ed27931b0bc"
                        },
                        {
                            "range": {
                                "from": "1386768",
                                "to": "1386783"
                            },
                            "hash": "5e452bf3484731e828ee6586ef11fbd2a10608d50b93d1cacf795a79f7fbc354"
                        },
                        {
                            "range": {
                                "from": "1386784",
                                "to": "1386815"
                            },
                            "hash": "629dbda1218813d03c45a40278ea24b178b6bf983e324cea8b394a4c182e034f"
                        },
                        {
                            "range": {
                                "from": "1386816",
                                "to": "1386879"
                            },
                            "hash": "118b34abb576989c1a29e42848801605541db6b732b2a13e9de068d864d4320d"
                        },
                        {
                            "range": {
                                "from": "1386880",
                                "to": "1387007"
                            },
                            "hash": "d5eea22c8c5a51c348778a9b016d61ea16baa861e48cc931e0955fff83657446"
                        },
                        {
                            "range": {
                                "from": "1386496",
                                "to": "1386751"
                            },
                            "hash": "313c99adb7d1c37907c7e7b8d894b74dc656609c24dea2d4757bc3408d657a8d"
                        },
                        {
                            "range": {
                                "from": "1387008",
                                "to": "1387519"
                            },
                            "hash": "c17ad93d19fd07bf29420367b03425677d4038b56b8e7381c3a9bfe6a128e175"
                        },
                        {
                            "range": {
                                "from": "1387520",
                                "to": "1388543"
                            },
                            "hash": "c28844bdcea456b22aff573fe8513dee54b954be6e02b0732b6cbbcf9fd01600"
                        },
                        {
                            "range": {
                                "from": "1384448",
                                "to": "1386495"
                            },
                            "hash": "681361372e0c2a93125e8893b4472faded55ec2bc4e286d36182c432e8a6a8a9"
                        },
                        {
                            "range": {
                                "from": "1388544",
                                "to": "1392639"
                            },
                            "hash": "f1b630ac4afc057c3b1a6bae0012c7414d9b2bc73ac5978aac9895cb63144a4e"
                        },
                        {
                            "range": {
                                "from": "1376256",
                                "to": "1384447"
                            },
                            "hash": "7fcf29c1f15c0d31bf4dda3f1af48d3d034f75e2a60c818ba112a810adf36404"
                        },
                        {
                            "range": {
                                "from": "1392640",
                                "to": "1409023"
                            },
                            "hash": "e38eb71576d2656b924937df7dd28e63c85b5fca9eb15d3cdc331e35d0a353f9"
                        },
                        {
                            "range": {
                                "from": "1409024",
                                "to": "1441791"
                            },
                            "hash": "b0b5cde6695c62196ae1a0dc66c637173fc6026357cbbb784f69dbb9312ffbf5"
                        },
                        {
                            "range": {
                                "from": "1310720",
                                "to": "1376255"
                            },
                            "hash": "6276cccce69ae18d2fb278371848d31af09764089700a27cf774dbcc71437995"
                        },
                        {
                            "range": {
                                "from": "1441792",
                                "to": "1572863"
                            },
                            "hash": "3838f438938a2d43ee2746b58cc2b7f5c41e4e2fadf3a44af20bece443152c24"
                        },
                        {
                            "range": {
                                "from": "1048576",
                                "to": "1310719"
                            },
                            "hash": "8256f43b4bf0cd9c67d6d753e3f96ad489eaab5adb225f8ce3909164c2faa279"
                        },
                        {
                            "range": {
                                "from": "1572864",
                                "to": "2097151"
                            },
                            "hash": "10804b01fd7da84d4a1378917a6e696b5b986080dbd827e04db44ead4da3beb9"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "1048575"
                            },
                            "hash": "84c55b30648860a50aa0972473b8f24dd7dfb87ba94ddd2a2f23965a9d8e715c"
                        },
                        {
                            "range": {
                                "from": "2097152",
                                "to": "4194303"
                            },
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "ea34aca9aa8c7027f7e1c1f01f116a14d32e05bed6e6d79888324700d3fc3246",
                        "masked_value": "12766063450006500087",
                        "masked_token_id": ""
                    },
                    "target_key": "30b7c21177ac8949a01f0b446cbf6110709707716dec09db91dcce5d2417de3e",
                    "public_key": "649d984118f7a6837a030880b91ef96ab322683fc9bb5cf142928c24ddc8a261",
                    "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "e_memo": ""
                },
                {
                    "index": "2125346",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "2125346",
                                "to": "2125346"
                            },
                            "hash": "7dd3295649e325ccac2aa69f4cc8f10afb87c0680ad390f805b63a0d11fe297c"
                        },
                        {
                            "range": {
                                "from": "2125347",
                                "to": "2125347"
                            },
                            "hash": "ae438abaf6fa542683fdc4704b1a62ca310c465c142264cda922b7bf521e8d98"
                        },
                        {
                            "range": {
                                "from": "2125344",
                                "to": "2125345"
                            },
                            "hash": "693718ab1dc2c9d89ac519ec59d6b44a35d2cb8369f2652124e360cfd65cbfe1"
                        },
                        {
                            "range": {
                                "from": "2125348",
                                "to": "2125351"
                            },
                            "hash": "f8962c1462b2d5a99d815ea14fbe357f951c4ecb85aeba979d042146142b8b36"
                        },
                        {
                            "range": {
                                "from": "2125352",
                                "to": "2125359"
                            },
                            "hash": "445b9bc3d917d40cb4d464630124f4a88287d6b299a78e44abdb7d74a9d2e89c"
                        },
                        {
                            "range": {
                                "from": "2125360",
                                "to": "2125375"
                            },
                            "hash": "821be3e9f4a8f34191903bcfd8aa6f8fd090af93ecfab6b12402fdcc55f26f31"
                        },
                        {
                            "range": {
                                "from": "2125312",
                                "to": "2125343"
                            },
                            "hash": "f5aaa34635e1ef1f867dd13edabb97e4facf4b261cc1833cb70e8e38efe7a293"
                        },
                        {
                            "range": {
                                "from": "2125376",
                                "to": "2125439"
                            },
                            "hash": "55f45c44de2691d09c2e0cc8748db14d5aa65f06e909cd32b3f3ed1e0f1c9a2a"
                        },
                        {
                            "range": {
                                "from": "2125440",
                                "to": "2125567"
                            },
                            "hash": "3eb0238cc2645260308de7a10e5bba59c4ceaf44fe7051566cd4fb4e0895ea0c"
                        },
                        {
                            "range": {
                                "from": "2125568",
                                "to": "2125823"
                            },
                            "hash": "ece6bea2f096046d32402cce1695a3d2c795ce89e3f1c36f8d8a8affeb96227f"
                        },
                        {
                            "range": {
                                "from": "2124800",
                                "to": "2125311"
                            },
                            "hash": "a6e25ec4b4cd74d6e731cb13e249e001dfb07c0b06ab76a26f7f2dea627506e2"
                        },
                        {
                            "range": {
                                "from": "2123776",
                                "to": "2124799"
                            },
                            "hash": "19c5cfc849af3218159e43ccbe1fb9d85a6b3c76dcd839d0147998ba7f1daf0a"
                        },
                        {
                            "range": {
                                "from": "2121728",
                                "to": "2123775"
                            },
                            "hash": "8e8a29df691ff8f1e2437a1b91f0292ab9418d27a26ce0571421af036ff51563"
                        },
                        {
                            "range": {
                                "from": "2125824",
                                "to": "2129919"
                            },
                            "hash": "319c39c0f862240e60b0bd9a660de55153a67d8a8e6e8587556b19b37d6f437f"
                        },
                        {
                            "range": {
                                "from": "2113536",
                                "to": "2121727"
                            },
                            "hash": "6a679b25ea63bf173fe7dab977c313d7d993787ed97823c9f260761a654e3530"
                        },
                        {
                            "range": {
                                "from": "2097152",
                                "to": "2113535"
                            },
                            "hash": "eb70785ab822fbe6848fac70e63e8d2d8108321bdcdcb114f72a7446a583a610"
                        },
                        {
                            "range": {
                                "from": "2129920",
                                "to": "2162687"
                            },
                            "hash": "bd3635306a9c0f54768804c6c03166ccc7ee1a10c6dbf453e31212ea08becbad"
                        },
                        {
                            "range": {
                                "from": "2162688",
                                "to": "2228223"
                            },
                            "hash": "ac5b9616bd0cf54f1cddfa9a0de4cbb04bb1d2332e5303b229e99dc0d6110974"
                        },
                        {
                            "range": {
                                "from": "2228224",
                                "to": "2359295"
                            },
                            "hash": "ca95becb768d2da58f351edc53b37c5181c1a88817d17487008f6d6213c247d7"
                        },
                        {
                            "range": {
                                "from": "2359296",
                                "to": "2621439"
                            },
                            "hash": "856155ebef20bf530b0898f239e91460b8522e51ab4683b9290202b2beb4df7c"
                        },
                        {
                            "range": {
                                "from": "2621440",
                                "to": "3145727"
                            },
                            "hash": "18d379b1613545493acbd87811576eca818101115d6e40f924a759b3e1b6613d"
                        },
                        {
                            "range": {
                                "from": "3145728",
                                "to": "4194303"
                            },
                            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "2097151"
                            },
                            "hash": "f16256b5f5e635de8e230ad587df3f3d578bc5ba515c77e54d9bedee36e4435b"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "f0b6c50862cb78a68bf973428a97548dfcbbccd2b4e12d37a33db2fa0022e666",
                        "masked_value": "8577792118217636704",
                        "masked_token_id": ""
                    },
                    "target_key": "82d0e00b52c6e77eb6bec45700ba3b6d0c4f60aaf2666029e42608e82cad1c3f",
                    "public_key": "381315cfeca2bb5ed37a541f498b7feef3ffb9bcb6910f318a6f6466cd92cb08",
                    "e_fog_hint": "85045aa57961e6bfde9b5462288d40f57f83ed5f6ff03089a9beadb5949c4a578057d61634a5a290add523f3e2c6c4cdcec68872f96a8a39fa014e3f7cc7fed3d9694704faacf5c739ba38dffb99e02329ac0100",
                    "e_memo": ""
                },
                {
                    "index": "2277750",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "2277750",
                                "to": "2277750"
                            },
                            "hash": "6f1f13ddb6d83ba9cafe9a4ef74e928ec7c14d972af0875953aee8f464f879b5"
                        },
                        {
                            "range": {
                                "from": "2277751",
                                "to": "2277751"
                            },
                            "hash": "aa1eb04ea502183151f78a5ef2d13a9df176edd0c2697d873433ed454b406cb1"
                        },
                        {
                            "range": {
                                "from": "2277748",
                                "to": "2277749"
                            },
                            "hash": "fa2a86a8858821643aeb9140de5278b82451bce9c6fff45ab8aeeff7c3a52f3c"
                        },
                        {
                            "range": {
                                "from": "2277744",
                                "to": "2277747"
                            },
                            "hash": "a2dea261e997473cbdd62c30c11522829a09c360d5796ddeca2493c4d415ed06"
                        },
                        {
                            "range": {
                                "from": "2277752",
                                "to": "2277759"
                            },
                            "hash": "be5b70c52e9051bf17c45e50e0f8c6114da30beaedc77e355d9d8495c22a39c2"
                        },
                        {
                            "range": {
                                "from": "2277728",
                                "to": "2277743"
                            },
                            "hash": "78575cf0691e4ae4a09a620e0dfe51178ffac01e756c4f2644b549e7a3015674"
                        },
                        {
                            "range": {
                                "from": "2277696",
                                "to": "2277727"
                            },
                            "hash": "cf0c24eaadf7b8eca532a754f5e37ab5c1fc1e9b87bebf43cc274c2d15e8c947"
                        },
                        {
                            "range": {
                                "from": "2277632",
                                "to": "2277695"
                            },
                            "hash": "172b9c889b455494171072e1d6a2354799e6b17edae8b72bf8f1c668330803f8"
                        },
                        {
                            "range": {
                                "from": "2277760",
                                "to": "2277887"
                            },
                            "hash": "ea78cfe72968d29405ac71a47519c836bdcd8d0f59da5c81b4bc4229a61b57f9"
                        },
                        {
                            "range": {
                                "from": "2277376",
                                "to": "2277631"
                            },
                            "hash": "5704678da9fc86b85c0c58be44d8bcd00c340e56a9ff6a096fd7b43aeb56ba92"
                        },
                        {
                            "range": {
                                "from": "2277888",
                                "to": "2278399"
                            },
                            "hash": "baf0228a3254b378dfa60ca4f754696828fcc8bf3874dfb2fa039e9a5f872e8f"
                        },
                        {
                            "range": {
                                "from": "2278400",
                                "to": "2279423"
                            },
                            "hash": "ec4781ad766a2049ed73f88663866c2cf6c7b05c4dac1c8ff721532171c0a4dc"
                        },
                        {
                            "range": {
                                "from": "2279424",
                                "to": "2281471"
                            },
                            "hash": "d65c7c7c4c62a6cd60970bca93f19f9257dbf48cb45f9c223168d7934822ffb3"
                        },
                        {
                            "range": {
                                "from": "2281472",
                                "to": "2285567"
                            },
                            "hash": "6b29cf5cb41afe4b6efe603caf013024ddaebacfefca639d7362080a83ea9f4c"
                        },
                        {
                            "range": {
                                "from": "2285568",
                                "to": "2293759"
                            },
                            "hash": "dfdce7349e29dec37ee6c8c7a1edaaf2e6027f07bea3fdb4b6ce92ed16853c10"
                        },
                        {
                            "range": {
                                "from": "2260992",
                                "to": "2277375"
                            },
                            "hash": "83e3e6bec9ebee2c1f5c7e01e73e4a0bc2eafc88fd7358d4ecd195393fccaba6"
                        },
                        {
                            "range": {
                                "from": "2228224",
                                "to": "2260991"
                            },
                            "hash": "7d161a42df294c234558b94a2f22b8986d39a37e5dbff741802e65d5334c2e9e"
                        },
                        {
                            "range": {
                                "from": "2293760",
                                "to": "2359295"
                            },
                            "hash": "0e89057cb4e8f640d8a117eb6bfe444409ff095be5873e803366fa172ad255ee"
                        },
                        {
                            "range": {
                                "from": "2097152",
                                "to": "2228223"
                            },
                            "hash": "712316156af4fab2907756d0d2ef014f69ad95b5dbf061cdd38db9858990f226"
                        },
                        {
                            "range": {
                                "from": "2359296",
                                "to": "2621439"
                            },
                            "hash": "856155ebef20bf530b0898f239e91460b8522e51ab4683b9290202b2beb4df7c"
                        },
                        {
                            "range": {
                                "from": "2621440",
                                "to": "3145727"
                            },
                            "hash": "18d379b1613545493acbd87811576eca818101115d6e40f924a759b3e1b6613d"
                        },
                        {
                            "range": {
                                "from": "3145728",
                                "to": "4194303"
                            },
                            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "2097151"
                            },
                            "hash": "f16256b5f5e635de8e230ad587df3f3d578bc5ba515c77e54d9bedee36e4435b"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "2a0d1cf28b130dc0300fc55dcc59ff887db2e451378143bc24a6e5b12c3c372c",
                        "masked_value": "8255843966763408855",
                        "masked_token_id": ""
                    },
                    "target_key": "1486f9e275adf3b64c70cc7ada70d7129427225c9369f43cd093abd7cf88ea38",
                    "public_key": "a4475ff6dcf5fa1e8f4282154d3d45b4eb728392ebc3964403d527cb08fd9658",
                    "e_fog_hint": "62e3a313fc6d7cffe2db124d86e51d50379a358ba5c2aaa51cb309108bc2c3312ae8d0e9db65493e93f22ab1848a4d42c7252325a0b39af73513e191292ba1b3167b6695b425f503b709cb1fa010632d5da80100",
                    "e_memo": ""
                },
                {
                    "index": "557080",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "557080",
                                "to": "557080"
                            },
                            "hash": "7bd89771ce833642077d3a361eb6d51f1dc8fe997a4f36ba8e857e9fb172f982"
                        },
                        {
                            "range": {
                                "from": "557081",
                                "to": "557081"
                            },
                            "hash": "36087bfe4b0517f0da1fba460bf049ab03c751e0126f76742ac7c47f40e37af1"
                        },
                        {
                            "range": {
                                "from": "557082",
                                "to": "557083"
                            },
                            "hash": "ba5fff45112a19bd74207557d6080160e5fcf1c2c96b09e292dd3bf5309b6035"
                        },
                        {
                            "range": {
                                "from": "557084",
                                "to": "557087"
                            },
                            "hash": "7b3b3b1da13b7f7d577793fd90d48c1af3ca652c0a2d3aa11fc93eecefa277b7"
                        },
                        {
                            "range": {
                                "from": "557072",
                                "to": "557079"
                            },
                            "hash": "a53c8a96b19852f4660a3969770da266259556cd312c33cd4de4777c6e2c7a2e"
                        },
                        {
                            "range": {
                                "from": "557056",
                                "to": "557071"
                            },
                            "hash": "494137cefd77912fc822ba3d953b320a09eb70ecd785e3c46a9918a65c38796d"
                        },
                        {
                            "range": {
                                "from": "557088",
                                "to": "557119"
                            },
                            "hash": "be43f7b5178e08e901c4bbcc6d4ebcd6ff9123b64bdd4d5c855ffb0b62ab2af0"
                        },
                        {
                            "range": {
                                "from": "557120",
                                "to": "557183"
                            },
                            "hash": "9eca7d2b0ab5b1edc52ffd633719d2f1c47238cc7cba90a191001a29ce1cb226"
                        },
                        {
                            "range": {
                                "from": "557184",
                                "to": "557311"
                            },
                            "hash": "2b7319a2423daad90247d61d85bf38425f2a691e2964e7197909f636ff732ee8"
                        },
                        {
                            "range": {
                                "from": "557312",
                                "to": "557567"
                            },
                            "hash": "ade3ae591702fb50dde7beb2684f19848f3db818e5a64fb351277fa3ef792aaa"
                        },
                        {
                            "range": {
                                "from": "557568",
                                "to": "558079"
                            },
                            "hash": "d6f6f860805aa5bbaac25b316932b886b61df5aa3d2b4cf4a188505fe02b4bc3"
                        },
                        {
                            "range": {
                                "from": "558080",
                                "to": "559103"
                            },
                            "hash": "bc2e46977b6d4ba7c3ba4cc35db1c7acd05d5b061594aa83d8f98d6894cc4c93"
                        },
                        {
                            "range": {
                                "from": "559104",
                                "to": "561151"
                            },
                            "hash": "0feee099b2fdf32c535d3e119ac5d2fe85610136ca399c34356db75701d46462"
                        },
                        {
                            "range": {
                                "from": "561152",
                                "to": "565247"
                            },
                            "hash": "71859b4947075d89879f6903623e72d286cf927bdc752bad20fdc304f11ab805"
                        },
                        {
                            "range": {
                                "from": "565248",
                                "to": "573439"
                            },
                            "hash": "614ca3bd0bed118052357490af3b7dbead29c1c69f115a183c9bf929a3babebb"
                        },
                        {
                            "range": {
                                "from": "573440",
                                "to": "589823"
                            },
                            "hash": "d1f5f7e41902f3853650f7e560dc2c97ecc00e7679f04d91ed9259bc0d2ad1c1"
                        },
                        {
                            "range": {
                                "from": "524288",
                                "to": "557055"
                            },
                            "hash": "9e3d43b74e8bb136f131ade5cde4d3a06f88c6eb884b1d1b1dc26d34e2a8f1b0"
                        },
                        {
                            "range": {
                                "from": "589824",
                                "to": "655359"
                            },
                            "hash": "bb594bd2451f84da04e08e7546c769ed356564e64603409864f07bfba0a53960"
                        },
                        {
                            "range": {
                                "from": "655360",
                                "to": "786431"
                            },
                            "hash": "0e9ef9d0354896086a7943b076aa7ddd0d0dd94d30542b82d90735e9a080a32a"
                        },
                        {
                            "range": {
                                "from": "786432",
                                "to": "1048575"
                            },
                            "hash": "4d4dec78fc598560811d42adc2744d7f2499c0b684414c355f6f3b577bfe74fe"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "524287"
                            },
                            "hash": "deee6f72a764e18887a16d96006b2c35f23c411d273805e4f10827151cba7a2a"
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
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "00c6c6dfcdb39492e8b94ef1c8204bfaacfabf71ecd701549d2c71c07c29db52",
                        "masked_value": "6089238337223914626",
                        "masked_token_id": ""
                    },
                    "target_key": "6aeca9fbfa84f2c627f81df4b7fee6cb753f16e46242b2ed39b15056f6936d32",
                    "public_key": "c412d12371888eb88e3ccb6b1ee964ee05769bfc86c610b35fb5cc6b38fbe018",
                    "e_fog_hint": "d34562346dc4d6f38a38690da144fe1b079317a238e5d05a8e047770057eed019c44d85bfce0ac74b501cf34f1144131bf9a30296b29c6b20bd6a6d1f8b779387c1c674095e1a68bbbe9be50d458b6ff28850100",
                    "e_memo": ""
                },
                {
                    "index": "1749654",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "1749654",
                                "to": "1749654"
                            },
                            "hash": "c8b97203fb9f2e1f1de45c6eace9e2fd024cc06d7e03f6dea88b1267d845e739"
                        },
                        {
                            "range": {
                                "from": "1749655",
                                "to": "1749655"
                            },
                            "hash": "31ebe742e529fb2712d254d2c63a0e7b1997a99ad09f4e40e71d870ad3854770"
                        },
                        {
                            "range": {
                                "from": "1749652",
                                "to": "1749653"
                            },
                            "hash": "8b79efc8c8fdc9ceb0d7193769802f93348573ef71e4e2610738defc2e7f168e"
                        },
                        {
                            "range": {
                                "from": "1749648",
                                "to": "1749651"
                            },
                            "hash": "96d468cf7771f0de9006100541ac3043335c2caafe3f9f2f573f3a796b7dbca6"
                        },
                        {
                            "range": {
                                "from": "1749656",
                                "to": "1749663"
                            },
                            "hash": "9852fcc805960f26b6e7d00060e1fe1747285761b993dec16e75c822e97160a2"
                        },
                        {
                            "range": {
                                "from": "1749632",
                                "to": "1749647"
                            },
                            "hash": "404646858c2805565694cdd564321b0eed8dadb34440e548de15925ae9c0b5fd"
                        },
                        {
                            "range": {
                                "from": "1749664",
                                "to": "1749695"
                            },
                            "hash": "1be00159348d705c9dc8c17d654006d8493df354e042c7271600d816fbbcda1d"
                        },
                        {
                            "range": {
                                "from": "1749696",
                                "to": "1749759"
                            },
                            "hash": "fa55101758a7cbf754823340e1babe1224edbc117f98475ed019ec3185e2dd62"
                        },
                        {
                            "range": {
                                "from": "1749504",
                                "to": "1749631"
                            },
                            "hash": "e53ad4f57c1f4a3c4d19dac7ee79af257ae167d5faefdbc5108b2e286ef1c17c"
                        },
                        {
                            "range": {
                                "from": "1749760",
                                "to": "1750015"
                            },
                            "hash": "0a8ab26b33813d98db58cb0f2322cdc380e6a841ad007a26e05a3f0e3ba2407e"
                        },
                        {
                            "range": {
                                "from": "1748992",
                                "to": "1749503"
                            },
                            "hash": "9343a5116332930b74971fb77af042be47660b79f9aaf60793cebfc1634673f8"
                        },
                        {
                            "range": {
                                "from": "1750016",
                                "to": "1751039"
                            },
                            "hash": "2d48145124bc45492cfb36032b8fc588aabe89d5b483d196493f7dc4a04f8eac"
                        },
                        {
                            "range": {
                                "from": "1751040",
                                "to": "1753087"
                            },
                            "hash": "00ecdaf472d7993d351acbaa5d3d8c03ca1783d738923a3b1813c477578411c8"
                        },
                        {
                            "range": {
                                "from": "1744896",
                                "to": "1748991"
                            },
                            "hash": "eb09e41583e9248b83f29c0f9893bdcd5d40c2f2bbe75272f83ca8059f19beaa"
                        },
                        {
                            "range": {
                                "from": "1736704",
                                "to": "1744895"
                            },
                            "hash": "4ed922041e882a5f7e92648ff8be293bf89fe3a39f77f6e8c33a84d9909871a5"
                        },
                        {
                            "range": {
                                "from": "1753088",
                                "to": "1769471"
                            },
                            "hash": "aca717bccf00ce190189f093eaf3757639877b5c9c6be444c9a5cc8d9d02c034"
                        },
                        {
                            "range": {
                                "from": "1703936",
                                "to": "1736703"
                            },
                            "hash": "a4ef3bd3a74469086910609e60c49340fe2910d2f9b154cca6969e6f3198beb0"
                        },
                        {
                            "range": {
                                "from": "1769472",
                                "to": "1835007"
                            },
                            "hash": "4e634a8203c25419cd532101184f5644849269fdf786ad997490174032761db9"
                        },
                        {
                            "range": {
                                "from": "1572864",
                                "to": "1703935"
                            },
                            "hash": "8ff2dfe8fc1e46ba72e84b103360d11d207beaf0f9e64bacc902e31cb38003ec"
                        },
                        {
                            "range": {
                                "from": "1835008",
                                "to": "2097151"
                            },
                            "hash": "ea05310be210cf5cffa37a42cbc279b05130748571085a8b310f18c888cb5e45"
                        },
                        {
                            "range": {
                                "from": "1048576",
                                "to": "1572863"
                            },
                            "hash": "32da5beee3189a34fa5600bf13ff9bd6b7728988c428adddc61b384aabf3595a"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "1048575"
                            },
                            "hash": "84c55b30648860a50aa0972473b8f24dd7dfb87ba94ddd2a2f23965a9d8e715c"
                        },
                        {
                            "range": {
                                "from": "2097152",
                                "to": "4194303"
                            },
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "1659abfbc6b69e2b2b8e43bcc89c2b8ed7623458fc1217362c1b018bf4bc9844",
                        "masked_value": "6513773625834171169",
                        "masked_token_id": ""
                    },
                    "target_key": "daf256196157db904c98ae105ca5ca7cfdeb33a69d9274304c16fb6f1d5d885d",
                    "public_key": "08237e4d8eb7fa9eb7dda73137325f3e34864477452034eb2ef0004330fbc162",
                    "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "e_memo": ""
                },
                {
                    "index": "51654",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "51654",
                                "to": "51654"
                            },
                            "hash": "1954b9d23a4315a15a3acfedcfe894703fd28dba24b9a7bb33083f0148cc8df0"
                        },
                        {
                            "range": {
                                "from": "51655",
                                "to": "51655"
                            },
                            "hash": "e9a5343b30e22a058a4b69c2895c007b6fa51dea15e8d156d84aa4766771b0a5"
                        },
                        {
                            "range": {
                                "from": "51652",
                                "to": "51653"
                            },
                            "hash": "9091712a3c292144f6b777ef1bcdc78bd65bbd811f8e302422b8104179327fea"
                        },
                        {
                            "range": {
                                "from": "51648",
                                "to": "51651"
                            },
                            "hash": "51fe9ce55cc0834ec4dc4c2158c0acd13fe1c3d220d5ae2cf0e4b26d73759816"
                        },
                        {
                            "range": {
                                "from": "51656",
                                "to": "51663"
                            },
                            "hash": "966edf4f5e7a9001e7340db3e4e789408f90373314646df8b34fb20958cafe9b"
                        },
                        {
                            "range": {
                                "from": "51664",
                                "to": "51679"
                            },
                            "hash": "8f80a3dfc59d12ea5c60c8c347351925fff9a8b85679778a4c50bd32d98ae393"
                        },
                        {
                            "range": {
                                "from": "51680",
                                "to": "51711"
                            },
                            "hash": "44d1674c90a266adadbe5917aef9f4e2775907944c854c05502822587106452e"
                        },
                        {
                            "range": {
                                "from": "51584",
                                "to": "51647"
                            },
                            "hash": "45884506b9a3f06f7c1b27b871809b89a674c5c550e796bb8349de20158260e4"
                        },
                        {
                            "range": {
                                "from": "51456",
                                "to": "51583"
                            },
                            "hash": "50cfdd1136c23c13aa57616d5d5cd61461f78d7358da7c9f566c6a9cbdef6b02"
                        },
                        {
                            "range": {
                                "from": "51200",
                                "to": "51455"
                            },
                            "hash": "9c84bed489923d0368965a19d47ef03717bc889022307fb5bb60e5a4075ba973"
                        },
                        {
                            "range": {
                                "from": "51712",
                                "to": "52223"
                            },
                            "hash": "fce5ac33177877494efb34d3e3f6996016f5c76734515a09e66f59233d399c0a"
                        },
                        {
                            "range": {
                                "from": "52224",
                                "to": "53247"
                            },
                            "hash": "78bf98d383ec120ed6aaf72983892f9f18f918f3bf1d9ebfbf8608272e2bfdf5"
                        },
                        {
                            "range": {
                                "from": "49152",
                                "to": "51199"
                            },
                            "hash": "2c255a6baddd65da35f7309c75b031ba1130b26eead7870dd819f13fffcd3255"
                        },
                        {
                            "range": {
                                "from": "53248",
                                "to": "57343"
                            },
                            "hash": "f4c79a321ecbf4764b63008ad0a457fbc77405c85d905961e1398983061e6767"
                        },
                        {
                            "range": {
                                "from": "57344",
                                "to": "65535"
                            },
                            "hash": "8d2069be00c51f8365eb2bd180ff35d770feb2d031a6fed8c38105a1f2a5394b"
                        },
                        {
                            "range": {
                                "from": "32768",
                                "to": "49151"
                            },
                            "hash": "660be6aa8140beaeba811c8ef76e5597156850c629ed50aabd41203f56f803dd"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "32767"
                            },
                            "hash": "f51fe6feb690e118f4f0c78ce034adec04123f146d8a98f78aac3bdadc0b7199"
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
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "bad93e66bba18272880a840d005cd9d7eee97d18992919bbb2ea4479f1800f6e",
                        "masked_value": "10162198986509262383",
                        "masked_token_id": ""
                    },
                    "target_key": "b079199d926c093b7c106421039950dbba2fd441cfd0f2dd82c005bc9e5d747c",
                    "public_key": "387226027c41dfb30787c2e8f0bba28478add55195e447e6ffd9d6018a166522",
                    "e_fog_hint": "e46b9a2a2a6083cca8468530f1d2bd1494259ccbad7c743bd1c7b689d129ac19b069f4489346b78b7491421395372b3a9a0d0a0d77c1ca05b74f07dde17c1888693eaa0dcd8a4779715bce99363d7c759e170100",
                    "e_memo": ""
                },
                {
                    "index": "595927",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "595927",
                                "to": "595927"
                            },
                            "hash": "d5bb6bb04eb366c97b455c97357bb131a16e81949791f6bd15384c7be3f1ebf4"
                        },
                        {
                            "range": {
                                "from": "595926",
                                "to": "595926"
                            },
                            "hash": "0ef2df1d2285364afb594af468ebebba5683a52ed09ec2441592afe5bb7673b2"
                        },
                        {
                            "range": {
                                "from": "595924",
                                "to": "595925"
                            },
                            "hash": "cb8aaa1118aa2c69a752f678cdfbad0d6caa9c60184267f7808b9433a81a339f"
                        },
                        {
                            "range": {
                                "from": "595920",
                                "to": "595923"
                            },
                            "hash": "3e01824270586e29b51c60d70996eb6d3478c5877830bbd452047a2003899679"
                        },
                        {
                            "range": {
                                "from": "595928",
                                "to": "595935"
                            },
                            "hash": "f434b080e725455a532e10ac562dd9319f87e2eaaeb6e7c85060c9b9036ac611"
                        },
                        {
                            "range": {
                                "from": "595904",
                                "to": "595919"
                            },
                            "hash": "b93a0ca8e88f84b5bc76ee4a304be2651d7c7f330714951c76417fdd3ffc5d4f"
                        },
                        {
                            "range": {
                                "from": "595936",
                                "to": "595967"
                            },
                            "hash": "c45e5a595b5de9241b878d830126b67426d4b5454535efe7a7e14503cd7d978b"
                        },
                        {
                            "range": {
                                "from": "595840",
                                "to": "595903"
                            },
                            "hash": "cfd0c27865dd384461268fb9bcb95e1732bd0c469eb4b133af8282b1a242c9dc"
                        },
                        {
                            "range": {
                                "from": "595712",
                                "to": "595839"
                            },
                            "hash": "0ac2cd19aa48996b11ea9f7aa80f6623116aa97f87af9cb9695046b3416a9ab8"
                        },
                        {
                            "range": {
                                "from": "595456",
                                "to": "595711"
                            },
                            "hash": "3cc4f30d8c9a9caa5d8876e2221221e9fc84e76ea23cec1ae32e558d8b6dfec6"
                        },
                        {
                            "range": {
                                "from": "594944",
                                "to": "595455"
                            },
                            "hash": "d936e0ae4510be18250e31a79edc57cb99f9d26929c8a26d1608620f350e5045"
                        },
                        {
                            "range": {
                                "from": "593920",
                                "to": "594943"
                            },
                            "hash": "3395c43fa21adc940009ec676b578f0dc91c0f3850d21008c5a8276116d2aa40"
                        },
                        {
                            "range": {
                                "from": "595968",
                                "to": "598015"
                            },
                            "hash": "1042f55010315d8f7575a0d3aa69659c946c1082cb68d509f13ee53de15822ff"
                        },
                        {
                            "range": {
                                "from": "589824",
                                "to": "593919"
                            },
                            "hash": "18637bbe64c68b448cf793c5671d208734151060b6add159132f546244f3ea1b"
                        },
                        {
                            "range": {
                                "from": "598016",
                                "to": "606207"
                            },
                            "hash": "e901e94f252e816a59fbbee3d367caa98571dd37b8ade8bd382964bebb2fc43f"
                        },
                        {
                            "range": {
                                "from": "606208",
                                "to": "622591"
                            },
                            "hash": "fb5c97bdd7cc1ce996f2bd5a65921d4e4eeba4bf2619e98bbd35fcea3bc2fd9e"
                        },
                        {
                            "range": {
                                "from": "622592",
                                "to": "655359"
                            },
                            "hash": "3f2069c6a8ee5b0d4e4b10ba783819d5b46c0bf69e9dc7c392415fafe345fdcd"
                        },
                        {
                            "range": {
                                "from": "524288",
                                "to": "589823"
                            },
                            "hash": "91ac1d42a7874b1fa0cba1450ba5de987215d690d128aeda9b6841421149001a"
                        },
                        {
                            "range": {
                                "from": "655360",
                                "to": "786431"
                            },
                            "hash": "0e9ef9d0354896086a7943b076aa7ddd0d0dd94d30542b82d90735e9a080a32a"
                        },
                        {
                            "range": {
                                "from": "786432",
                                "to": "1048575"
                            },
                            "hash": "4d4dec78fc598560811d42adc2744d7f2499c0b684414c355f6f3b577bfe74fe"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "524287"
                            },
                            "hash": "deee6f72a764e18887a16d96006b2c35f23c411d273805e4f10827151cba7a2a"
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
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "c25de8e6b883ff5f1fe80a6e3945f063f201df0066a2fa12a4946dd045e4750a",
                        "masked_value": "2692248685967633359",
                        "masked_token_id": ""
                    },
                    "target_key": "3a6fc6e452369bc7cca072b7c20c2f19eccc6aa5faf994d064f3ec64fdb5100f",
                    "public_key": "9eba76a221439f0ad2f1da102327c13bb88ba6c0d1ea71a31708d25de95c5632",
                    "e_fog_hint": "b23be95cf960470f931c42af401f13549f94fa167b8f4be1d3060f62291c714fbb404ea2d4786f6e4d7d2b9bcd5dde4776bc87ce0a6c22a4c60aeb4742f615a703685889b1ecb94a252bee9cbacfadcd37800100",
                    "e_memo": ""
                },
                {
                    "index": "1010544",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "1010544",
                                "to": "1010544"
                            },
                            "hash": "4974e78a1d57414af8e928ba888f245586e44d35e4c0ad583b93f415f3054ca5"
                        },
                        {
                            "range": {
                                "from": "1010545",
                                "to": "1010545"
                            },
                            "hash": "1ca98ae508cb5f00f69f6741d0eaa04a8ec2efd8523078261e83dbc1f1583534"
                        },
                        {
                            "range": {
                                "from": "1010546",
                                "to": "1010547"
                            },
                            "hash": "1d2f7660d5fa86ea2d5f34a4dd5915828829f3549290dfe5faac548a46eaa543"
                        },
                        {
                            "range": {
                                "from": "1010548",
                                "to": "1010551"
                            },
                            "hash": "6d25c266263018e4656adcbc83f737c2e96d83108a30d21d3a28b3bee3b0f45b"
                        },
                        {
                            "range": {
                                "from": "1010552",
                                "to": "1010559"
                            },
                            "hash": "fa38eab6de86d489166666443f7b19e603e0821140fd6d28df111bf065fc2b62"
                        },
                        {
                            "range": {
                                "from": "1010528",
                                "to": "1010543"
                            },
                            "hash": "d366ad00e09561e56cd93246ea5d1ad5ac49c054c3ba18bc2ee84de7caa149e3"
                        },
                        {
                            "range": {
                                "from": "1010496",
                                "to": "1010527"
                            },
                            "hash": "d754d737f4ac224fcc877f523936bf2ed432896080f51a36f94ec3b2a2ccafa0"
                        },
                        {
                            "range": {
                                "from": "1010432",
                                "to": "1010495"
                            },
                            "hash": "91282d639ad0095dbce3f88180cf535699028ed888030cc807d14cda52382d5e"
                        },
                        {
                            "range": {
                                "from": "1010560",
                                "to": "1010687"
                            },
                            "hash": "da68c0026534a9ed1728877d138ab933ff502f9b5f8f493a9b61c005550928a4"
                        },
                        {
                            "range": {
                                "from": "1010176",
                                "to": "1010431"
                            },
                            "hash": "e4522fde277471b4e85ae974e011257c3af795c60fdb6513f977b99259d37a8e"
                        },
                        {
                            "range": {
                                "from": "1009664",
                                "to": "1010175"
                            },
                            "hash": "ffca7886ecbc1d2dc6c23b9b7b61307570c128616f9ee83a687c1517b4c56d61"
                        },
                        {
                            "range": {
                                "from": "1010688",
                                "to": "1011711"
                            },
                            "hash": "635a339e3cca26684d41338dcfd284a35ee50b9374b69bd5d98608fdadabc952"
                        },
                        {
                            "range": {
                                "from": "1007616",
                                "to": "1009663"
                            },
                            "hash": "02fbe2ef44a25a79811279e2728738cbc07cdade3328e42adf598728bac6ca90"
                        },
                        {
                            "range": {
                                "from": "1011712",
                                "to": "1015807"
                            },
                            "hash": "7ff650f01fb509b2f7e18253fbb127eb6dc2bb728abff985a092b00ee1af7d3e"
                        },
                        {
                            "range": {
                                "from": "999424",
                                "to": "1007615"
                            },
                            "hash": "64ed6c185ab0e0c73613835c847c9cb89bcb9079946a7881b485752c3c5b6d38"
                        },
                        {
                            "range": {
                                "from": "983040",
                                "to": "999423"
                            },
                            "hash": "beded407fd041a52bb6505b457d9bb41f60f0ca5ad5c8bd607cf4e966016deb4"
                        },
                        {
                            "range": {
                                "from": "1015808",
                                "to": "1048575"
                            },
                            "hash": "e7b7cc91a9386735957bc07864887e3d19b75feadc07a3223b1f096f8ba9d67e"
                        },
                        {
                            "range": {
                                "from": "917504",
                                "to": "983039"
                            },
                            "hash": "efb00d3ca2c2aeade7a6ee9fd1d6e2682e24045c89c7e93c86041900408987ed"
                        },
                        {
                            "range": {
                                "from": "786432",
                                "to": "917503"
                            },
                            "hash": "98416c374da499d980f6145feced5ed46b2ffe01b00fbfd993b27a560f3594ed"
                        },
                        {
                            "range": {
                                "from": "524288",
                                "to": "786431"
                            },
                            "hash": "92f6d50bccb54547f39bd2e57207ac7194a9cdf2a4b12fdee6258559d99d2cad"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "524287"
                            },
                            "hash": "deee6f72a764e18887a16d96006b2c35f23c411d273805e4f10827151cba7a2a"
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
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ],
            [
                {
                    "masked_amount": {
                        "commitment": "9e4a38f849cf414722680cb9b85a24f90ee02c9d65023212dae47e592c3be731",
                        "masked_value": "615345482596651732",
                        "masked_token_id": ""
                    },
                    "target_key": "ea05aa0c5a10029b4d6c9d8616f42c1ef99603f2a3a8938dab6802c2527f8e05",
                    "public_key": "6c4316f9416ed29dd7942a6594f9de943f2bc64b5d91f5ff9fc17a5409d39b0f",
                    "e_fog_hint": "cab22947bbb778085e4e75fd3b295e602fdef0213e7925dc7e10eacbbfc4db73e4749ea0ffdba7d2bf8dc5a54ccec102d87b1926903da62f5e4c6a5b166822f09e06243b245d9c6acd99bf15af54b58fc9020100",
                    "e_memo": ""
                },
                {
                    "index": "766515",
                    "highest_index": "2850534",
                    "elements": [
                        {
                            "range": {
                                "from": "766515",
                                "to": "766515"
                            },
                            "hash": "e19ed8ecfff73c16f71a1a12257c48bb0efbdca872b5c6b443275d3915d130c5"
                        },
                        {
                            "range": {
                                "from": "766514",
                                "to": "766514"
                            },
                            "hash": "067e6939cf11e5ae9d3a796943c9b1b64436151cf6720f37d773c858e5d47f6a"
                        },
                        {
                            "range": {
                                "from": "766512",
                                "to": "766513"
                            },
                            "hash": "f5a96f35e3c1825753110ef45bd3bd88603bf65f2570b1aefd85c86d7e283a23"
                        },
                        {
                            "range": {
                                "from": "766516",
                                "to": "766519"
                            },
                            "hash": "9afeb28e09016144c65218555225ab66b8e5571fd1889b05bc2fdf2a6d07ba76"
                        },
                        {
                            "range": {
                                "from": "766520",
                                "to": "766527"
                            },
                            "hash": "12b7c23d186b65209f5c923fdf76b142ea03874c633e816492455709a4a53278"
                        },
                        {
                            "range": {
                                "from": "766496",
                                "to": "766511"
                            },
                            "hash": "cfeed28e6c832728a24abd33b97d2b283ea0c0ae5b624ebbcef28865a021282e"
                        },
                        {
                            "range": {
                                "from": "766464",
                                "to": "766495"
                            },
                            "hash": "cab491467ea75fddd9cc5100555468288637d97664c7ed4b2edd589f33c09e66"
                        },
                        {
                            "range": {
                                "from": "766528",
                                "to": "766591"
                            },
                            "hash": "9dbbcf7c27466d7a69494e93a625832ac8519f4d27f60c691f0775b9323b444b"
                        },
                        {
                            "range": {
                                "from": "766592",
                                "to": "766719"
                            },
                            "hash": "02e2885a2f00682ff50b04ed9a9da960be5acb2ac6d94598a9c85edc4ae4d828"
                        },
                        {
                            "range": {
                                "from": "766720",
                                "to": "766975"
                            },
                            "hash": "0c02577d5e7000b112c4f3008d11f3b766469a81c90d17be6d3d987dceb3ddc8"
                        },
                        {
                            "range": {
                                "from": "765952",
                                "to": "766463"
                            },
                            "hash": "1e1e9da9141e37945d2a2f8f915feb466dde2144fc14ab88fd7654ca15709137"
                        },
                        {
                            "range": {
                                "from": "766976",
                                "to": "767999"
                            },
                            "hash": "73cccbdb06b2d350c4b02bd84ac1c6de8e2a085aaa248e03eb914b73e54af4ee"
                        },
                        {
                            "range": {
                                "from": "768000",
                                "to": "770047"
                            },
                            "hash": "0fef42eadd4ad1ba8e193723718179bdb91b9143dcc940a22b39d33f75da4795"
                        },
                        {
                            "range": {
                                "from": "761856",
                                "to": "765951"
                            },
                            "hash": "bd110aa67d22f67bd59b50806edfb389a2e5f97435331e8038c9015f308779ac"
                        },
                        {
                            "range": {
                                "from": "753664",
                                "to": "761855"
                            },
                            "hash": "189615607c11341fbfe8db59f7080941af52e4f35dd6bf3a76ddcdecdc23b440"
                        },
                        {
                            "range": {
                                "from": "770048",
                                "to": "786431"
                            },
                            "hash": "42ddfc9bf479f5fafef423f1818037abf8ac5432cafbc31295e809c83b7bfdfd"
                        },
                        {
                            "range": {
                                "from": "720896",
                                "to": "753663"
                            },
                            "hash": "3634f2475a02518fcbc90b047b1c1815c084f3c860550bda3a7db01533c9bd3f"
                        },
                        {
                            "range": {
                                "from": "655360",
                                "to": "720895"
                            },
                            "hash": "3d5a33433eeff530b10e938bd9ae7fa768cadd7f9ad1f317061732ed9b44e5c3"
                        },
                        {
                            "range": {
                                "from": "524288",
                                "to": "655359"
                            },
                            "hash": "dcab3fbcdc604f5b66eca6799be4d6d934301a386c42324fe7e50097cf3c330a"
                        },
                        {
                            "range": {
                                "from": "786432",
                                "to": "1048575"
                            },
                            "hash": "4d4dec78fc598560811d42adc2744d7f2499c0b684414c355f6f3b577bfe74fe"
                        },
                        {
                            "range": {
                                "from": "0",
                                "to": "524287"
                            },
                            "hash": "deee6f72a764e18887a16d96006b2c35f23c411d273805e4f10827151cba7a2a"
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
                            "hash": "09eb65894764d0d04ce70a2ebefd8ca443b8b33089600a38ad6d4cc506a80e47"
                        }
                    ]
                }
            ]
        ]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}

