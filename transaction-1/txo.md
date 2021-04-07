---
description: >-
  A TXO is a "Transaction Output." MobileCoin is a ledger built on the "Unspent
  Transaction Output" model (UTXO).
---

# Transaction Output TXO

In order to construct a transaction, the wallet will select "Unspent Transaction Outputs" and perform a cryptographic operation to mark them as "spent" in the ledger. Then, it will mint new TXOs for the recipient.

## Attributes <a id="object_method"></a>

| _Name_ | _Type_ | _Description_ |
| :--- | :--- | :--- |
| `object` | string, value is "txo" | String representing the object's type. Objects of the same type share the same value. |
| `value_pmob` | string \(uint64\) | Available pico MOB for this account at the current `account_block_index`. If the account is syncing, this value may change. |
| `received_block_index` | string \(uint64\) | Block index in which the TXO was received by an account. |
| `spent_block_index` | string \(uint64\) | Block index in which the TXO was spent by an account. |
| `is_spent_recovered` | boolean | Flag that indicates if the `spent_block_index` was recovered from the ledger. This value is null if the TXO is unspent. If true, some information may not be available on the TXO without user input. If true, the confirmation number will be null without user input. |
| `received_account_id` | string | The `account_id` for the account which has received this TXO. This account has spend authority. |
| `minted_account_i` | string | The `account_id` for the account which minted this TXO. |
| `account_status_map` | hash map | A normalized hash mapping account\_id to account objects. Keys include "type" and "status". |
| `txo_type` | string \(enum\) | With respect to this account, the TXO may be "minted" or "received". |
| `txo_status` | string \(enum\) | With respect to this account, the TXO may be "unspent", "pending", "spent", "secreted" or "orphaned". For received TXOs received as an assigned address, the lifecycle is "unspent" -&gt; "pending" -&gt; "spent". For outbound, minted TXOs, we cannot monitor its received lifecycle status with respect to the minting account, we note its status as "secreted". If a TXO is received at an address unassigned \(likely due to a recovered account or using the account on another client\), the TXO is considered "orphaned" until its address is calculated -- in this case, there are manual ways to discover the missing assigned address for orphaned TXOs or to recover an entire account. |
| `target_key` | string \(hex\) | A cryptographic key for this TXO. |
| `public_key` | string \(hex\) | The public key for this TXO, can be used as an identifier to find the TXO in the ledger. |
| `e_fog_hint` | string \(hex\) | The encrypted fog hint for this TXO. |
| `subaddress_index` | string \(uint64\) | The assigned subaddress index for this TXO with respect to its received account. |
| `assigned_address` | string \(uint64\) | The address corresponding to the subaddress index which was assigned as an intended sender for this TXO. |
| `key_image` \(only on pending/spent\) | string \(hex\) | A fingerprint of the TXO derived from your private spend key materials, required to spend a TXO |
| `confirmation` | string \(hex\) | A confirmation that the sender of the TXO can provide to validate that they participated in the construction of this TXO. |
| `offset_count` | integer | The value to offset pagination requests. Requests will exclude all list items up to and including this object. |

## Example <a id="object_method"></a>

### Received and Spent TXO

```text
{
  "object": "txo",
  "txo_id": "14ad2f88...",
  "value_pmob": "8500000000000",
  "received_block_index": "14152",
  "spent_block_index": "20982",
  "is_spent_recovered": false,
  "received_account_id": "1916a9b3...",
  "minted_account_id": null,
  "account_status_map": {
    "1916a9b3...": {
      "txo_status": "spent",
      "txo_type": "received"
    }
  },
  "target_key": "6d6f6f6e...",
  "public_key": "6f20776f...",
  "e_fog_hint": "726c6421...",
  "subaddress_index": "20",
  "assigned_subaddress": "7BeDc5jpZ...",
  "key_image": "6d6f6269...",
  "confirmation": "23fd34a...",
  "offset_count": 284
}
```

### TXO Spent Between Accounts in the Same Wallet

```text
{
  "object": "txo",
  "txo_id": "84f3023...",
  "value_pmob": "200",
  "received_block_index": null,
  "spent_block_index": null,
  "is_spent_recovered": false,
  "received_account_id": "36fdf8...",
  "minted_account_id": "a4db032...",
  "account_status_map": {
    "36fdf8...": {
      "txo_status": "unspent",
      "txo_type": "received"
    },
    "a4db03...": {
      "txo_status": "secreted",
      "txo_type": "minted"
    }
  },
  "target_key": "0a2076...",
  "public_key": "0a20e6...",
  "e_fog_hint": "0a5472...",
  "subaddress_index": null,
  "assigned_subaddress": null,
  "key_image": null,
  "confirmation": "0a2044...",
  "offset_count": 501
}
```

