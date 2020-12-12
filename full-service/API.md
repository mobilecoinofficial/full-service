# The Account object
# Attributes
# account_id
# string
# Unique identifier for the account.
# name
# string
# Display name for the account.
# network_height
# string (uint64)
# The block height of MobileCoin's distributed ledger. The local_height is synched when it reaches the network_height.
# local_height
# string (uint64)
# The local block height downloaded from the ledger. The local database will sync up to the network_height. The account_height can only sync up to local_height.
# account_height
# string (uint64)
# The scanned local block height for this account. This value will never be greater than the local_height. At fully synced, it will match network_height.
# is_synced
# boolean
# Whether the account is synced with the network_height. Balances may not appear correct if the account is still syncing.
# available_pmob
# string (uint64)
# Available pico MOB for this account at the current account_height. If the account is syncing, this value may change.
# pending_pmob
# string (uint64)
# Pending, out-going pico MOB. The pending value will clear once the ledger processes the outgoing txos. The available_pmob will reflect the change.
# main_address
# string
# B58 Address Code for the account's main address. The main address is determined by the seed subaddress. It is not assigned to a single recipient, and should be consider a free-for-all address.
# More attributes
# object
# string, value is "account"
# String representing the object's type. Objects of the same type share the same value
{
  "object": "account",
  "account_id": "1916a9b3...",
  "name": "I love MobileCoin",
  "network_height": '88888888',
  "local_height": '88888888',
  "account_height": "88888888",
  "is_synced": true,
  "available_pmob": '123000000',
  "pending_pmob": "1000",
}
# The Wallet Status object
# Attributes
# network_height
# string (uint64)
# The block height of the MobileCoin ledger. The local_height is synched when it reaches the value.
# local_height
# string (uint64)
# The local block height downloaded from the ledger. The local database will sync up to the network_height. The account_height can only sync up to local_height.
# is_synced_all
# boolean
# Whether ALL accounts is synced with the network_height. Balances may not appear correct if the account is still syncing.
# total_available_pmob
# string (uint64)
# Available pico mob for ALL account at the account_height. If the account is syncing, this value may change.
# total_pending_pmob
# string (uint64)
# Pending out-going pico mob from ALL accounts. Pending pico mobs will clear once the ledger processes the outoing txo. The available_pmob will reflect the change.
# account_ids
# list
# A list of all account_ids imported into the wallet in order of import.
# account_map
# hash map
# A normalized hash mapping account_id to account objects.
# More attributes
# object
# string, value is "wallet_status"
# String representing the object's type. Objects of the same type share the same value.
{
  "object": 'wallet_status',
  "network_height": '88888888',
  "local_height": '88888888',
  "is_synced_all": false,
  "total_available_pmob": '123456789',
  "total_pending_pmob": '1000',
  "account_ids": ["1916a9b3...", "9b3ea14b..."],
  "account_map": {
    "1916a9b3...": {
      "object": "account",
      "account_id": "1916a9b3...",
      "name": "I love MobileCoin",
      "network_height": '88888888',
      "local_height": '88888888',
      "account_height": "88888888",
      "is_synced": true,
      "available_pmob": '123000000',
      "pending_pmob": "1000",
    },
    "9b3ea14b...": {
      "object": "account",
      "account_id": "9b3ea14b...",
      "name": "Joint account with Satoshi",
      "network_height": '88888888',
      "local_height": '88888888',
      "account_height": "88880000",
      "is_synced": false,
      "available_pmob": "456789",
      "pending_pmob": "0",
    },
  },
}