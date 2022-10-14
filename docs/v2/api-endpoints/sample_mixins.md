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
        "num_mixins": 2,
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
            {
                "masked_amount": {
                    "commitment": "268bca85a8bf01d775f98952788ef2eaf48618e0ac4dbb642426ac270f63e501",
                    "masked_value": "4148226062671934601",
                    "masked_token_id": ""
                },
                "target_key": "46e18441764ca38f669abd609cb04aa1961ba3c57855f363c45045117006260e",
                "public_key": "8a28ebd659c9914343427121a3cc3b5b527a97805ce11ca1d6b16568326ffe22",
                "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "e_memo": ""
            },
            {
                "masked_amount": {
                    "commitment": "887a666054c0366bdf951ea84af07d25de6267301fb5b841be3fc412ed9a4470",
                    "masked_value": "1551464221591799897",
                    "masked_token_id": ""
                },
                "target_key": "c68e49d858ff75d150f8441759a2e7bf3ff187306b7a03020104fcac34929439",
                "public_key": "cc0d1969185915b0dfb1d6bef089d9ac3be214e5f40b8aa3332b60827a48ac45",
                "e_fog_hint": "1d19b3e329b61410e42a20675473fa9583b3e4fbfa803f8933cf6a7b0d29dbf51a8e1a0f3bb5e7be496acdb9c4d0fcb4363a8d4a53601d59a2378d7cf7a3344e9414da8bac896edb0591d90bc51ea658c69a0100",
                "e_memo": ""
            }
        ],
        "membership_proofs": [
            {
                "index": "643891",
                "highest_index": "2954595",
                "elements": [
                    {
                        "range": {
                            "from": "643891",
                            "to": "643891"
                        },
                        "hash": "16439812756f65a2bad5bd7df813aa318979b465220b3c3654b1474b5f42cf24"
                    },
                    {
                        "range": {
                            "from": "643890",
                            "to": "643890"
                        },
                        "hash": "88c2d7d82a4d45510651058fa8c4c0f7c4baa7ddb8fe101f31492586b089b0f0"
                    },
                    {
                        "range": {
                            "from": "643888",
                            "to": "643889"
                        },
                        "hash": "1252defa6d3289a7c9d24460d399fb7681e36a479c82076d0857448c9e206daf"
                    },
                    {
                        "range": {
                            "from": "643892",
                            "to": "643895"
                        },
                        "hash": "ad09a8f8ce75974c2237129f43360c2a357177886c48186183c60d96b625f29b"
                    },
                    {
                        "range": {
                            "from": "643896",
                            "to": "643903"
                        },
                        "hash": "119ed507664cfef1da5c17378b08b6b0e4974c20a08a213eb308e8540c907547"
                    },
                    {
                        "range": {
                            "from": "643872",
                            "to": "643887"
                        },
                        "hash": "8aa944350fb0d0595bda4035939f581c4fe25e8d0554aa8f2b9235b40fec0b63"
                    },
                    {
                        "range": {
                            "from": "643840",
                            "to": "643871"
                        },
                        "hash": "6da268a3f750752ea70fe9c49af2df27cecae71568541efe06bea5589e60148f"
                    },
                    {
                        "range": {
                            "from": "643904",
                            "to": "643967"
                        },
                        "hash": "ba4199316471f0c7fc262093a0b77ff027e7a983acc0cfef838e86c6f08942a0"
                    },
                    {
                        "range": {
                            "from": "643968",
                            "to": "644095"
                        },
                        "hash": "eb470ef71de14005d27d51be6774d123bea003dda316618e308ed5fba9a82e81"
                    },
                    {
                        "range": {
                            "from": "643584",
                            "to": "643839"
                        },
                        "hash": "3ad7566ffaa14b05695a30e25e916fdb8791318b32f3ca05360a5324391a8cee"
                    },
                    {
                        "range": {
                            "from": "643072",
                            "to": "643583"
                        },
                        "hash": "5ffbfbf22f33738dc5d5d3aa5f7ed2dcabc59bd662132ee7e660a23b6bef668e"
                    },
                    {
                        "range": {
                            "from": "644096",
                            "to": "645119"
                        },
                        "hash": "a6d774dec28e475b01dd3418c4f3698b303bcc22e89c73d74de0c4f4fb55fda6"
                    },
                    {
                        "range": {
                            "from": "645120",
                            "to": "647167"
                        },
                        "hash": "69855c4cd1af89d7fd3b498da26e9aafe25eaed453a6435c82a270cae786b6ed"
                    },
                    {
                        "range": {
                            "from": "638976",
                            "to": "643071"
                        },
                        "hash": "c0f365bb448b01c7c15aee1ed599929b26a93aa29eb277556d38522fc0e84816"
                    },
                    {
                        "range": {
                            "from": "647168",
                            "to": "655359"
                        },
                        "hash": "75698e4045eb48ad902a9eba22c1cf57e39172e37e3fb4f500c019b0f4753e9f"
                    },
                    {
                        "range": {
                            "from": "622592",
                            "to": "638975"
                        },
                        "hash": "1021ca2ad629be7f6939c123e868e08dd57dcdd3fcbf833a48cf1a50cb3d56f1"
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
                        "hash": "ff083a72249806122533f32baa14253f6bf3801d7b1f9a804ab86cd15e7542a9"
                    }
                ]
            },
            {
                "index": "1441542",
                "highest_index": "2954595",
                "elements": [
                    {
                        "range": {
                            "from": "1441542",
                            "to": "1441542"
                        },
                        "hash": "361dd7f0a49848ef8af280ac9ab0293edf017ec8bcf77ec386243621babaf1a8"
                    },
                    {
                        "range": {
                            "from": "1441543",
                            "to": "1441543"
                        },
                        "hash": "9fb233507554c39f480c07f95abf81efa26509060027f1fbe2fd8a3dfaba4b20"
                    },
                    {
                        "range": {
                            "from": "1441540",
                            "to": "1441541"
                        },
                        "hash": "03a6468c2ecc6470ef9afa8805f1d00ecab0bd837e2b9a8ddaa5a49bc4bf57d4"
                    },
                    {
                        "range": {
                            "from": "1441536",
                            "to": "1441539"
                        },
                        "hash": "767d515ad4f9c5c87acaa9375ddd71a219f710b4bc9b9df65f15e8de65e2f90d"
                    },
                    {
                        "range": {
                            "from": "1441544",
                            "to": "1441551"
                        },
                        "hash": "c8cb3365172d07041be5f233bb949ba3d7427d725b605f41c12a838108dcf380"
                    },
                    {
                        "range": {
                            "from": "1441552",
                            "to": "1441567"
                        },
                        "hash": "8e239fa9d67e4de48132720c4223df43d5ec94b696e36afa1d998c1208fd84a7"
                    },
                    {
                        "range": {
                            "from": "1441568",
                            "to": "1441599"
                        },
                        "hash": "506b80222a94a324b25ae59f54718c2c152310cc24779764feeb359fb212df7d"
                    },
                    {
                        "range": {
                            "from": "1441600",
                            "to": "1441663"
                        },
                        "hash": "11da0472aacf7228baa1d620a2112c85b36e2e52a089ae6ae67ba93fc015a277"
                    },
                    {
                        "range": {
                            "from": "1441664",
                            "to": "1441791"
                        },
                        "hash": "2c76256cbf719f32916a7c9ed7a343ca31f374347c0e8b42fc047dd132b36f06"
                    },
                    {
                        "range": {
                            "from": "1441280",
                            "to": "1441535"
                        },
                        "hash": "917cdf2fbdc8a18cffd6b2beaed2e75f884900cd9553725df18786898812c800"
                    },
                    {
                        "range": {
                            "from": "1440768",
                            "to": "1441279"
                        },
                        "hash": "50ed8c28966abff98d024cc14019fa64fe714c47ffae23c1cc2f0acb51b1c6fe"
                    },
                    {
                        "range": {
                            "from": "1439744",
                            "to": "1440767"
                        },
                        "hash": "c8bc95e8ab8380473b7d5727a58d859539066baffbf170b8f2f9e7b74245baf6"
                    },
                    {
                        "range": {
                            "from": "1437696",
                            "to": "1439743"
                        },
                        "hash": "66f969f7e412fb08bba0ccd891fe3b0047ad69c5519cf06f0d77add0132d6da0"
                    },
                    {
                        "range": {
                            "from": "1433600",
                            "to": "1437695"
                        },
                        "hash": "90a555645cfa3bd6d8e3e0d51960bcda6480ae639aa05b0facf1f5e8d837e636"
                    },
                    {
                        "range": {
                            "from": "1425408",
                            "to": "1433599"
                        },
                        "hash": "5f39939869191af4cf196da51cdf326dd43bb4626d7a8e50e81d57a30fdc4411"
                    },
                    {
                        "range": {
                            "from": "1409024",
                            "to": "1425407"
                        },
                        "hash": "ecde44bae54f53cf3caef36da0ec353a42903faee2dd7d3159b422fa6af8df0a"
                    },
                    {
                        "range": {
                            "from": "1376256",
                            "to": "1409023"
                        },
                        "hash": "b3ce002cc4940f56dbd8b29a875d8e0d81ecae2a4c6abf502d9fce01a1e77292"
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
                        "hash": "ff083a72249806122533f32baa14253f6bf3801d7b1f9a804ab86cd15e7542a9"
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

