---
description: Get the MobileCoin transaction TXO
---

# Get MobileCoin Protocol TXO

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose            | Requirements                  |
|----------------|--------------------|-------------------------------|
| `txo_id`       | The id of the TXO. | Must be a valid id for a TXO. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}

```
{
  "method": "get_mc_protocol_txo",
  "params": {
    "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```

{% endtab %}

{% tab title="Response" %}

```
{
  "method":"get_mc_protocol_txo",
  "result":{
    "txo":{
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
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}
{% endtabs %}
