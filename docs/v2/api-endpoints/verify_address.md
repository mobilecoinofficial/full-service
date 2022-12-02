---
description: Verify whether an address is correctly b58-encoded.
---

# Verify Address

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "verify_address",
  "params": {
    "address": "gK4GSmZ7de6pCnsPnkqdwV47a14fR7M5JuKf3TAfqwGcsDsa3cJW5gKnu52ZDWepfEPi5T6a55gVGB6AvQKtKKtBQEYSwUpDTpSKfG9Et4QA9zLUQyEcpfCx1t79tuoe93sUezp9wXeyT9eSgQPiMmNBXGx7JfhZXqjmiXGRrDyEMMpgY5B7pWMXzD7SH7UW5AFwJkYoNSL9Ff6N4ztebFGS46H9hJ6VQuGh4wHcDbhY6sGGuJH",
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "verify_address",
  "result": {
    "details": {
        "view_public_key": "08ad42828b934e6590794f788c5a5599afa12ca8080b90afbbbf928a0a30cd6f",
        "spend_public_key": "4aa1eb7d23b7bfe4db3973277d85872eb7dcfe175c93c5c8758ecfd780ad6a10",
        "fog_report_url": "fog://fog.prod.mobilecoinww.com",
        "fog_report_id": null,
        "fog_authority_sig": "c4ab081494440128d24e6b1451323888699a5fa2bf9922c36469774ac5114f0ad6ccc48b95a5e633cb4827e53569a74159b9941871890aa5bb3c73b341b75d82"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

