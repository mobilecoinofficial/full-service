---
description: Get view only TXOs for a given view only account with offset and limit parameters
---

# Get TXOs For View Only Account

## Parameters

| Parameter | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `offset` | The pagination offset. Results start at the offset index. Optional, defaults to 0. | |
| `limit` | Limit for the number of results. Optional, defaults to 100 | |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_txos_for_view_only_account",
  "params": {
    "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
    "offset": "2",
    "limit": "8"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_txos_for_view_only_account",
  "result": {
    "txo_ids": [
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167",
      "00408833347550b046f0996afe92313745f76e307904686e93de5bab3590e9da",
      "005b41a40be1401426f9a00965cc334e4703e4089adb8fa00616e7b25b92c6e5"
    ],
    "txo_map": {
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167": {
        "object": "view_only_txo",
        "txo_id_hex": "84eab721b7eeb4dc6f6d73c0504182a06ccfb98e2d341acac2dfe22d831fae44",
        "value_pmob": "10000000000000",
        "public_key": "ef3e04533424fd181e8039ec4e2df0bc67c2f59dbbe55d660202d0fc588638d2",
        "view_only_account_id_hex": "324a0969a356a81916eecb3aa002da2bbc79154a835c9f6df61d71f67dc5f632",
        "spent": false
      }
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167": {
        "object": "view_only_txo",
        "txo_id_hex": "27eab721b7eeb4dc6f6d73c0504182a06ccfb98e2d341acac2dfe22d831fae44",
        "value_pmob": "20000000000000",
        "public_key": "222204533424fd181e8039ec4e2df0bc67c2f59dbbe55d660202d0fc588638d2",
        "view_only_account_id_hex": "324a0969a356a81916eecb3aa002da2bbc79154a835c9f6df61d71f67dc5f632",
        "spent": false
      }
      "005b41a40be1401426f9a00965cc334e4703e4089adb8fa00616e7b25b92c6e5": {
        "object": "view_only_txo",
        "txo_id_hex": "93eab721b7eeb4dc6f6d73c0504182a06ccfb98e2d341acac2dfe22d831fae44",
        "value_pmob": "30000000000000",
        "public_key": "123454533424fd181e8039ec4e2df0bc67c2f59dbbe55d660202d0fc588638d2",
        "view_only_account_id_hex": "324a0969a356a81916eecb3aa002da2bbc79154a835c9f6df61d71f67dc5f632",
        "spent": false
      }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
