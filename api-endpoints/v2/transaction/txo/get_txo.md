---
description: Get details of a given TXO.
---

# Get TXO

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Parameter | Purpose                              | Requirements |
| --------- | ------------------------------------ | ------------ |
| `txo_id`  | The TXO ID for which to get details. |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Examples

Below are two examples of get\_txo. The first request provides the details of an example txo that was received from a counterparty, and the second shows the details of change txo.

{% tabs %}
{% tab title="Counterparty TXO Request" %}
```
{
  "method": "get_txo",
  "params": {
    "txo_id": "92766ac6dccbf93227166f777986bbe7c0ef6651204db99b8c26f3ee8145a9ba"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Counterpart TXO  Response" %}
```
{
  "method": "get_txo",
  "result": {
    "txo": {
      "id": "92766ac6dccbf93227166f777986bbe7c0ef6651204db99b8c26f3ee8145a9ba",
      "value": "1000000000",
      "token_id": "0",
      "received_block_index": "2397012",
      "spent_block_index": "2411307",
      "account_id": "aa2bc2d42fe51e19f28bfa6a52579d60baea78305097705f3f6fda1394fe62f2",
      "status": "spent",
      "target_key": "0a202687e8320ffddcf986b4a7c837b1fb87703d0683de8adf24b62e0bf04829fb22",
      "public_key": "0a20faec0d08b57efc721d6ae89e688d08d4ba7e6b847a98a60bb9a1c51d8b567a79",
      "e_fog_hint": "0a54729ebbbd52582d4806d22d886b052657807ea4cf7627c321aeccaad49b5e3b3ba1404ca348e408587587c1e2d92c5593430b891ef310fada3fd1273ecebd7c24816601d6131f7591a3a5e14c10325686f71f0100",
      "subaddress_index": "0",
      "key_image": "0a20326750e080ac00fad2351c1ea1e65c89f8b1e8bfe4b6606f5b8cdec2e92e0868",
      "confirmation": "0a20d4bc0ba27d4ee5ce57931446b8ee3322540c4623ae6dcad2984c3efd46fe1da2",
      "shared_secret": "0a20bcdd043dccb6f49d27a560cd1ffa1e0e18e85130dc092cd60f4499d7b8ec0c1c",
      "memo": {
        "AuthenticatedSender": {
          "sender_address_hash": "bed8d8c6849285431fd4bebe4b3e22c5",
          "payment_request_id": null,
          "payment_intent_id": null
        }
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}

```
{% endtab %}

{% tab title="Change TXO Request Body" %}
```
{
  "method": "get_txo",
  "params": {
    "txo_id": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Change TXO Response" %}
```
{
  "method": "get_txo",
  "result": {
    "txo": {
      "id": "85b708102903c1169a61ddf665d62cbd0a920a0f7dd760b9083cb567769be1fb",
      "value": "470400000000",
      "token_id": "0",
      "received_block_index": "1769533",
      "spent_block_index": "1769541",
      "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "status": "spent",
      "target_key": "0a20bcaa42886171e60c50f0a4527663507a890fbecb5016f6d9042ce6be1cd7fb52",
      "public_key": "0a20cecc879afd79153210ff79b58947416a883d4f68253d415533c0e8898e09f045",
      "e_fog_hint": "0a54643db209825ced0df98a277c989b9d1876ac4009397137af1fabd3856c7c97dd629be47752cd532aa1f4bb1412d4dac9a76d50e67b4b99da017dc3a40caa99b4933ef6b4b51c56a338fc8648244eba5a22d90100",
      "subaddress_index": "18446744073709551614",
      "key_image": "0a20fafbf66b4da787c3a7d0c6a12d67620efcb47c3299ab4382627e468c718d4d1e",
      "confirmation": "0a207e8073157c3c938cc06c10c17094ba6940ec6ea15985df4760e43aaddd9bdccb",
      "shared_secret": "0a20d662b5dc2d2ada8b72cecde0a84d822aad7018e37c3ac58445740064fa26ba78",
      "memo": {
          "Destination": {
              "recipient_address_hash": "8a515f44149956609f75b27d214daed6",
              "num_recipients": "1",
              "fee": "400000000",
              "total_outlay": "1400000000",
              "payment_request_id": null,
              "payment_intent_id": null
          }
      }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
