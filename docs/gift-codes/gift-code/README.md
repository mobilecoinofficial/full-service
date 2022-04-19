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

