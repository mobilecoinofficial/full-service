---
description: Create a new account in the wallet.
---

# Create Account

Parameters

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "create_account",
  "params": {
    "name": "Alice"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "create_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Alice",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  }
}
```
{% endtab %}
{% endtabs %}

{% api-method method="post" host="method: " path="import\_account" %}
{% api-method-summary %}
Import Account
{% endapi-method-summary %}

{% api-method-description %}

{% endapi-method-description %}

{% api-method-spec %}
{% api-method-request %}
{% api-method-body-parameters %}
{% api-method-parameter name="fog\_report\_url" type="string" required=false %}

{% endapi-method-parameter %}

{% api-method-parameter name="fog\_report\_id" type="string" required=false %}

{% endapi-method-parameter %}

{% api-method-parameter name="fog\_authority\_apki" type="string" required=false %}

{% endapi-method-parameter %}

{% api-method-parameter name="next\_subaddress\_index" type="string" required=false %}

{% endapi-method-parameter %}

{% api-method-parameter name="first\_block\_index" type="string" required=false %}

{% endapi-method-parameter %}

{% api-method-parameter name="name" type="string" required=false %}

{% endapi-method-parameter %}

{% api-method-parameter name="key\_derivation\_version" type="string" required=true %}

{% endapi-method-parameter %}

{% api-method-parameter name="mnemonic" type="string" required=true %}
24 words
{% endapi-method-parameter %}
{% endapi-method-body-parameters %}
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



