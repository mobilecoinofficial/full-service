---
description: Check the type of the b58 code
---

# Check B58 Type

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L75)

| Required Param | Purpose           | Requirements |
| -------------- | ----------------- | ------------ |
| `b58_code`     | The code to check | `String`     |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L58)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "check_b58_type",
  "params": {
    "b58_code": "37Vz37cUTxcrB8PVMLFedtdz1dS9xcV4TGYsCfwjW1jEuGTMtQVRgptZ7xi571gaRhUxk3j9HLjoEGMD7VMQWGHX7PWJD5qcAYPA1SB96WdikREV2azdqoyJvdrgyCT5wt9e8KtmjkoVcHeB1whY6NjD9yEevJVv5GU"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "check_b58_type",
  "result": {
    "b58_type": "PaymentRequest",
    "data": {
      "public_address_b58": "52bnq1k91NsFPuwSNH4ujFw94yrTahJ3jDmC8T4aY13iNswnDpzCC48qH5k1Y8o262WA9ph2v1rmyShMC9c7fVwwGsQXT6XLkBphewdZ8pc",
      "memo": "Payment for dinner with family",
      "value": "528000000000",
      "token_id": "0"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
