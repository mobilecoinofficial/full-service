---
description: How to use view-only accounts with the offline signer.
---

# Signer

The signer is a secondary program built with full service that provides users the ability to generate accounts, sync accounts, and sign transactions that were built from view only accounts.

There are 5 things that the signer can do

1. Create a new account
2. Import an existing account
3. Create a view only account import package
4. Sync a view only account
5. Sign an unsigned transaction

## Creating an Account

The first thing you will most likely want to do is to create an account and store the resulting account secrets somewhere safe!

To create an account run the transaction signer binary with the `create` command.  You will want to run this on your offline machine so your keys are secure:

`./signer create --name="My Account"`\
`Creating account`\
`Wrote mobilecoin_secret_mnemonic_4395e9.json`\
`Wrote mobilecoin_view_account_import_package_4395e9.json`

This command creates two files, one of them containing the account mnemonic and spend key, and the second containing the view key for the online machine. The secret mnemonic contains all the information needed to backup the account, including the ability to spend funds and view transactions. Make sure to store this in a secure location and not lose it, as it is required to do anything with the account.

## Importing an Existing Account

If you already have an account you would like to use, you can run the transaction signer binary with the `import` command. This command takes in an optional name parameter, and a mnemonic parameter formatted as follows:

`./signer import "ranch invest renew amount twin summer opinion earth lock broken shed ghost idea genuine now seminar draw sorry hold hunt eager inhale party enable" --name="My Account"`

This will write the same two files as the `create` command, but with the specified mnemonic. Be sure to delete shell history logs which might contain a copy of this secret mnemonic.

## Create View Only Account Import Package

In order to view and submit transactions on the online machine, the wallet service needs to import the view key. The view key import package is normally created during account creation or import, but can be recreated with the command:

`./transaction-signer view-only-import-package mobilecoin_secret_mnemonic_f8c307.json`\
`Wrote mobilecoin_view_account_import_package_f8c307.json`

This file contains a JSON RPC command which should be `POSTed`to the Full Service endpoint at `/wallet/v2` on the online machine, in order to import the account.

## Create and Sign an Unsigned Transaction

1. (online machine) Start by calling the [build\_unsigned\_transaction](../../api-endpoints/v2/transaction/transaction/build\_unsigned\_transaction.md) endpoint for the view only account in Full Service. This will generate a result which will need to be saved to a JSON file and moved to the offline machine.
2. (online --> offline machine) Copy your saved JSON file from step 1 to your offline machine
3. (offline machine) From the offline machine, call the Sign function of the transaction signer, which takes in the secret mnemonic file and the unsigned transaction request.\
   \
   The result of this will be a transaction file in the directory of the binary that contains the entire method to be called with Full Service.
4. (offline --> online machine) Copy the resulting transaction file back to your online machine.
5. (online machine) Submit the transaction with the [submit\_transaction](../../api-endpoints/v2/transaction/transaction/submit\_transaction.md) endpoint.\
   This will submit a transaction to the MobileCoin network and update the relevant TXOs for the view only account that was used to sign the transaction.build\_unsigned\_transaction

A few things to note:

1. If you do not include a tombstone block with the request to build the unsigned transaction, it will default to 10 blocks in the future of where the current network height is. This may or may not give you enough time to successfully sign the transaction and submit it, depending on how long it takes to transfer the signing material and how fast the network is moving. Any future tombstone block may be selected for a transaction, but consensus only accepts ones that are AT MAX 100 blocks from the current block index.
2. If you include recipients that are FOG enabled addresses, the tombstone block height will be locked at 10 blocks in the future because of requirements from FOG. There is currently no workaround for this, but the limit may be increased to allow more time to sign transactions.

## Syncing an Account

Syncing an account is not typically necessary, but will be required in a few different situations

1. You have re-imported an already existing account that has MOB on it
2. You submit a transaction through a different service or instance of Full Service

To start a sync, you must first start from Full Service with the command [create\_view\_only\_account\_sync\_request](https://github.com/mobilecoinofficial/full-service/blob/main/docs/view-only-accounts/syncing/create\_view\_only\_account\_sync\_request.md)

The result in the response from this call will need to be saved to a json file and sent to the offline machine.

From the offline machine, call the Sync function of the transaction signer, which takes in the secret mnemonic file, the sync request file, and optionally a number of subaddresses to generate and attempt to decode the txos with, defaulting to 1000 subaddresses.

`./transaction-signer sync mobilecoin_secret_mnemonic_4395e9.json sync_request.json`

The result of this will be a file in the directory of the binary that contains the entire method to be called on Full Service. This will add any missing KeyImages for txos and Subaddresses required to decode them to the view only account.
