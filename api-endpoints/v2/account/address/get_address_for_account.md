---
description: Get an assigned address by index for an account.
---

# Get Address For Account

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements                                 |
| -------------- | -------------------------------------------- | -------------------------------------------- |
| `account_id`   | The account on which to perform this action. | The account must exist in the wallet.        |
| `index`        | The subaddress index to lookup               | The address must have already been assigned. |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "get_address_for_account",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "index": 1
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "get_address_for_account",
  "result": {
    "address": {
      "public_address_b58": "52bnq1k91NsFPuwSNH4ujFw94yrTahJ3jDmC8T4aY13iNswnDpzCC48qH5k1Y8o262WA9ph2v1rmyShMC9c7fVwwGsQXT6XLkBphewdZ8pc",
      "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
      "metadata": "Legacy Change",
      "subaddress_index": "1"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
