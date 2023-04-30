---
description: Get details of a given TXO.
---

# Get TXO

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/request.rs#L40)

| Parameter | Purpose                              | Requirements |
|-----------|--------------------------------------|--------------|
| `txo_id`  | The TXO ID for which to get details. |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}

```
{
  "method": "get_txo",
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
  "method":"get_txo",
  "result":{
    "txo":{
      "id":"34f8a29a2fdd2446694bf175e533c6bf0cd4ecac9d52cd793ef06fc011661b89",
      "value":"4764600000000",
      "token_id":"0",
      "received_block_index":"1352852",
      "spent_block_index":"1352857",
      "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
      "status":"spent",
      "target_key":"0a2058020dbb7e6047ba3ebd701f760066a8fde253932c02cfed125459aa0f45fa27",
      "public_key":"0a203c0225fab2d6df245887b7acebf22c238ffafa54842ab2663ac27833975a2212",
      "e_fog_hint":"0a54d572db8d9d8df79884eb8334c6e8ece9a7f268d1643307760206a95b9198360140845214e93c373f5401da3efb2be0357a30a8d3e590e7360ec124230ea628c4820568c302270be4f6dfcc6263a657164a590100",
      "subaddress_index":"18446744073709551614",
      "key_image":"0a201c091d59f09c7efe6e48662f810b29d4ed4308911726e001a964fbf8e251b25a",
      "confirmation":"0a20c1a3d0ced6b25dbd1d9110aeb7e99ba899129fcc5d7064fcc3a8626b245ae7e5"
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```

{% endtab %}
{% endtabs %}
