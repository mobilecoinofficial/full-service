---
description: Get the JSON representation of the last recent "Block" objects in the ledger.
---

# Get Recent Blocks

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `limit` | The number of blocks to return | |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Body Request" %}
```text
{
  "method": "get_recent_blocks",
  "params": {
    "limit": 2,
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_recent_blocks",
  "result": {
    "blocks": [
      {
        "id": "257ff2b9eaffac94af49aad3752133b3005b0eb1fb12f0c3beb4cf031da8f108",
        "version": "2",
        "parent_id": "bbcd793fbf98bc82468c518c8cc754d9eed8cf48702987a33ad982fb03c15508",
        "index": "1378218",
        "cumulative_txo_count": "4137864",
        "root_element": {
          "range": {
            "from": "0",
            "to": "4194303"
          },
          "hash": "9546556dad370fbb5afb21970213551e0e10330c28d2e1fc39be252c211188ad"
        },
        "contents_hash": "d91cf3225ed252e4a134429a96dc384a2505d0d3bb773c0d55677117af4462a1"
      },
      {
        "id": "bbcd793fbf98bc82468c518c8cc754d9eed8cf48702987a33ad982fb03c15508",
        "version": "2",
        "parent_id": "f0b701f1eb070e2c0ec652ecf8b32fc858df20cf5c40ced855f315cebc7a60ca",
        "index": "1378217",
        "cumulative_txo_count": "4137861",
        "root_element": {
          "range": {
            "from": "0",
            "to": "4194303"
          },
          "hash": "2d1d266cd6659561cba07188758c920f5da8b4d192004e96270732f259cbc591"
        },
        "contents_hash": "8064cff29b04735e2f78767f687696cc4926d5ab5ecdfcb34ca2de7aca2bc2c7"
      }
    ],
    "block_contents": [
      {
        "key_images": [
          "0a202625b073d304a2d0915cb3f7efe59928f9e4e9d37d7348566e8c76308a6d3574",
          "0a205a70001e6740a68ae8dde3fe04eac289bb103bef61e0ce56897e4881a79e6b7d"
        ],
        "outputs": [
          {
            "masked_amount": {
              "commitment": "a2ce1ce0be4a0ceabca33eb29db8bcfc19b923207f85980d2f8bb03d0036f23a",
              "masked_value": "2833140401228471876",
              "masked_token_id": "df22df15382a452f",
              "version": 1
            },
            "target_key": "3670cfd20f3a375e844b29eec0cf32d94c55b2694c50f2353778f0412a5bf02e",
            "public_key": "3e2ddbdcee5b80bc5522f505f85d98cedc1f007e6de696c8cc3687b9ed68ff10",
            "e_fog_hint": "730c9e46bfa5960ca9d8619b11a6b6399c0c34ed2c42122992dd7746d0869b3434970a08d0f4c014652678b6abdb7a8d1a37925f2334c1d7e0742997ecb600a5552d66b1434529e37460ca7efc77a78573d40100",
            "e_memo": "8a1ff5f33944dc8e3f8e71d2989a2c70b218cb0338339fe264621dc374654722a92abe670d4462259f6a6a8b638ac89354fe1ae60c4f3ce7c32e77278c6695842dfa"
          },
          {
            "masked_amount": {
              "commitment": "be1c82662b09996ce08ca794e10b426698c8792a8eac336a3c8199dc765aa65c",
              "masked_value": "13849038797285909200",
              "masked_token_id": "ec4c6d6e0af1dd00",
              "version": 1
            },
            "target_key": "8acdd2efc9970abb6b78fde479305ef896a62f904149d9f0129d2b2656bee132",
            "public_key": "56a361f432dc8dc515859bd3236044586f73ae5f3a90792afaec44b2f7a36455",
            "e_fog_hint": "9d3d05cfdffd35b74c1082f97af2666a0cd0ada7094306e3d510c9048b0ef365d5daa40afc65ee6daee6a3f640d1767c79aa54666ea4cb177bcf153e34361b52f63ea524e23d0769ed9f779c58245c2a27400100",
            "e_memo": "e8624cabaa4581c5f9f612a092f02409341018c8648d6fc8455e4e49985629ae8dbef20fa6c35a0da25774cc09ef9d19673c9010e5a532a8e4902f7dc91097cd3858"
          },
          {
            "masked_amount": {
              "commitment": "4e591230d1839539abc53c92b5e9d1af5992143cf7794a4a1ca963679f690048",
              "masked_value": "8969994541795584088",
              "masked_token_id": "2904992a11ee40dd",
              "version": 1
            },
            "target_key": "2e7c5052422ebee60f278c5cc30c54517e6394bc6d23e462cb550cc2ebdd3140",
            "public_key": "c4e12b8b87448e4b5491b64941dfa5508fa2081e07e168dee9d60c0039f33c40",
            "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "e_memo": "91243098a3babe81323c90ccb6578a4fa6164fe3c13a485594dc9f1eb9bad4b6ef8d40e029b40c32821195bf79f5a2865bedf18c287a8197f9dd8bf0d16ac67202f0"
          }
        ]
      },
      {
        "key_images": [
          "0a204050f5486f5979ae946e03026bd5bb0ceff5cb8defeff047b9a9d9220610746c",
          "0a2088c2323687804d55ed683332feda3eda5ff7c8a66f1a75b0eb31cbc28fb7406f"
        ],
        "outputs": [
          {
            "masked_amount": {
              "commitment": "e0971bd281bb0cedfa336eb837e80b019df6a1d072603bb67f06d7c18cfc9a06",
              "masked_value": "15708444948380422301",
              "masked_token_id": "6c4c414a634b2f82",
              "version": 1
            },
            "target_key": "00837df1548b8a69a51251368cd5c94704db6032883cc84d3d7b02dd869ead62",
            "public_key": "4e4d5bdb0f74c1b6a780ad12a505a33e7c49c6c51367d5d79b999d6eb51fdf30",
            "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "e_memo": "bbe2a8e4f5a0e8ffc1a17dc2dcd27b44ab3dd7907ab600acc64f517dbe726461f8728652174c5af79a31a96d2fff6b754e878ce592ea8aa07a79c795ab35f39a8d9f"
          },
          {
            "masked_amount": {
              "commitment": "5049bd497cdbbec13ab38ad548c77d2e18c24adccd3a7d9fa608acc0bf916c66",
              "masked_value": "6750428392038692076",
              "masked_token_id": "dedd08ff94d39f08",
              "version": 1
            },
            "target_key": "b4a3e8d91746a23af085a1abf44f14c93b0b719ad7192a324fa12790fe86a502",
            "public_key": "6473e0f18679edcc20fdbccf2876b2ea2ce4cf6e047404be72f66ed438482e50",
            "e_fog_hint": "54e3caff452051384aa26bc73c500d26e314ccb95b546326d483fb297a86246eb5c8d89c05afa039d38aae7e3831f7e11753d37b043f2255715cc21853391ba9f64cc3407e2ee637f1d6af396c8da6df61490100",
            "e_memo": "70f5cc0c70aea608a216782412e8a9c236a5b36d3cf12f0bd5163f86789a5d0b47e6acea96d588819d43bb853240c36c385cb4d866c954311592b5c2fd23a82f5240"
          },
          {
            "masked_amount": {
              "commitment": "ac530133865b10be86927880168b504fdfd170cb0f34c1fb1747e27f4003135d",
              "masked_value": "10314885951227847898",
              "masked_token_id": "c72f6f8025f7788b",
              "version": 1
            },
            "target_key": "dcc52f2425101366563d2ce6910c30849a6ce817adb27667bfab6bab66d7143b",
            "public_key": "ec7761d4f32c99a8c65fefe06897c960afeec4351012869e6999801dbe4de401",
            "e_fog_hint": "d483539b71f6b65d54a24a697ce8931cdb12296419bcffd097c2f5e2ee2a426ce876069c9a8132b87b74acfb71c27421ccde6818d18418d3acfa773f8fc2e486d96b779392ae4460fdbd5ccd7e02b982fa570100",
            "e_memo": "7a5379bb34492e6858cbdd3166d0649ee1af5e3b29ca99b7d15326b88802a69cc713370911e82d2594dd96bf474320c97b87804449a38db919c74236b3294141f9d9"
          }
        ]
      }
    ],
    "network_status": {
      "network_block_height": "1378219",
      "local_block_height": "1378219",
      "local_num_txos": "4137864",
      "fees": {
        "0": "400000000",
        "1": "2560",
        "8192": "2560"
      },
      "block_version": "2"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

