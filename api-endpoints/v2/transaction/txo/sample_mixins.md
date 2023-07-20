---
description: Sample a desired number of mixins from the ledger, excluding a list of tx outs
---

# Sample Mixins

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param     | Purpose                        | Requirements                                                                        |
| ------------------ | ------------------------------ | ----------------------------------------------------------------------------------- |
| `num_mixins`       | The number of mixins to sample | Must be less than the number of txos in the ledger minus number of excluded outputs |
| `excluded_outputs` | Txos to exclude from sampling  | Txo must exist in the ledger                                                        |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Body Request" %}
```json
{
  "method": "sample_mixins",
  "params": {
    "num_mixins": 2,
    "excluded_outputs": [
      {
        "masked_amount": {
          "commitment": "a8b9496bfe9a95a3cfbae1fda980ce2a1fa7e2827da6916de204ae12d094210a",
          "masked_value": "5154035447619221459",
          "masked_token_id": "3db375a7f674e6cc",
          "version": 2
        },
        "target_key": "bcaa42886171e60c50f0a4527663507a890fbecb5016f6d9042ce6be1cd7fb52",
        "public_key": "cecc879afd79153210ff79b58947416a883d4f68253d415533c0e8898e09f045",
        "e_fog_hint": "643db209825ced0df98a277c989b9d1876ac4009397137af1fabd3856c7c97dd629be47752cd532aa1f4bb1412d4dac9a76d50e67b4b99da017dc3a40caa99b4933ef6b4b51c56a338fc8648244eba5a22d90100",
        "e_memo": "13d173c60b40cfe99de248d38166f99e5cfcd45327b03dc46b1dd0147e78f4c19f881afe2f56e50da8743597d6eec8c6e44336e606dd235e8b7edca15a5d7a0c9c08"
      }
    ]
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
          "commitment": "0838c08221f219c329fc5566806991372ab17c90107d7bba74e2c6308feea11c",
          "masked_value": "97441741736397669",
          "masked_token_id": "e6830f30fb15b31c",
          "version": null
        },
        "target_key": "22d1b72752d44efbf110d0ea15bda320a75f4d973cd96681e862cd84a1f27732",
        "public_key": "f8166fb241230db916aa5626167d554a703e817107b4e14598617f8dd2fd8504",
        "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "e_memo": "056edf036b09af5187dc67ab15c04f6c12321e103b36b1a6a8143da1c4d2589bee9349e17f3cb12ffd31c1fb39b685bf9a7efbc2b60b8b01af3feac95ca55723f968"
      },
      {
        "masked_amount": {
          "commitment": "964dccf8ef6e921c900a843aa16c5aee4ad9c511e66a391efe52f73afe29e959",
          "masked_value": "17562165403859100725",
          "masked_token_id": "",
          "version": null
        },
        "target_key": "66bab59acb88b8c5b33329f308f90aa888a0be701bc650ead4393d7b3393997c",
        "public_key": "d029d3706d9a21211c0caf11bf94f78bdb05701733095fde84947acd949fe91b",
        "e_fog_hint": "1ed5da6044b2542c2a7f522389eaa64e0f8ad7bb5dc05ec57117375ae2c22c5939e5802df2404b60cca2788534a19f4e6693c632037006c5b553421089bc5815bb466bd12847e07de26a681f7d2e3226417e0100",
        "e_memo": ""
      }
    ],
    "membership_proofs": [
      {
        "index": "3459177",
        "highest_index": "5367263",
        "elements": [
          {
            "range": {
              "from": "3459177",
              "to": "3459177"
            },
            "hash": "61e6b248e942db420f4c90d839fbe74fc94a3734dee546f250d4b0bbdf300225"
          },
          {
            "range": {
              "from": "3459176",
              "to": "3459176"
            },
            "hash": "768bce70a88ed872372db33a39569b5a659a2b54b16784d9c81bd524b9dc55b3"
          },
          {
            "range": {
              "from": "3459178",
              "to": "3459179"
            },
            "hash": "9fa3eaa39d83e4af951af4ef18de019dbddc5beaa0464a1429dc8dd61198490b"
          },
          {
            "range": {
              "from": "3459180",
              "to": "3459183"
            },
            "hash": "6b79f9348fdca3577c192fd2003a70849f810151f62002edf6f0cb286aef4228"
          },
          {
            "range": {
              "from": "3459168",
              "to": "3459175"
            },
            "hash": "8aaf6b3c2f43ddbdad26f6e87d26458c506766883e3a96b2dd85ba49528dabe9"
          },
          {
            "range": {
              "from": "3459184",
              "to": "3459199"
            },
            "hash": "0d9d83b66d076a05ad5e582eab37f19a0f1935b8b8f9b3252e518cd7bcff22d8"
          },
          {
            "range": {
              "from": "3459136",
              "to": "3459167"
            },
            "hash": "3fbbd4ba4667f29b27b4e8dfc60972b85e81defd41ddb0521c9682d6fc641291"
          },
          {
            "range": {
              "from": "3459072",
              "to": "3459135"
            },
            "hash": "1d326333fb10869140761d5ba509e5926d7aa80550286b39d6b87720e8599a05"
          },
          {
            "range": {
              "from": "3459200",
              "to": "3459327"
            },
            "hash": "e0f8845f0579d9a70101e54db106dd0a2d6cc74804fea187c70dbb280979846e"
          },
          {
            "range": {
              "from": "3459328",
              "to": "3459583"
            },
            "hash": "01f244ce7a0df9e77ad6eb15e5bad1bd0ab5e28c5cdbbe5dfd9b833ed7da618f"
          },
          {
            "range": {
              "from": "3459584",
              "to": "3460095"
            },
            "hash": "9d5bf512ccf162ee73c46734815e0ae53cbd18ed444227d253bdff26f7a2f5d8"
          },
          {
            "range": {
              "from": "3460096",
              "to": "3461119"
            },
            "hash": "184f133e002a5ada83bdb07c31d287d2ddb09efeadf66a10568a5c760959deb5"
          },
          {
            "range": {
              "from": "3457024",
              "to": "3459071"
            },
            "hash": "05e9639eabd763501958159e0190753b07936b3fe739b69f6b389c36445565be"
          },
          {
            "range": {
              "from": "3461120",
              "to": "3465215"
            },
            "hash": "26a2f9653846e572c650d508d0291aa481f09c571de405792652cd92632de9be"
          },
          {
            "range": {
              "from": "3465216",
              "to": "3473407"
            },
            "hash": "d2f1c1272a9a0fa7be6fce9cc2a302e038e003dd4966b3859e16a26afda9c8bf"
          },
          {
            "range": {
              "from": "3440640",
              "to": "3457023"
            },
            "hash": "87a3bc6e6cdfe86ae51418701117b8108926e137371f0b465494aeb9d9412a0c"
          },
          {
            "range": {
              "from": "3407872",
              "to": "3440639"
            },
            "hash": "0a04738acbda3aaad8ba0e680b10625cda8c683a79a10771bca7dcec41a2fb4a"
          },
          {
            "range": {
              "from": "3473408",
              "to": "3538943"
            },
            "hash": "4b0ecce6804c5de2fc1edbfc239b29676190d8de5d262faf87f5b1463be376d1"
          },
          {
            "range": {
              "from": "3538944",
              "to": "3670015"
            },
            "hash": "43a333d35f08f48333f8c7d5a6de0ee9f433bcc59f1ea97ccbbc09ac81cc8ad8"
          },
          {
            "range": {
              "from": "3145728",
              "to": "3407871"
            },
            "hash": "f0ef92e1014f0385b87cd5e7e4071ebf608c177a842d8bdf2768a7ee8cc1b50c"
          },
          {
            "range": {
              "from": "3670016",
              "to": "4194303"
            },
            "hash": "942a839767026435f15275a7e1c1bae9ee224ffdfa9d51b84180d59136eac349"
          },
          {
            "range": {
              "from": "2097152",
              "to": "3145727"
            },
            "hash": "8a4cd94c536573c79e6c451d101067ce98310e0a82fe04a2a2d4572529062614"
          },
          {
            "range": {
              "from": "0",
              "to": "2097151"
            },
            "hash": "f16256b5f5e635de8e230ad587df3f3d578bc5ba515c77e54d9bedee36e4435b"
          },
          {
            "range": {
              "from": "4194304",
              "to": "8388607"
            },
            "hash": "eb23dd9d58931d5864677e7e08488b7d1f9d3c31698137d7c32065c8b8078009"
          }
        ]
      },
      {
        "index": "1235231",
        "highest_index": "5367263",
        "elements": [
          {
            "range": {
              "from": "1235231",
              "to": "1235231"
            },
            "hash": "6db5358eb5d66ab0ab3acbde6415ffb545c3c5ccc5cb43ca89c5a6cd3f6eb20c"
          },
          {
            "range": {
              "from": "1235230",
              "to": "1235230"
            },
            "hash": "e75feeb99ae45d50bf68790089fe6a8811af2ed3d6d839741ed680537adb0131"
          },
          {
            "range": {
              "from": "1235228",
              "to": "1235229"
            },
            "hash": "2988439e6f19c732389e64835b761541adf5e14920499c6ce3eaf5ef273014fa"
          },
          {
            "range": {
              "from": "1235224",
              "to": "1235227"
            },
            "hash": "0edc18070486119101fd1309c358f95cacb281129822bd84f9f029ba55e984b5"
          },
          {
            "range": {
              "from": "1235216",
              "to": "1235223"
            },
            "hash": "a047d0b22bd0c27b2d432f8a67da942d281c64180fa00cfdcd7fb36c8d5024de"
          },
          {
            "range": {
              "from": "1235200",
              "to": "1235215"
            },
            "hash": "cca9a0f0f102dbc9f344c651291bbf06dbd8ef85096b46c64049665cb13f336c"
          },
          {
            "range": {
              "from": "1235232",
              "to": "1235263"
            },
            "hash": "f8c0917066e72a0773dba2f71acacece0245a43ec0f7d2a97632859e8396206a"
          },
          {
            "range": {
              "from": "1235264",
              "to": "1235327"
            },
            "hash": "e4a39c334a1100868b11338d00bae2af58bb1cd786bc03d12e1db1dc95b7b50f"
          },
          {
            "range": {
              "from": "1235328",
              "to": "1235455"
            },
            "hash": "b736285a7c8033507c5e118c849d5435a1fa19aed4f2597cf394b6cd4a2e3fc5"
          },
          {
            "range": {
              "from": "1234944",
              "to": "1235199"
            },
            "hash": "23f0be146b8cd5cbc619eb00048d1aa3e4f9df1abe4b536a02586f769cf22bf1"
          },
          {
            "range": {
              "from": "1235456",
              "to": "1235967"
            },
            "hash": "4aaf34af6b459a2622c49b78d0fb319d546c7c32557999a003e034b6d3d65ed9"
          },
          {
            "range": {
              "from": "1235968",
              "to": "1236991"
            },
            "hash": "f37c05c07bc6b8d05db6a969a69f37097a4b02f94c961f1d4cb52ac1aec88ecc"
          },
          {
            "range": {
              "from": "1232896",
              "to": "1234943"
            },
            "hash": "50ca4035117430421ada1e5309f6b765c684ce1d4a9f1022124c67a99674c03e"
          },
          {
            "range": {
              "from": "1228800",
              "to": "1232895"
            },
            "hash": "6d66fd7f77c7f09cac003a732b27d915f04c98aa323d24bf93a0f252f1831d84"
          },
          {
            "range": {
              "from": "1236992",
              "to": "1245183"
            },
            "hash": "ff3ae0126e2041ef0eee373f551aa61c8f8b2f4286567e8d42495b7c58369dc8"
          },
          {
            "range": {
              "from": "1212416",
              "to": "1228799"
            },
            "hash": "a42bb93ebf2b2785c6a2bb1790894979fc67c9eb5edc3760573d00a4d980d3cf"
          },
          {
            "range": {
              "from": "1179648",
              "to": "1212415"
            },
            "hash": "ee06afd61f29a9889f6d07b59a3da638ad617baa672ba66a54d839b7bdd197a0"
          },
          {
            "range": {
              "from": "1245184",
              "to": "1310719"
            },
            "hash": "f4456417c40eb74bfe285cac99eebc035f191bcc7e4ddc6bcea4c12696463112"
          },
          {
            "range": {
              "from": "1048576",
              "to": "1179647"
            },
            "hash": "ca0062dd1badee7016ab1e700537b5d7be914ed1d152fe26ba4750210a03a42a"
          },
          {
            "range": {
              "from": "1310720",
              "to": "1572863"
            },
            "hash": "d9d332ec9f1ae763616d98ad5c99b91a371f526566bbf178a56f38531abb1d06"
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
            "hash": "9c6e27cabd25e1292480472a97c888c801f9c26c00cc0f5409bc0194d3a32eef"
          },
          {
            "range": {
              "from": "4194304",
              "to": "8388607"
            },
            "hash": "eb23dd9d58931d5864677e7e08488b7d1f9d3c31698137d7c32065c8b8078009"
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
