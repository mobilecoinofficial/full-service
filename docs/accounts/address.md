---
description: >-
  An Address is a public address created from the Account Key. An Address
  contains a public View key and a public Spend key, as well as optional Fog
  materials, if the account is enabled for mobile.
---

# Address

Addresses in the Full-service Wallet are useful to help distinguish incoming transactions from each other. Due to MobileCoin's privacy properties, without using "subaddresses," the wallet would be unable to disambiguate which transactions were from which sender. By creating a new address for each contact, and sharing the address with only that contact, you can be certain that when you receive funds at that address, it is from the assigned contact.

The way this works under the hood is by using the "subaddress index" to perform a cryptographic operation to generate a new subaddress.

Important: If you receive funds at a subaddress that has not yet been assigned, you will not be able to spend the funds until you assign the address. We call those funds "orphaned" until they have been "recovered" by assigning the subaddress in the wallet to which they were sent.

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | String, value is "address" | String representing the object's type. Objects of the same type share the same value. |
| `public_address` | String | A shareable B58-encoded string representing the address. |
| `account_id` | String | A unique identifier for the assigned associated account. |
| `metadata` | String | An arbitrary string attached to the object. |
| `subaddress_index` | String \(uint64\) | The assigned subaddress index on the associated account. |
| `offset_count` | Integer | The value to offset pagination requests for assigned\_address list. Requests will exclude all list items up to and including this object. |

## Example

```text
{
  "object": "address",
  "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
  "metadata": "",
  "subaddress_index": "2",
  "offset_count": "7"
}
```

## Methods

### `assign_address_for_account`

Assign an address to a given account.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| ​`metadata` | The metadata for this address. | String; can contain stringified JSON. |

{% tabs %}
{% tab title="Request Body" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "assign_address_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
          "metadata": "For transactions from Carol"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "assign_address_for_account",
  "result": {
    "address": {
      "object": "address",
      "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "metadata": "",
      "subaddress_index": "2",
      "offset_count": "7"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `get_all_addresses_for_account`

Get all assigned addresses for a given account.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_addresses_for_account",
        "params": {
          "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_all_addresses_for_account",
  "result": {
    "public_addresses": [
      "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG",
      "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z"
    ],
    "address_map": {
      "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3": {
        "object": "address",
        "public_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Main",
        "subaddress_index": "0",
        "offset_count": "5"
      },
      "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG": {
        "object": "address",
        "public_address": "6prEWE8yEmHAznkZ3QUtHRmVf7q8DS6XpkjzecYCGMj7hVh8fivmCcujamLtugsvvmWE9P2WgTb2o7xGHw8FhiBr1hSrku1u9KKfRJFMenG",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "Change",
        "subaddress_index": "1",
        "offset_count": "6"
      },
      "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z": {
        "object": "address",
        "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
        "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
        "metadata": "",
        "subaddress_index": "2",
        "offset_count": "7"
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

### `verify_address`

Verify whether an address is correctly b58-encoded.

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |

{% tabs %}
{% tab title="Request Body" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "verify_address",
        "params": {
          "address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "verify_address",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

