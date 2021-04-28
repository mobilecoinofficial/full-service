---
description: >-
  红包是一个只包括一个交易结果的一次性账户。在给红包充值后会得到一个 Base58 编码，开发者可以根据该编码构建用户界面（比如二维码）从而便于接收者使用红包中的额度。
---

# 红包码

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `gift_code` | 字符串 | 用于分享的 Base 58 编码的红包码。 |
| `entropy` | 字符串 | 一次性红包账户的密保口令。 |
| `value_pmob` | 字符串 | 红包内 MOB 的金额。 |
| `memo` | 字符串 | 红包的备注信息。 |

## Example

```text
{
  "object": "gift_code",
  "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
  "entropy": "41e1e794f8a2f7227fa8b5cd936f115b8799da712984c85f499e03bca43cba9c",
  "value_pmob": "60000000000",
  "memo": "新年快乐！",
  "account_id": "050d8d97aaf31c70d63c6aed828c11d3fb16b56b44910659b6724621047b81f9",
  "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
}
```

## 方法

### `build_gift_code`

在一个 `tx_proposal` （待确认交易）中构建一个红包，您可以给 `tx_proposal` 充值并提交到账簿。

| 参数 | 是否必须 | 用途 | 条件 |
| :--- | :--- | :--- | :--- |
| `account_id` | 是 | 进行操作的账户。 | 账户必须存在在钱包内。 |
| `value_pmob` | 是 | 发送到当前交易的 MOB 金额。 |  |
| `input_txo_ids` | 否 | 作为当前交易输入的 TXO（交易输出）。 |  交易输出 ID \(可从 `get_all_txos_for_account` 获取\). |
| `fee`  | 否 | 与当前交易一起提交的手续费。 |  默认为 `MINIMUM_FEE` = .01 MOB. |
| `tombstone_block`  | 否 |  当前交易的有效期（当区块高度超过本参数时即失效）。 |  默认为 `cur_height` + 50. |
| `max_spendable_value` | 否 | 当前交易的最大输入数额。  |  |
| `memo` | 否 | 红包的备注信息。 |  |

{% tabs %}
{% tab title="build\_gift\_code" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_gift_code",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "value_pmob": "42000000000000",
          "memo": "生日快乐！"
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
  "method": "build_gift_code",
  "result": {
    "tx_proposal": "...",
    "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `submit_gift_code`

将一个 `tx_proposal` 提交至账簿，在该 `tx_proposal` 被账簿确认并记录后将其红包码加入到 `wallet_db`。

| 参数 | 是否必须 | 用途 | 条件 |
| :--- | :--- | :--- | :--- |
| `gift_code_b58` | 是 | Base 58 编码的红包码。 | 必须为有效的红包码的 Base 58 编码。 |
| `from_account_id` | 是 | 进行操作的账户。 | 账户必须存在在钱包内。 |
| `tx_proposal` | 是 | 要提交的待确认交易。 |  通过 `build_gift_code` 创建。 |

{% tabs %}
{% tab title="submit\_gift\_code" %}


```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "submit_gift_code",
        "params": {
          "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
          "tx_proposal": '$(cat test-tx-proposal.json)',
          "from_account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
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
  "method": "submit_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "生日快乐！",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_gift_code`

从数据库中获取一个红包的密保口令，金额和备注。

| 参数 | 是否必须 | 用途 | 条件 |
| :--- | :--- | :--- | :--- |
| `gift_code_b58` | 是 | Base 58 编码的红包码。 | 必须为有效的红包码的 Base 58 编码。 |

{% tabs %}
{% tab title="get\_gift\_code" %}


```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_gift_code",
        "params": {
          "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
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
  "method": "get_gift_code",
  "result": {
    "gift_code": {
      "object": "gift_code",
      "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
      "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
      "value_pmob": "42000000000000",
      "memo": "Happy Birthday!",
      "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
      "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `get_all_gift_codes`

获取数据库中现有的全部红包码。

{% tabs %}
{% tab title="get\_all\_gift\_codes" %}


```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_gift_codes",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}


```text
{
  "method": "get_all_gift_codes",
  "result": {
    "gift_codes": [
      {
        "object": "gift_code",
        "gift_code_b58": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe",
        "entropy": "487d6f7c3e44977c32ccf3aa74fdbe02aebf4a2845efcf994ab5f2e8072a19e3",
        "value_pmob": "80000000000",
        "memo": "新年快乐！",
        "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
        "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
      },
      {
        "object": "gift_code",
        "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY",
        "entropy": "14aa16d9d4000628c82826d9c43bbc17414f8677e74882bf21e44db75d4c2b87",
        "value_pmob": "20000000000",
        "memo": "生日快乐！",
        "account_id": "dba3d3b99fe9ce6bc666490b8176be91ace0f4166853b0327ea39928640ea840",
        "txo_id": "ab917ed9e69fa97bd9422452b1a2f615c2405301b220f7a81eb091f75eba3f54"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `check_gift_code_status`

查看一个红包码的状态，可能为 pending（等待中），available（未收）或 claimed（已收）。

| 参数 | 是否必须 | 用途 | 条件 |
| :--- | :--- | :--- | :--- |
| `gift_code_b58` | 是 | Base 58 编码的红包码。 | 必须为有效的红包码的 Base 58 编码。 |

{% tabs %}
{% tab title="check\_gift\_code\_status" %}


```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "check_gift_code_status",
        "params": {
          "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY"
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
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeAvailable",
    "gift_code_value": 100000000,
    "gift_code_memo": "生日快乐！"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}

{
  "method": "check_gift_code_status",
  "result": {
    "gift_code_status": "GiftCodeSubmittedPending",
    "gift_code_value": null
    "gift_code_memo": "",
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

#### 输出

* `GiftCodeSubmittedPending` - 红包的交易输出还没有被同步到账簿中。
* `GiftCodeAvailable` - 红包码已经生效，而且还未被收取。
* `GiftCodeClaimed` - 红包码已经被收取了。

### `claim_gift_code`

通过红包码把红包收取到钱包内的一个账户里。

| 参数 | 是否必须 | 用途 | 条件 |
| :--- | :--- | :--- | :--- |
| `gift_code_b58` | 是 | Base 58 编码的红包码。 | 必须为有效的红包码的 Base 58 编码。 |
| `account_id` | 是 | 进行操作的账户。 | 账户必须存在在钱包内。 |

{% tabs %}
{% tab title="claim\_gift\_code" %}


```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "claim_gift_code",
        "params": {
          "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
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
  "method": "claim_gift_code",
  "result": {
    "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

### `remove_gift_code`

从数据库里删除一个红包码。

| 参数 | 是否必须 | 用途 | 条件 |
| :--- | :--- | :--- | :--- |
| `gift_code_b58` | 是 | Base 58 编码的红包码。 | 必须为有效的红包码的 Base 58 编码。 |

{% tabs %}
{% tab title="remove\_gift\_code" %}


```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "remove_gift_code",
        "params": {
          "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
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
  "method": "remove_gift_code",
  "result": {
    "removed": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

