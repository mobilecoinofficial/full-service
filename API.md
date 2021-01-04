# Full Service API

The Full Service Wallet API provides http endpoints for interacting with your MobileCoin. See the [README](../README.md) for a full description of the http API.

## Full Service Data Types

The Full Service Wallet API provides several objects that correspond to the data types of the wallet

### The Account Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| account_id | string | Unique identifier for the account.
| name | string | Display name for the account.
| network_height | string (uint64) | The block height of MobileCoin's distributed ledger. The local_height is synched when it reaches the network_height.
| local_height | string (uint64) | The local block height downloaded from the ledger. The local database will sync up to the network_height. The account_height can only sync up to local_height.
| account_height| string (uint64) | The scanned local block height for this account. This value will never be greater than the local_height. At fully synced, it will match network_height.
| is_synced | boolean | Whether the account is synced with the network_height. Balances may not appear correct if the account is still syncing.
| available_pmob | string (uint64) | Available pico MOB for this account at the current account_height. If the account is syncing, this value may change.
| pending_pmob | string (uint64) | Pending, out-going pico MOB. The pending value will clear once the ledger processes the outgoing txos. The available_pmob will reflect the change.
| main_address | string | B58 Address Code for the account's main address. The main address is determined by the seed subaddress. It is not assigned to a single recipient, and should be consider a free-for-all address.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "account" | String representing the object's type. Objects of the same type share the same value
| next_subaddress_index | string (uint64) | This index represents the next subaddress to be assigned as an address. This is useful information in case the account is imported elsewhere.
| recovery_mode | boolean | A flag that indicates this imported account is attempting to un-orphan found TXOs. It is recommended to move all MOB to another account after recovery if the user is unsure of the assigned addresses.
| offset_count | int | The value to offset pagination requests for assigned_address list. Requests will exclude all list items up to and including this object.

#### Example Object

```json
{
  "object": "account",
  "account_id": "1916a9b3...",
  "name": "I love MobileCoin",
  "network_height": "88888888",
  "local_height": "88888888",
  "account_height": "88888888",
  "is_synced": true,
  "available_pmob": "123000000",
  "pending_pmob": "1000",
  "next_subaddress_index": "128",
  "recovery_mode": false,
  "offset_count": 27
}
```

#### API Methods Returning Account Objects

