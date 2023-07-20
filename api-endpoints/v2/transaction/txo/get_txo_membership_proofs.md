---
description: Get the Tx Out Membership Proof for a selection of Tx Outs
---

# Get TXO Membership Proofs

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                   | Requirements                 |
| -------------- | ----------------------------------------- | ---------------------------- |
| `outputs`      | The TXOs to get the membership proofs for | TXO must exist in the ledger |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

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
  "method": "get_txo_membership_proofs",
  "result": {
    "outputs": [
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
    ],
    "membership_proofs": [
      {
        "index": "5367170",
        "highest_index": "5367257",
        "elements": [
          {
            "range": {
              "from": "5367170",
              "to": "5367170"
            },
            "hash": "453da08e60e4e0985665d17c07c6a5bdf39912cfb87fe92e3278bca821ca1e70"
          },
          {
            "range": {
              "from": "5367171",
              "to": "5367171"
            },
            "hash": "06f3b74bebb339589abaeea49c0ddea6a7858293ff4c5dee503ddb583303054a"
          },
          {
            "range": {
              "from": "5367168",
              "to": "5367169"
            },
            "hash": "719654cafaf9076a1e7ea16f6664ce44377ee15b9a587ef6877c1016ee7c4d17"
          },
          {
            "range": {
              "from": "5367172",
              "to": "5367175"
            },
            "hash": "f383827a0fb9c086ca3581dd2be4e72c2d3b97e57fc26efc8e27fffb86da95a8"
          },
          {
            "range": {
              "from": "5367176",
              "to": "5367183"
            },
            "hash": "ea069a668579c8b4e55c453ebe69447a7fd3ebf344339f6d99ad93072b951307"
          },
          {
            "range": {
              "from": "5367184",
              "to": "5367199"
            },
            "hash": "e72724396282ddc0794b8f8f3ad9382675ba50836598f89761d3dd32b57ebe11"
          },
          {
            "range": {
              "from": "5367200",
              "to": "5367231"
            },
            "hash": "1aac72f9374b5f68fd70269df3017358200579e1d2b1c5268dffc41e39f33c12"
          },
          {
            "range": {
              "from": "5367232",
              "to": "5367295"
            },
            "hash": "d40a086473a7e7bfdbdc14bbe71df0978c1790d1307b2fc04c908bac90fed366"
          },
          {
            "range": {
              "from": "5367040",
              "to": "5367167"
            },
            "hash": "39eaad9dc0d36dc04b8080156d3dde4ffae67e522d46ee1837462f1645d46e5e"
          },
          {
            "range": {
              "from": "5366784",
              "to": "5367039"
            },
            "hash": "454de105fb908a627034dc6c292f08ccbfc6aafc373995e210e15130de1886c2"
          },
          {
            "range": {
              "from": "5367296",
              "to": "5367807"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "5365760",
              "to": "5366783"
            },
            "hash": "65c1db6db311e2d758b9d0acdb76c40341a1e9c4c0f8dc2d06da277d3107d8fa"
          },
          {
            "range": {
              "from": "5367808",
              "to": "5369855"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "5369856",
              "to": "5373951"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "5357568",
              "to": "5365759"
            },
            "hash": "e47090d9a03d2c652248c088fceb23382013a05d9abb462773f3b608724449db"
          },
          {
            "range": {
              "from": "5341184",
              "to": "5357567"
            },
            "hash": "7f3777cf110065701885a4e94f25b7c2ce45410aec2194d52b8c3efce7aeb2e3"
          },
          {
            "range": {
              "from": "5308416",
              "to": "5341183"
            },
            "hash": "75859635c16a276891d94be8150483ec998c57c0c0acf4be2c96b6584257c8c3"
          },
          {
            "range": {
              "from": "5242880",
              "to": "5308415"
            },
            "hash": "1df926505475ea25dd83c58dff977f9b7d3681a8db660e82e693d2afe86a0772"
          },
          {
            "range": {
              "from": "5373952",
              "to": "5505023"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "5505024",
              "to": "5767167"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "5767168",
              "to": "6291455"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "4194304",
              "to": "5242879"
            },
            "hash": "6f300f82a0003395e4e6b40875bad1b624e1176f17738242c06999de5c36ee63"
          },
          {
            "range": {
              "from": "6291456",
              "to": "8388607"
            },
            "hash": "ffdaaf4305e365c4c30ca1e5fbf4f5e62b081441ee94eb2d0980470b5e705968"
          },
          {
            "range": {
              "from": "0",
              "to": "4194303"
            },
            "hash": "e5284a509264a61a2c65e2a060d5aa6745c69c8b62bc9f05cd878dc9f5a46b89"
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