## Methods

### `get_txo_object`

Get the JSON representation of the "TXO" object in the ledger.

{% tabs %}
{% tab title="get\_txo\_object" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_txo_object",
        "params": {
          "txo_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_txo_object",
  "result": {
    "txo": ...
  }
}
```
{% endtab %}
{% endtabs %}

### `get_txo`

Get details of a given TXO.

{% tabs %}
{% tab title="get\_txo" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_txo",
        "params": {
          "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_txo",
  "result": {
    "txo": {
      "object": "txo",
      "txo_id": "fff4cae55a74e5ce852b79c31576f4041d510c26e59fec178b3e45705c5b35a7",
      "value_pmob": "2960000000000",
      "received_block_index": "8094",
      "spent_block_index": "8180",
      "is_spent_recovered": false,
      "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
      "minted_account_id": null,
      "account_status_map": {
        "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
          "txo_status": "spent",
          "txo_type": "received"
        }
      },
      "target_key": "0a209eefc082a656a34fae5cec81044d1b13bd8963c411afa28aecfce4839fc9f74e",
      "public_key": "0a20f03f9684e5420d5410fe732f121626352d45e4e799d725432a0c61fa1343ac51",
      "e_fog_hint": "0a544944e7527b7f09322651b7242663edf17478fd1804aeea24838a35ad3c66d5194763642ae1c1e0cd2bbe2571a97a8c0fb49e346d2fd5262113e7333c7f012e61114bd32d335b1a8183be8e1865b0a10199b60100",
      "subaddress_index": "0",
      "assigned_subaddress": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
      "key_image": "0a205445b406012d26baebb51cbcaaaceb0d56387a67353637d07265f4e886f33419",
      "confirmation": null,
      "offset_count": 25
    }
  }
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action | Account must exist in the wallet |
| `txo_id` | The TXO ID for which to get details |  |

### `get_all_txos_for_account`

Get all TXOs for a given account.

{% tabs %}
{% tab title="get\_all\_txos\_for\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_txos_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'  | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_all_txos_for_account",
  "result": {
    "txo_ids": [
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167",
      "00408833347550b046f0996afe92313745f76e307904686e93de5bab3590e9da",
      "005b41a40be1401426f9a00965cc334e4703e4089adb8fa00616e7b25b92c6e5",
      ...
    ],
    "txo_map": {
      "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167": {
        "account_status_map": {
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "spent",
            "txo_type": "received"
          }
        },
        "assigned_subaddress": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "e_fog_hint": "0a54bf0a5f37989b379b9db3e8937387c5033428b399d44ee524c02b53ce8b7fa7ffc7181a854255cefc68704f69eedd43a891d2ed65c9f6e4c0fc645c2bc156278395221100a4fc3a1d617d04f6eca8851e846a0100",
        "is_spent_recovered": false,
        "key_image": "0a20f041e3da520a6e3328d43a920b90bf87826a1602c9249cf6591dd32328a4544e",
        "minted_account_id": null,
        "object": "txo",
        "offset_count": 262,
        "confirmation": null,
        "public_key": "0a201a592874a596aeb14cbeb1c7d3449cbd20dc8078ad7fff657e131d619145ef0a",
        "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "received_block_index": "128567",
        "spent_block_index": "128569",
        "subaddress_index": "0",
        "target_key": "0a209e1067117870549a77a47de04bd810da052abfc23d60a0c433367bfc689b7428",
        "txo_id": "001cdcc1f0a22dc0ddcdaac6020cc03d919cbc3c36923f157b4a6bf0dc980167",
        "value_pmob": "990000000000"
      },
      "84f30233774d728bb7844bed59d471fe55ee3680ab70ddc312840db0f978f3ba": {
        "account_status_map": {
          "36fdf8fbdaa35ad8e661209b8a7c7057f29bf16a1e399a34aa92c3873dfb853c": {
            "txo_status": "unspent",
            "txo_type": "received"
          },
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "secreted",
            "txo_type": "minted"
          }
        },
        "assigned_subaddress": null,
        "e_fog_hint": "0a5472b079a520696518cc7d7c3036e855cbbcf1a3e247db32ab2e62e835183077b862ef86ec4963a584650cc028eb645569f9de1392b88f8fd7fa07aa28c4e035fd5f4866f3db3d403a05d2adb5e4f2992c010b0100",
        "is_spent_recovered": false,
        "key_image": null,
        "minted_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "object": "txo",
        "offset_count": 501,
        "confirmation": "0a204488e153cce1e4bcdd4419eecb778f3d2d2b024b39aaa29532d2e47e238b2e31",
        "public_key": "0a20e6736474f73e440686736bfd045d838c2b3bc056ffc647ad6b1c990f5a46b123",
        "received_account_id": "36fdf8fbdaa35ad8e661209b8a7c7057f29bf16a1e399a34aa92c3873dfb853c",
        "received_block_index": null,
        "spent_block_index": null,
        "subaddress_index": null,
        "target_key": "0a20762d8a723aae2aa70cc11c62c91af715f957a7455b695641fe8c94210812cf1b",
        "txo_id": "84f30233774d728bb7844bed59d471fe55ee3680ab70ddc312840db0f978f3ba",
        "value_pmob": "200"
      },
      "58c2c3780792ccf9c51014c7688a71f03732b633f8c5dfa49040fa7f51328280": {
        "account_status_map": {
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "unspent",
            "txo_type": "received"
          }
        },
        "assigned_subaddress": "7BeDc5jpZu72AuNavumc8qo8CRJijtQ7QJXyPo9dpnqULaPhe6GdaDNF7cjxkTrDfTcfMgWVgDzKzbvTTwp32KQ78qpx7bUnPYxAgy92caJ",
        "e_fog_hint": "0a546f862ccf5e96a89b3ede770a70aa26ce8be704a7e5a73fff02d16ee1f694297b6c17d2e668d6181df047ae68730dfc7913b28aca66450ee1de0ca3b0bedb07664918899848f217bcbbe48be2ef40074ae5dd0100",
        "is_spent_recovered": false,
        "key_image": "0a20784ab38c4541ce23abbec6744431d6ae14101c49c6535b3e9bf3fd728db13848",
        "minted_account_id": null,
        "object": "txo",
        "offset_count": 8,
        "confirmation": null,
        "public_key": "0a20d803a979c9ec0531f106363a885dde29101fcd70209f9ed686905512dfd14d5f",
        "received_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "received_block_index": "79",
        "spent_block_index": null,
        "subaddress_index": "0",
        "target_key": "0a209abadbfcec6c81b3d184dc104e51cac4c4faa8bab4da21a3714901519810c20d",
        "txo_id": "58c2c3780792ccf9c51014c7688a71f03732b633f8c5dfa49040fa7f51328280",
        "value_pmob": "4000000000000"
      },
      "b496f4f3ec3159bf48517aa7d9cda193ef8bfcac343f81eaed0e0a55849e4726": {
        "account_status_map": {
          "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10": {
            "txo_status": "secreted",
            "txo_type": "minted"
          }
        },
        "assigned_subaddress": null,
        "e_fog_hint": "0a54338fcf8609cf80dfe017bee2339b22b626af2957ef579ae8829f3d8e7fab6c20365b6a99727fcd5e3de7784fca7e1cbb77ec35e7f2c39ea47ef6121716119ba5a67f8a6026a6a6274e7262ea8ea8280782440100",
        "is_spent_recovered": false,
        "key_image": null,
        "minted_account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10",
        "object": "txo",
        "offset_count": 498,
        "confirmation": null,
        "public_key": "0a209432c589bb4e5101c26e935b70930dfe45c78417527fb994872ebd65fcb9c116",
        "received_account_id": null,
        "received_block_index": null,
        "spent_block_index": null,
        "subaddress_index": null,
        "target_key": "0a208c75723e9b9a4af0c833bfe190c43900c3b41834cf37024f5fecfbe9919dff23",
        "txo_id": "b496f4f3ec3159bf48517aa7d9cda193ef8bfcac343f81eaed0e0a55849e4726",
        "value_pmob": "980000000000"
      }
    ]
  }
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action | Account must exist in the wallet |

{% hint style="info" %}
Note, you may wish to filter TXOs using a tool like jq. For example, to get all unspent TXOs, you can use:

```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_txos_for_account",
        "params": {
          "account_id": "a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
        },
        "jsonrpc": "2.0",
        "id": 1,
      }' \
  -X POST -H 'Content-type: application/json' \
  | jq '.result | .txo_map[] | select( . | .account_status_map[].txo_status | contains("unspent"))'
```
{% endhint %}

