---
description: >-
  Verify whether an address is correctly b58-encoded and return the address_hash
  of the provided address.
---

# Verify Address

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L40)

| Required Param | Purpose                                      | Requirements |
| -------------- | -------------------------------------------- | ------------ |
| `address`      | The address on which to perform this action. |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

`Result` attributes:

| Name           | Type                           | Description                                                                                                     |
| -------------- | ------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `verified`     | boolean                        | `true` if supplied address is a valid B58-encoded public address, otherwise `false`                             |
| `address_hash` | 16 bytes as hex-encoded string | hash of the provided public address, as used by [memos](../../transaction/txo/memo/) to identify counterparties |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "verify_address",
  "params": {
    "address": "8VWJpZDdmLT8sETcZfHdVojWdFmoo54yVEk7nmae7ixiFfxjZyVFLFj9moCiJBzkeg6Vd5BPXbbwrDvoZuxWZWsyU3G3rEvQdqZBmEbfh7x",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "verify_address",
  "result": {
    "verified": true,
    "address_hash": "52383c63bf75ca49771e8f6e4c2df0ca"
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}
