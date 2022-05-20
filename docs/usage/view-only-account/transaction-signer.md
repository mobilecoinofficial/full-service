---
description: How to use view only accounts with the transaction signer
---

# Transaction Signer

The transaction signer is a secondary program built with full service that provides users the ability to generate accounts and subaddresses, sync accounts, and sign transactions that were built from an online machine.

There are 5 things that the transaction signer can do

1. Create an account
2. Create a view only account import package
3. Sync a view only account
4. Generate new subaddresses
5. Sign an unsigned transaction

## Creating an Account

The first thing you will most likely want to do is to create an account and store the resulting account secrets somewhere safe!

To create an account run the transaction signer binary with the `create` command. This command takes in an optional name flag and generates 1 file as an output in the directory the binary lives in and is in the format of `mobilecoin_secret_mnemonic_{account_id[0..6]}_.json`

This file contains your account mnemonic, which can be used to spend the funds on the account! Make sure to store this in a secure location and not lose it, as it is required to perform the other functions of this program.

## Create View Only Account Import Package

The next thing that will need to be done is to create the view only account import package for the newly create account.

This command takes in the secret file generated when creating an account and outputs 1 file in the directory the binary lives in and is in the format of `mobilecoin_view_account_import_package_{account_id[0..6]}.json`

This file contains the full command that can be sent to Full Service on the online machine to import your account.

## Syncing an Account

Syncing an account is not typically necessary, but will be required in a few different situations

1. You have re-imported an already existing account that has MOB on it
2. You submit a transaction through a different service or instance of Full Service

To start a sync, you must first start from Full Service with the command [create\_view\_only\_account\_sync\_request](../../view-only-accounts/syncing/create\_view\_only\_account\_sync\_request.md)

The result in the response from this call will need to be saved to a json file and sent to the offline machine.

From the offline machine, call the Sync function of the transaction signer, which takes in the secret mnemonic file, the sync request file, and optionally a number of subaddresses to generate and attempt to decode the txos with, defaulting to 1000 subaddresses.

The result of this will be a file in the directory of the binary that contains the entire method to be called on Full Service. This will add any missing KeyImages for txos and Subaddresses required to decode them to the view only account.

## Generating Subaddresses

If you would like to use the subaddresses feature of MobileCoin accounts, then these subaddresses must be generated with the transaction signer.

Start by calling the [create\_new\_subaddresses\_request](../../view-only-accounts/subaddress/create\_new\_subaddress\_request.md) from the view only account in Full Service. This will generate a result which will need to be saved to a json file and moved to the offline machine.

From the offline machine, call the Subaddresses function of the transaction signer, which takes in the secret mnemonic file and the subaddresses request.

The result of this will be a file in the directory of the binary that contains the entire method to be called with Full Service. This will add a set of new subaddresses to the view only account.

## Sign an Unsigned Transaction

Start by calling the [build\_unsigned\_transaction](../../transactions/transaction/build\_unsigned\_transaction.md) endpoint for the view only account in Full Service. This will generate a result which will need to be saved to a json file and moved to the offline machine.

From the offline machine, call the Sign function of the transaction signer, which takes in the secret mnemonic file and the unsigned transaction request.

The result of this will be a file in the directory of the binary that contains the entire method to be called with Full Service. This will submit a transaction to the MobileCoin network and update the relevant TXOs for the view only account that was used to sign the transaction.

A few things to note:

1. If you do not include a tombstone block with the request to build the unsigned transaction, it will default to 10 blocks in the future of where the current network height is. This may or may not give you enough time to successfully sign the transaction and submit it, depending on how long it takes to transfer the signing material and how fast the network is moving. Any future tombstone block may be selected for a transaction, but consensus only accepts ones that are AT MAX 100 blocks from the current block index.
2. If you include recipients that are FOG enabled addresses, the tombstone block height will be locked at 10 blocks in the future because of requirements from FOG. There is currently no workaround for this, but the limit may be increased to allow more time to sign transactions.

