---
description: >-
  A gift code is a one-time account that contains a single TXO. Fund gift codes
  with MOB and build a user interface, such as a QR code, for consumers to claim
  and spend.
---

# Gift Code

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `gift_code` | string | The base58-encoded gift code string to share. |
| `entropy` | string | The entropy for the account in this gift code. |
| `value_pmob` | string | The amount of MOB contained in the gift code account. |
| `memo` | string | The memo associated with the gift code. |

## Example

```text
{
  "object": "gift_code",
  "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
  "entropy": "41e1e794f8a2f7227fa8b5cd936f115b8799da712984c85f499e03bca43cba9c",
  "value_pmob": "60000000000",
  "memo": "Happy New Year!",
  "account_id": "050d8d97aaf31c70d63c6aed828c11d3fb16b56b44910659b6724621047b81f9",
  "txo_id": "5806b6416cd9f5f752180988bc27af246e13d78a8d2308c48a3a85d529e6e57f"
}
```

## Methods

### `build_gift_code`

Build a gift code in a `tx_proposal` that you can fund and submit to the ledger.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |
| `value_pmob` | The amount of MOB to send in this transaction. |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `input_txo_ids` | The specific TXOs to use as inputs to this transaction. |  TXO IDs \(obtain from `get_all_txos_for_account`\). |
| `fee` | The fee amount to submit with this transaction. |  If not provided, uses `MINIMUM_FEE` = .01 MOB. |
| `tombstone_block` | The block after which this transaction expires. |  If not provided, uses `cur_height` + 50. |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction. |  |
| `memo` | Memo for whoever claims the gift code. |  |

{% tabs %}
{% tab title="build\_gift\_code" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "build_gift_code",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "value_pmob": "42000000000000",
          "memo": "Happy Birthday!"
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

Submit a `tx_proposal` to the ledger that adds the gift code to the `wallet_db` once the `tx_proposal` has been appended to the ledger.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid base58-encoded gift code. |
| `from_account_id` | The account on which to perform this action. | The account must exist in the wallet. |
| `tx_proposal` | Transaction proposal to submit. |  Created with `build_gift_code`. |

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
      "memo": "Happy Birthday!",
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

Recall a gift code's entropy, value, and memo from the database.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid base58-encoded gift code. |

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

Get all the gift codes currently in the database.

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
        "memo": "Happy New Year!",
        "account_id": "1e7a1cf00adc278fa27b1e885e5ed6c1ff793c6bc56a9255c97d9daafdfdffeb",
        "txo_id": "46725fd1dc65f170dd8d806a942c516112c080ec87b29ef1529c2014e27cc653"
      },
      {
        "object": "gift_code",
        "gift_code_b58": "2yE5NUCa3CZfv72aUazPoZN4x1rvWE2bNKvGocj8n9iGdKCc9CG72wZeGfRb3UBx2QmaoX6CZsVpYFySgQ3tfmhWpywfrf4GQq4JF1XQmCrrw8qW3C9h3qZ9tfu4fFxgY",
        "entropy": "14aa16d9d4000628c82826d9c43bbc17414f8677e74882bf21e44db75d4c2b87",
        "value_pmob": "20000000000",
        "memo": "Happy Birthday!",
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

Check the status of a gift code, which may be pending, available, or claimed.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | The base58-encoded gift code must be valid. |

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
    "gift_code_memo": "Happy Birthday!"
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

#### Outputs

* `GiftCodeSubmittedPending` - The gift code TXO has not yet appeared in the ledger.
* `GiftCodeAvailable` - The gift code TXO is available to be claimed.
* `GiftCodeClaimed` - The gift code TXO has been spent.

### `claim_gift_code`

Claim a gift code to an account in the wallet.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | The base58-encoded gift code must be valid. |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

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

Remove a gift code from the database.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | The base58-encoded gift code must be valid. |

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

