---
description: TXO 是“交易输出（Transaction Output）”的缩写。MobileCoin 是一个基于 UTXO （未花交易输出）模型的账簿。
---

# 交易输出 TXO

在构造交易时，钱包会选择未花的 TXO 并通过一些密码学操作将它们在账簿内标记为“已花”。在此之后为收款方铸（Mint）新的 TXO。

## 属性 <a id="object_method"></a>

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "txo" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `value_pmob` | 字符串，内容为 64 位无符号整数 | 当前账户在当前 `account_block_index` 位置的可使用的 pico MOB。 在账户完成同步后本数值可能会发生变化。 |
| `received_block_index` | 字符串，内容为 64 位无符号整数 | TXO 被接收的区块索引。 |
| `spent_block_index` | 字符串，内容为 64 位无符号整数 | TXO 被使用（消费）的区块索引。 |
| `is_spent_recovered` | 布尔型 | 指示 `spent_block_index` 是否被从账簿恢复。当 TXO 未被花费时未 `null`。 If true, some information may not be available on the TXO without user input. If true, the confirmation number will be null without user input. |
| `received_account_id` | 字符串 | 接收本 TXO 的账户的 `account_id`。该账户拥有使用本 TXO 的权力。 |
| `minted_account_id` | 字符串 | 本 TXO 被铸（Mint）的账户的 `account_id`。 |
| `account_status_map` | 散列表 | 一个规范化的从 `account_id` 到账户对象的散列映射。键值包括 "type" 和 "status"。 |
| `txo_type` | 由字符串表示的枚举类型 | 对于当前账户，一个 TXO 可能为 "minted" 或 "received"。 |
| `txo_status` | 由字符串表示的枚举类型  | 对于当前账户, 一个 TXO 可能为 "unspent", "pending", "spent", "secreted" 或 "orphaned"。 通过指定地址接收到的 TXO 的生命周期为："unspent" -&gt; "pending" -&gt; "spent"。 对于花出的 TXO，我们并不能监控其接收状态，因此记为 "secreted"。如果一个 TXO 是通过未分配的地址接收的（可能是由于账户信息恢复不完整导致），在对应的子地址被重新计算之前，TXO 的状态将为 "orphaned"，在这种情况下，您可以通过手动途径找回该子地址，或是恢复整个账户。 |
| `target_key` | 字符串 \(hex\) | 本 TXO 的密码学键值。 A cryptographic key for this TXO. |
| `public_key` | 字符串 \(hex\) | 本 TXO 的公钥，可以作为本 TXO 在账簿上的标识符。|
| `e_fog_hint` | 字符串 \(hex\) | 本 TXO 的加密的雾信息（Fog hint）。  |
| `subaddress_index` | 字符串，内容为 64 位无符号整数 | 为本 TXO 指定的接收方的子地址索引。|
| `assigned_address` | 字符串，内容为 64 位无符号整数 | 为本 TXO 指定的发送方的地址。|
| `key_image` \(只会出现在发送中/已花费 TXO 中\) | 字符串 \(hex\) | 从可花（Spend）私钥计算出的 TXO 指纹，是发送 TXO 的必要信息。|
| `confirmation` | 字符串 \(hex\) | 发送者参与了 TXO 构造的证明。|
| `offset_count` | 整型 | 请求的分页偏移量。请求将只会返回当前对象之后的内容（不包括当前对象）。 |

## 示例 <a id="object_method"></a>

### 接收的并已花费的 TXO

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

### 在同一个钱包内的两个账户间花费的 TXO

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

## 方法

### `get_txo_object`

Get the JSON representation of the "TXO" object in the ledger.

| Parameter | Purpose | Requirements |
| :--- | :--- | :--- |
| `txo_id` | A TXO identifier. |  |

{% tabs %}
{% tab title="Request Body" %}
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

{% tab title="Response" %}
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

| Parameter | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
| `txo_id` | The TXO ID for which to get details. |  |

{% tabs %}
{% tab title="Request Body" %}
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

{% tab title="Response" %}
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

### `get_all_txos_for_account`

Get all TXOs for a given account.

| Parameter | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
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

{% tab title="Response" %}
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

{% hint style="info" %}
您可以使用诸如 jq 的工具来选取您关心的 TXO。比如，若要选取全部未花的 TXO，您可以使用：
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

