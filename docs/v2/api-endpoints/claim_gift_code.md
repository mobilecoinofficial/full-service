---
description: Claim a gift code to an account in the wallet.
---

# Claim Gift Code

## Parameters

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid b58-encoded gift code. |
| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The public address of the account. | |

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "claim_gift_code",
  "params": {
    "gift_code_b58": "3DkTHXADdEUpRJ5QsrjmYh8WqFdDKkvng126zTP9YQb7LNXL8pbRidCvB7Ba3Mvek5ZZdev8EXNPrJBpGdtvfjk3hew1phmjdkf5mp35mbyvhB8UjRqoJJqDRswLrmKQL",
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "address": "996ucua1TCxcSWTgvrwit9duR2oXk25ZAF41xh5QnkkEkwmNQfiFW8XXm7Uu3kCM2aVW9krJRtCWA9ZeMCYiLnNvajfB6hbLzvYF4HJD6ak"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
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

