---
description: >-
  An account in the wallet. An account is associated with one AccountKey,
  containing a View keypair and a Spend keypair.
---

# Account

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "account" | String representing the object's type. Objects of the same type share the same value. |
| `account_id` | string | The unique identifier for the account. |
| `name` | string | The display name for the account. |
| `main_address` | string | The b58 address code for the account's main address. The main address is determined by the seed subaddress. It is not assigned to a single recipient and should be considered a free-for-all address. |
| `next_subaddress_index` | string \(uint64\) | This index represents the next subaddress to be assigned as an address. This is useful information in case the account is imported elsewhere. |
| `recovery_mode` | boolean | A flag that indicates this imported account is attempting to un-orphan found TXOs. It is recommended to move all MOB to another account after recovery if the user is unsure of the assigned addresses. |

## Example

```text
{
  "object": "account",
  "account_id": "1916a9b3...",
  "name": "I love MobileCoin",
  "main_address": "4bgkVAH...",
  "next_subaddress_index": "3",
  "first_block_index": "3500",
  "recovery_mode": false
}
```

## Methods

### `create_account`

Create a new account in the wallet.

{% tabs %}
{% tab title="create\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "create_account",
        "params": {
          "name": "Alice"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "create_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "name": "Alice",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
  }
}
```
{% endtab %}
{% endtabs %}

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |

### `import_account`

Import an existing account from the secret entropy.

{% tabs %}
{% tab title="import\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "import_account",
        "params": {
          "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
          "key_derivation_version": "2",
          "name": "Bob"
          "next_subaddress_index": 2,
          "first_block_index": "3500"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
   -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `mnemonic` | The secret mnemonic to recover the account. | The mnemonic must be 24 words. |
| `key_derivation_version` | The version number of the key derivation used to derive an account key from this mnemonic. The current version is 2. |  |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |
| `next_subaddress_index` | The next known unused subaddress index for the account. |  |
| `first_block_index` | The block from which to start scanning the ledger. |  |

### `import_account_from_legacy_root_entropy` \(deprecated\)

Import an existing account from the secret entropy.

{% tabs %}
{% tab title="import\_legacy\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "import_account_from_legacy_root_entropy",
        "params": {
          "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
          "name": "Bob"
          "next_subaddress_index": 2,
          "first_block_index": "3500",
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
   -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "import_account",
  "result": {
    "account": {
      "object": "account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Bob",
      "main_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `entropy` | The secret root entropy. | 32 bytes of randomness, hex-encoded. |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `name` | A label for this account. | A label can have duplicates, but it is not recommended. |
| `next_subaddress_index` | The next known unused subaddress index for the account. |  |
| `first_block_index` | The block from which to start scanning the ledger. |  |

{% hint style="warning" %}
`If you attempt to import an account already in the wallet, you will see the following error message:`

```text
{"error": "Database(Diesel(DatabaseError(UniqueViolation, "UNIQUE constraint failed: accounts.account_id_hex")))"}
```
{% endhint %}

### `get_account`

Get the details of a given account.

{% tabs %}
{% tab title="get\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_account",
        "params": {
          "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'  | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_account",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "key_derivation_version:": "2",
      "name": "Alice",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

{% hint style="warning" %}
If the account is not in the database, you will receive the following error message:

```text
{
  "error": "Database(AccountNotFound(\"a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10\"))",
  "details": "Error interacting with the database: Account Not Found: a4db032dcedc14e39608fe6f26deadf57e306e8c03823b52065724fb4d274c10"
}
```
{% endhint %}

### `get_all_accounts`

Get the details of all accounts in a given wallet.

{% tabs %}
{% tab title="get\_all\_accounts" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_all_accounts",
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
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

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

### `get_account_status`

Get the current status of a given account. The account status includes both the account object and the balance object.

{% tabs %}
{% tab title="get\_account\_status" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "get_account_status",
        "params": {
           "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "get_account_status",
  "result": {
    "account": {
      "account_id": "b0be5377a2f45b1573586ed530b2901a559d9952ea8a02f8c2dbb033a935ac17",
      "main_address": "7JvajhkAZYGmrpCY7ZpEiXRK5yW1ooTV7EWfDNu3Eyt572mH1wNb37BWiU6JqRUvgopPqSVZRexhXXpjF3wqLQR7HaJrcdbHmULujgFmzav",
      "name": "Brady",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    },
    "balance": {
      "account_block_index": "152918",
      "is_synced": true,
      "local_block_index": "152918",
      "network_block_index": "152918",
      "object": "balance",
      "orphaned_pmob": "0",
      "pending_pmob": "2040016523222112112",
      "secreted_pmob": "204273415999956272",
      "spent_pmob": "0",
      "unspent_pmob": "51080511222211091"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `update_account_name`

Rename an account.

{% tabs %}
{% tab title="update\_account\_name" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "update_account_name",
        "params": {
          "acount_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
          "name": "Carol"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json'  | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "update_account_name",
  "result": {
    "account": {
      "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
      "main_address": "4bgkVAH1hs55dwLTGVpZER8ZayhqXbYqfuyisoRrmQPXoWcYQ3SQRTjsAytCiAgk21CRrVNysVw5qwzweURzDK9HL3rGXFmAAahb364kYe3",
      "name": "Carol",
      "next_subaddress_index": "2",
      "first_block_index": "3500",
      "object": "account",
      "recovery_mode": false
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |
| `name` | The new name for this account. |  |

### `remove_account`

Remove an account from a given wallet.

{% tabs %}
{% tab title="remove\_account" %}
```text
curl -s localhost:9090/wallet \
  -d '{
        "method": "remove_account",
        "params": {
          "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```
{% endtab %}

{% tab title="return" %}
```text
{
  "method": "remove_account",
  "result": {
    "removed": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action. | The account must exist in the wallet. |

