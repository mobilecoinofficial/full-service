---
description: Get the Tx Out Membership Proof for a selection of Tx Outs
---

# Get TXO Membership Proofs

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                   | Requirements                 |
|----------------|-------------------------------------------|------------------------------|
| `outputs`      | The TXOs to get the membership proofs for | TXO must exist in the ledger |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L41)

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
  "method":"get_txo_membership_proofs",
  "result":{
    "outputs":[
      {
        "masked_amount":{
          "commitment":"c29cbaee8f6e1e824bf3e4a010a4a4479b61432082c890fc7481ddecff5e4d3d",
          "masked_value":"1242678427782368707",
          "masked_token_id":"4aee541399075d50",
          "version":null
        },
        "target_key":"58020dbb7e6047ba3ebd701f760066a8fde253932c02cfed125459aa0f45fa27",
        "public_key":"3c0225fab2d6df245887b7acebf22c238ffafa54842ab2663ac27833975a2212",
        "e_fog_hint":"d572db8d9d8df79884eb8334c6e8ece9a7f268d1643307760206a95b9198360140845214e93c373f5401da3efb2be0357a30a8d3e590e7360ec124230ea628c4820568c302270be4f6dfcc6263a657164a590100",
        "e_memo":"e236aa212d0f726f44d3f257934ff59dbf0ff79a3a37e51efb4fe740ee547a8e2948bf0fe2620c1c573ccb4b176c86af178f71eaa2bc88308e6ec82bfc4d519f9a88"
      }
    ],
    "membership_proofs":[
      {
        "index":"4061754",
        "highest_index":"4061885",
        "elements":[
          {
            "range":{
              "from":"4061754",
              "to":"4061754"
            },
            "hash":"9580ebd0aae878a5f0c7c275c89563b4fbf4bbb6ebc9f52eb797a7cc365f8d63"
          },
          {
            "range":{
              "from":"4061755",
              "to":"4061755"
            },
            "hash":"3efd20e1369706f1050d274fc41f6d07ce1eafb0448865f332dcfd1acd57fea0"
          },
          {
            "range":{
              "from":"4061752",
              "to":"4061753"
            },
            "hash":"f80f53aa151b6483da0bc54e64529d5759063d09d9d7290711dd6f74b4c441b7"
          },
          {
            "range":{
              "from":"4061756",
              "to":"4061759"
            },
            "hash":"eb0739a0bf274b0fe076230ce65971bccfeba43df60c32c64916f3c820722d22"
          },
          {
            "range":{
              "from":"4061744",
              "to":"4061751"
            },
            "hash":"6bcdeb6165e08bbba2260760343fe9106b1b050f079f92f4833568e03f8d52e3"
          },
          {
            "range":{
              "from":"4061728",
              "to":"4061743"
            },
            "hash":"be41497d280906e1ec9eb3f066e4899de47c15b2c9353d0fec522392a0798b4d"
          },
          {
            "range":{
              "from":"4061696",
              "to":"4061727"
            },
            "hash":"53ea20108e4200dbbe74a1280b416edcaafd5332c1dbbe9625c8d3af0383d843"
          },
          {
            "range":{
              "from":"4061760",
              "to":"4061823"
            },
            "hash":"3906dd527c1d13e15a13b22d14126bf1a6d10a4416a01445771ad6734b2aa521"
          },
          {
            "range":{
              "from":"4061824",
              "to":"4061951"
            },
            "hash":"b30e9b17e15423a9e0ee0f594b84114ca396857089e1462342ce4788f8b5f31f"
          },
          {
            "range":{
              "from":"4061952",
              "to":"4062207"
            },
            "hash":"ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range":{
              "from":"4061184",
              "to":"4061695"
            },
            "hash":"c3dd76b5d70b50d933919fc4c2719ed53e133d699d34777e0c31800468c22471"
          },
          {
            "range":{
              "from":"4062208",
              "to":"4063231"
            },
            "hash":"ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range":{
              "from":"4059136",
              "to":"4061183"
            },
            "hash":"a7fba61baf9e6c74a2df7e5622bfa2bdea50c96791d35cc9b9bb749370698a17"
          },
          {
            "range":{
              "from":"4055040",
              "to":"4059135"
            },
            "hash":"a4d60358e9f569993f9ce9eba2fa817c236051b1ff79250c9dd4f8b7be254a00"
          },
          {
            "range":{
              "from":"4046848",
              "to":"4055039"
            },
            "hash":"12034c89aa3483c60731cbe3eb512fa942496d8c9584bf4074d228d8759dd718"
          },
          {
            "range":{
              "from":"4030464",
              "to":"4046847"
            },
            "hash":"6fc9e8c2c26fdffad7be49f7b0644064cff011a148043a39c224577e95a019c5"
          },
          {
            "range":{
              "from":"3997696",
              "to":"4030463"
            },
            "hash":"de4c330c949c1f8c5c521cb589f755a6e8a1638672e858aa0cea1b5582406e92"
          },
          {
            "range":{
              "from":"3932160",
              "to":"3997695"
            },
            "hash":"57bcdc77e2ed2d595e31c1099e09e15c956de42d2f2be3f99a4bca593d594038"
          },
          {
            "range":{
              "from":"4063232",
              "to":"4194303"
            },
            "hash":"ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range":{
              "from":"3670016",
              "to":"3932159"
            },
            "hash":"3cfff30c9faf47c886f9fed5f8650690f8fc4f059b63175984bf6765c1355414"
          },
          {
            "range":{
              "from":"3145728",
              "to":"3670015"
            },
            "hash":"7ab34a14306b1f80a1e2587f17586462e75f1f2be2d9afa0c1caa41b6d13682f"
          },
          {
            "range":{
              "from":"2097152",
              "to":"3145727"
            },
            "hash":"8a4cd94c536573c79e6c451d101067ce98310e0a82fe04a2a2d4572529062614"
          },
          {
            "range":{
              "from":"0",
              "to":"2097151"
            },
            "hash":"f16256b5f5e635de8e230ad587df3f3d578bc5ba515c77e54d9bedee36e4435b"
          }
        ]
      }
    ]
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}
{% endtabs %}
