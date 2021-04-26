---
description: >-
  Gift codes are onetime accounts that contain a single Txo. Fund gift codes
  with MOB and create a user interface, such as a QR code, for consumers to
  claim and spend the MOB.
---

# Gift Code

### Attributes

| Object | Type | Value | Description |
| :--- | :--- | :--- | :--- |
| gift\_code | string | base58-encoded gift code |  |
| entropy | string | The account's entropy |  |
| value\_pmob | string | Amount of MOB |  |
| memo | string | Memo |  |

### Example

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

### Methods

* Build
* Submit
* Get
* Get All
* Check Status
* Claim
* Remove

### Build

{% tabs %}
{% tab title="API" %}
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

{% tab title="Result" %}
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

{% api-method method="get" host="" path="" %}
{% api-method-summary %}
Build
{% endapi-method-summary %}

{% api-method-description %}
Creates a gift code that you can then fund and submit to the ledger.
{% endapi-method-description %}

{% api-method-spec %}
{% api-method-request %}
{% api-method-form-data-parameters %}
{% api-method-parameter name="account\_id" type="number" required=true %}
The account must exist within the wallet for you to operate on it.
{% endapi-method-parameter %}

{% api-method-parameter name="value\_pmob" type="number" required=true %}
The amount of MOB to send in this transaction.
{% endapi-method-parameter %}
{% endapi-method-form-data-parameters %}
{% endapi-method-request %}

{% api-method-response %}
{% api-method-response-example httpCode=200 %}
{% api-method-response-example-description %}

{% endapi-method-response-example-description %}

```

```
{% endapi-method-response-example %}
{% endapi-method-response %}
{% endapi-method-spec %}
{% endapi-method %}



