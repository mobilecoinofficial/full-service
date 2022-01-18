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
| `offset` | integer | The value to offset pagination requests. Requests will exclude all list items up to and including this object. |
| `limit` | integer | The limit of returned results. |

## Example

```text
{
  "object": "address",
  "public_address": "3P4GtGkp5UVBXUzBqirgj7QFetWn4PsFPsHBXbC6A8AXw1a9CMej969jneiN1qKcwdn6e1VtD64EruGVSFQ8wHk5xuBHndpV9WUGQ78vV7Z",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
  "metadata": "",
  "subaddress_index": "2",
  "offset": "7",
  "limit": "6"
}
```

