---
description: Get details of a given TXO.
---

# Get TXO

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Parameter | Purpose                              | Requirements |
| --------- | ------------------------------------ | ------------ |
| `txo_id`  | The TXO ID for which to get details. |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
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

{% tab title="Response" %}
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