* [create_account](./README.md#create-account)
* [import_account](./README.md#import-account)
* [get_all_accounts](./README.md#get-all-accounts)
* [get_account](./README.md#get-account)
* [update_account_name](./README.md#update-account-name)

### The Wallet Status Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| network_height | string (uint64) | The block height of the MobileCoin ledger. The local_height is synched when it reaches the value.
| local_height | string (uint64) | The local block height downloaded from the ledger. The local database will sync up to the network_height. The account_height can only sync up to local_height.
| is_synced_all | boolean | Whether ALL accounts is synced with the network_height. Balances may not appear correct if the account is still syncing.
| total_available_pmob | string (uint64) | Available pico mob for ALL account at the account_height. If the account is syncing, this value may change.
| total_pending_pmob | string (uint64) | Pending out-going pico mob from ALL accounts. Pending pico mobs will clear once the ledger processes the outoing txo. The available_pmob will reflect the change.
| account_ids | list | A list of all account_ids imported into the wallet in order of import.
| account_map | hash map | A normalized hash mapping account_id to account objects.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "wallet_status" | String representing the object's type. Objects of the same type share the same value.

#### Example Object

```json
{
  "object": "wallet_status",
  "network_height": "88888888",
  "local_height": "88888888",
  "is_synced_all": false,
  "total_available_pmob": "123456789",
  "total_pending_pmob": "1000",
  "account_ids": ["1916a9b3...", "9b3ea14b..."],
  "account_map": {
    "1916a9b3...": {
      "account_height": "88888888",
      "account_id": "1916a9b3...",
      "available_pmob": "123000000",
      "is_synced": true,
      "local_height": "88888888",
      "name": "I love MobileCoin",
      "network_height": "88888888",
      "next_subaddress_index": "128",
      "object": "account",
      "pending_pmob": "1000",
      "recovery_mode": false
    },
    "9b3ea14b...": {
      "account_height": "88880000",
      "account_id": "9b3ea14b...",
      "available_pmob": "456789",
      "is_synced": false,
      "local_height": "88888888",
      "name": "Joint account with Satoshi",
      "network_height": "88888888",
      "next_subaddress_index": "57",
      "object": "account",
      "pending_pmob": "0",
      "recovery_mode": false
    }
  }
}
```

#### API Methods Returning Wallet Status Objects

* [get_wallet_status](./README.md#get-wallet-status)


### The Assigned Address Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| address_id | string | Unique identifier for the address.
| account_id | string | Unique identifier for the assigned associated account.
| public_address | string | Shareable B58 encoded string that represents this address.
| address_book_entry_id | serialized id | The id for an Address Book Entry object if associated to the address.
| comment | string | An arbitrary string attached to the object.

#### More Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "assigned_address" | String representing the object's type. Objects of the same type share the same value.
| subaddress_index | string (uint64) | The assigned subaddress index on the associated account..
| offset_count | int | The value to offset pagination requests for assigned_address list. Requests will exclude all list items up to and including this object.

#### Example Object

```json
{
  "object": "assigned_address",
  "address_id": "HpaL8g88...",
  "account_id": "1916a9b3...",
  "public_address": "HpaL8g88...",
  "address_book_entry_id": 36,
  "comment": "This is an assigned addresses that expects 1.5 MOB",
  "subaddress_index": "20",
  "offset_count": 21
}
```

#### API Methods Returning Assigned Address Objects

* [create_address](./README.md#create-assigned-subaddress)
* [get_all_addresses](./README.md#get-all-assigned-subaddresses-for-a-given-account)

### The Transaction Log Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| transaction_log_id | int | Unique identifier for the transaction log. This value is not associated to the ledger.
| direction | string | A string that identifies if this transaction log was sent or received. Valid values are "sent" or "received".
| is_sent_recovered | boolean | Flag that indicates if the sent transaction log was recovered from the ledger. This value is null for "received" transaction logs. If true, some information may not be available on the transaction log and its txos without user input. If true, the fee receipient_address_id, fee, and sent_time will be null without user input.
| account_id | string | Unique identifier for the assigned associated account. If the transaction is out-going, this account is from whence the txo came. If received, this is the receiving account.
| recipent_address_id | string | Unique identifier for the recipient associated account. Only available if direction is "sent".
| assigned_address_id | string | Unique identifier for the assigned associated account. Only available if direction is "received".
| value_pmob | string (uint64) | Value in pico MOB associated to this transaction log.
| fee_pmob | string (uint64) | Fee in pico MOB associated to this transaction log. Only on outgoing transaction logs. Only available if direction is "sent".
| block_height | string (uint64) | The scanned block height that generated this transaction log.
| status | string | String representing the transaction log status. On "sent", valid statuses are "built", "pending", "succeded", "failed".  On "received", the status is "succeded".

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "transaction_log" | String representing the object's type. Objects of the same type share the same value.
| txo_ids | list | A list of all txo_ids associated with this transaction log.
| sent_time | timestamp | Time at which sent transaction log was created. Only available if direction is "sent". This value is null if "received" or if the sent transactions were recovered from the ledger (is_sent_recovered = true).
| comment | string | An arbitrary string attached to the object.
| failure_code | int | Code representing the cause of "failed" status.
| failure_message | string | Human parsable explanation of "failed" status.
| offset_count | int | The value to offset pagination requests for transaction_log list. Requests will exclude all list items up to and including this object.

#### Example Objects

Received:

```json
{
  "object": "transaction_log",
  "transaction_log_id": "873dfb8c...",
  "direction": "received",
  "is_sent_recovered": null,
  "account_id": "1916a9b3...",
  "recipent_address_id": null,
  "assigned_address_id": "HpaL8g88...",
  "value_pmob": "8500000000000",
  "fee_pmob": null,
  "submitted_block_height": null,
  "fiunalized_block_height": "14152",
  "status": "succeeded",
  "intput_txo_ids": [],
  "output_txo_ids": ["28f2f033..."],
  "change_txo_ids": [],
  "sent_time": null,
  "comment": "This is a received tranaction log of 8.5 MOB!",
  "failure_code": null,
  "failure_message:": null,
  "offset_count": 1823
}
```

Sent - Failed:

```json
{
  "object": "transaction_log",
  "transaction_log_id": 2111,
  "direction": "sent",
  "is_sent_recovered": false,
  "account_id": "1916a9b3...",
  "recipent_address_id": "MZ1gUP8E...",
  "assigned_address_id": null,
  "value_pmob": "1288000000000",
  "fee_pmob": "10000000000",
  "submitted_block_height": "19152",
  "finalized_block_height": "19152",
  "status": "failed",
  "input_txo_ids": ["2bd44ea1..."],
  "output_txo_ids": ["3ce55d21..."],
  "change_txo_ids": ["1ac3d0f2..."],
  "sent_time": "2020-12-15 09:30:04 UTC",
  "comment": "This is an example of a failed sent tranaction log of 1.288 MOB and 0.01 MOB fee!",
  "failure_code": 3,
  "failure_message:": "Contains sent key image.",
  "offset_count": 2111
}
```

Sent - Success, Recovered:

```json
{
  "object": "transaction_log",
  "transaction_log_id": 888,
  "direction": "sent",
  "is_sent_recovered": true,
  "account_id": "1916a9b3...",
  "recipent_address_id": null,
  "assigned_address_id": null,
  "value_pmob": "8000000000000",
  "fee_pmob": null,
  "block_height": "8504",
  "status": "success",
  "txo_ids": ["fa1b94fa..."],
  "sent_time": null,
  "comment": "This is an example of recovered sent tranaction log of 8 MOB and unknown fee!",
  "failure_code": 3,
  "failure_message:": "Contains sent key image.",
  "offset_count": 888
}
```

#### API Methods Returning Transaction Log Objects

* [get_all_transactions_by_account](./README.md#get-all-transactions)
* [get_transaction](./README.md#get-transaction)

### The TXO Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| value_pmob | string (uint64) | Available pico MOB for this account at the current account_height. If the account is syncing, this value may change.
| received_block_height | string (uint64) | Block height in which the txo was received by an account.
| spent_block_height | string (uint64) | Block height in which the txo was spent by an account.
| is_spent_recovered | boolean | Flag that indicates if the spent_block_height was recovered from the ledger. This value is null if the txo is unspent. If true, some information may not be available on the txo without user input. If true, the proof will be null without user input.
| received_account_id | string | The account_id for the account which has received this TXO. This account has spend authority.
| minted_account_i | string | The account_id for the account which minted this TXO.
| account_status_map | hash map | A normalized hash mapping account_id to account objects. Keys include "type" and "status".
| | key: txo_type | With respect to this account, the TXO may be "minted" or "received".
| | key: txo_status | With respect to this account, the TXO may be "unspent", "pending", "spent", "secreted" or "orphaned". For received TXOs received as an assigned address, the lifecycle is "unspent" -> "pending" -> "spent". For outbound, minted TXOs, we cannot monitor its received lifecycle status with respect to the minting account, we note its status as "secreted". If a TXO is received at an address unassigned (likely due to a recovered account or using the account on another client), the TXO is considered "orphaned" until its address is calculated -- in this case, there are manual ways to discover the missing assigned address for orphaned TXOs or to recover an entire account.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "txo" | String representing the object's type. Objects of the same type share the same value.
| target_key | string (hex) | a cryptographic key for your txo
| public_key | string (hex) | the public key for this txo, can be used as an identifier to find the txo in the ledger
| e_fog_hint | string (hex) | the encrypted fog hint for this txo
| subaddress_index | string (uint64) | The assigned subaddress index for this TXO with respect to its received account.
| key_image (only on pending/spent) | string (hex) | a fingerprint of the txo derived from your private spend key materials, required to spend a txo
| offset_count | int | The value to offset pagination requests. Requests will exclude all list items up to and including this object.

#### Example Objects

Received and Spent TXO

```json
{
  "object": "txo",
  "txo_id": "14ad2f88...",
  "value_pmob": "8500000000000",
  "received_block_height": "14152",
  "spent_block_height": "20982",
  "is_spent_recovered": false,
  "received_account_id": "1916a9b3...",
  "minted_account_id": null,
  "account_status_map": {
    "1916a9b3...": {
      "txo_status": "spent",
      "txo_type": "received"
    }
  },
  "target_key": "6d6f6f6e...",
  "public_key": "6f20776f...",
  "e_fog_hint": "726c6421...",
  "subaddress_index": "20",
  "assigned_subaddress": "7BeDc5jpZ...",
  "key_image": "6d6f6269...",
  "proof": "23fd34a...",
  "offset_count": 284
}
```

Txo Spent from One Account to Another in the Same Wallet

```json
{
  "object": "txo",
  "txo_id": "84f3023...",
  "value_pmob": "200",
  "received_block_height": null,
  "spent_block_height": null,
  "is_spent_recovered": false,
  "received_account_id": "36fdf8...",
  "minted_account_id": "a4db032...",
  "account_status_map": {
    "36fdf8...": {
      "txo_status": "unspent",
      "txo_type": "received"
    },
    "a4db03...": {
      "txo_status": "secreted",
      "txo_type": "minted"
    }
  },
  "target_key": "0a2076...",
  "public_key": "0a20e6...",
  "e_fog_hint": "0a5472...",
  "subaddress_index": null,
  "assigned_subaddress": null,
  "key_image": null,
  "proof": "0a2044...",
  "offset_count": 501
}
```

#### API Methods Returning Transaction Log Objects

* [get_all_txos_by_account](./README.md#get-all-txos-for-a-given-account)
* [get_txo](./README.md#get-txo-details)

### The Proof Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| txo_id | string | Unique identifier for the Txo.
| proof | string | A string with a proof that can be verified to confirm that another party constructed or had knowledge of the construction of the associated Txo.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "proof" | String representing the object's type. Objects of the same type share the same value.

#### Example Object

```json
{
  "object": "proof",
  "txo_id": "873dfb8c...",
  "proof": "984eacd..."
}
```

#### API Methods Returning Proof Objects

* [get_proofs](./README.md#get-proofs)
* [verify_proof](./README.md#verify-proof)


### The Gift Code Object

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| gift_code | string | The b58 encoded contents of a gift code. Includes the entropy, the value, and a memo.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "gift_code" | String representing the object's type. Objects of the same type share the same value

#### Example Object

```
{
  "object": "account",
  "gift_code": "1916a9b3...",
  "entropy": "1916a9b3...",
  "value": "2000000000000",
  "memo": "Happy Birthday!"
}
```

#### API Methods Returning Account Objects

* [build_gift_code](./README.md#build-gift-code)
* [get_gift_code](./README.md#get-gift-code)
* [get_all_gift_codes](./README.md#get-all-gift-codes)

## Future API Objects

### The Recipient Address object

(Coming soon!)

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| address_id | string | Unique identifier for the address.
| public_address | string | Shareable B58 encoded string that represents this address.
| address_book_entry_id | serialized id | The id for an Address Book Entry object if associated to the address.
| comment | string | An arbitrary string attached to the object.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "address" | String representing the object's type. Objects of the same type share the same value.
| account_id | string | Unique identifier for the assigned associated account. Only for "assigned" addresses
| offset_count | int | The value to offset pagination requests for recipient_address list. Requests will exclude all list items up to and including this object.

#### Example Object

```json
{
  "object": "recipient_address",
  "address_id": "42Dik1AA...",
  "public_address": "42Dik1AA...",
  "address_book_entry_id": 36,
  "comment": "This is a receipient addresses",
  "offset_count": 12
}
```

### The Address Book Entry

(Coming soon!)

#### Attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| address_book_entry_id | int | Unique identifier for the address book entry. This value is not associated to the ledger.
| alias | string | An arbitrary string attached to the object. Useful as a user-level identifier.
| comment | string | An arbitrary string attached to the object.
| recipient_address_ids | list | A list of all recipient address_ids associated to this address book entry.
| assigned_address_ids | list | A list of all recipient address_ids associated to this address book entry.
| assigned_address_ids_by_account_map | hash map | A normalized hash mapping account_id to a list of assigned address_ids.

#### More attributes

| *Name* | *Type* | *Description*
| :--- | :--- | :---
| object | string, value is "address_book_entry" | String representing the object's type. Objects of the same type share the same value.
| offset_count | int | The value to offset pagination requests for address_book_entry list. Requests will exclude all list items up to and including this object.

#### Example Object

```json
{
  "object": "address_book_entry",
  "address_book_entry_id": 36,
  "alias": "Ojo de Tigre",
  "comment": "Homeboy from way back",
  "recipient_address_ids": ["42Dik1AA...", "MZ1gUP8E...", "4nZaeNa5..."],
  "assigned_address_ids": [ "HpaL8g88...", "YuG7Aa82...", "cPTw8yhs...", "6R6JwQAW..."],
  "assigned_address_ids_by_account_map": {
    "1916a9b3...": ["HpaL8g88...", "YuG7Aa82...", "cPTw8yhs..."],
    "9b3ea14b...": ["6R6JwQAW..."]
  },
  "offset_count": 36
}
```
