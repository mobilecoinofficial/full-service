---
description: >-
  An Address is a public address created from the Account Key. An Address
  contains a public View key and a public Spend key, as well as optional Fog
  materials, if the account is enabled for mobile.
---

# Address

An account can have u64::MAX (\~18.45 quintillion)  unique public addresses, which we call _**subaddresses,**_ generated for it, and full-service can determine which subaddress of the account was used when the funds were sent to the account.  Wallets built atop full-service use this capability to associate incoming funds with a designated purpose. One example is an exchange that provides a unique deposit address to each of its customers and thus knows which customer to credit when funds arrive at the exchange. Another example is an e-commerce site that issues unique payment addresses for its carts. When funds arrive, by matching the address that received the funds with subaddresses assigned to orders in its database, it knows which order has completed checkout.

Because the MobileCoin blockchain is end-to-end encrypted, every transaction output on the blockchain has to be checked by every account owner using their view private key in order to determine if the transaction output belongs to that account. If the above exchange or e-commerce site were to create independent accounts for each customer or cart, their site would not scale well as it would have to loop over every transaction output in every block, multiplied each of its customers view private keys, to identify transaction outputs sent to its customers. Imagine an exchange with 100,000 customers having to decode 30 transaction outputs per second using 100,000 private view keys in parallel. Expensive!&#x20;

Subaddresses solve this scaling problem because the wallet can scan the blockchain using the account's single private view key to determine which transaction outputs were sent to any subaddress of the account, and then do a table lookup for transaction outputs that match and determine to which subaddress the transaction output was sent.  The 3,000,000 decode operations per second of the above example, becomes 30 decodes per second instead, and only when there is a match is there a need to do a table lookup to determine which customer received funds, also an inexpensive operation.

Important: If the wallet database is reset and the account restored by importing its secret phrase or entropy, full-service will not be able to spend funds sent to subaddresses until the subaddresses are recovered as well by performing `assign_address_for_account` for each of the "orphaned" subaddresses.  Upon recovery, each `subaddress_index` will be assigned the same `public_address` as it orginally had.

## Attributes

| Name               | Type            | Description                                              |
| ------------------ | --------------- | -------------------------------------------------------- |
| `public_address`   | string          | A shareable B58-encoded string representing the address. |
| `account_id`       | string          | A unique identifier for the assigned associated account. |
| `metadata`         | string          | An arbitrary string attached to the object.              |
| `subaddress_index` | string (uint64) | The assigned subaddress index on the associated account. |

## Example

```
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
