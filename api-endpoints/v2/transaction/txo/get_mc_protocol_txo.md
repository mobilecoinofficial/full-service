---
description: Get the MobileCoin transaction TXO
---

# Get MobileCoin Protocol TXO

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose            | Requirements                  |
| -------------- | ------------------ | ----------------------------- |
| `txo_id`       | The id of the TXO. | Must be a valid id for a TXO. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_mc_protocol_txo",
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
  "method": "get_mc_protocol_txo",
  "result": {
    "txo": {
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
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
