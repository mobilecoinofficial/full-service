---
description: Get the details of all accounts in a given wallet.
---

# Get All Accounts

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
    "method": "get_all_accounts",
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_all_accounts",
  "result": {
    "account_ids": [
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0"
    ],
    "account_map": {
      "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52": {
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "key_derivation_version:": "1",
        "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "name": "Alice",
        "next_subaddress_index": "2",
        "first_block_index": "3500",
        "object": "account",
        "recovery_mode": false
      },
      "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0": {
        "account_id": "b6c9f6f779372ae25e93d68a79d725d71f3767d1bfd1c5fe155f948a2cc5c0a0",
        "key_derivation_version:": "2",
        "main_address": "7EqduSDpM1R5AfQejbjAqFxpuCoh6zJECtvJB9AZFwjK13dCzZgYbyfLf4TfHcE8LVPjzDdpcxYLkdMBh694mHfftJmsFZuz6xUeRtmsUdc",
        "name": "Alice",
        "next_subaddress_index": "2",
        "first_block_index": "3500",
        "object": "account",
        "recovery_mode": false
      }
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

