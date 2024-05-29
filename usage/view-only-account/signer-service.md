# Signer Service

The signer-service is a version of the signer that runs as a JRPC HTTP Service, exposing the functionality of the signer as a collection of API endpoints

1. [Create Account](../../api-endpoints/signer-service/create\_account.md)
2. [Get Account](../../api-endpoints/signer-service/get\_account.md)
3. [Sign Transaction](../../api-endpoints/signer-service/sign\_transaction.md)
4. [Sync Txos](../../api-endpoints/signer-service/sync\_txos.md)

## Running the Signer Service

The first thing to do is to start the signer service. This can be done by getting the signer-service binary from the [release package of the latest version of full service](https://github.com/mobilecoinofficial/full-service/releases).

```bash
./signer-service
```

All available parameters can be set as Environment Variables. Parameters names are converted to `SCREAMING_SNAKE_CASE` and are prefixed with `MC_SIGNER`. See `signer-service --help` for the full list. CLI arguments take precedence over Environment Variables.

| Parameter Name | Default   |
| -------------- | --------- |
| `listen_host`  | 127.0.0.1 |
| `listen_port`  | 9092      |

The four API endpoints can be reached at `listen_host:listen_port/api`

## Creating an Account

The first thing you will most likely want to do is to create an account and store the resulting account secrets somewhere safe!

To create an account call the [create account endpoint](../../api-endpoints/signer-service/create\_account.md) and store the resulting mnemonic safely. The account info can be used to import this account into full service as a view only account using the [import view only account](../../api-endpoints/v2/account/view-only-account/import\_view\_only\_account.md)

## Getting an existing accounts info

If you already have an account you would like to use, you can call the [get account endpoint](../../api-endpoints/signer-service/get\_account.md)

## Create and Sign an Unsigned Transaction

1. (online machine) Start by calling the [build\_unsigned\_transaction](../../api-endpoints/v2/transaction/transaction/build\_unsigned\_transaction.md) endpoint for the view only account in Full Service. This will generate a result which will need to be saved and transferred to the machine that has the signer-service.
2. (offline machine) From the offline machine, call the [sign transaction endpoint](../../api-endpoints/signer-service/sign\_transaction.md) of the signer service, which takes in the mnemonic and the unsigned tx proposal.\
   \
   The result of this will be a signed transaction proposal
3. (offline --> online machine) Copy the resulting transaction proposal back to your online machine.
4. (online machine) Submit the transaction proposal with the [submit\_transaction](../../api-endpoints/v2/transaction/transaction/submit\_transaction.md) endpoint.\
   This will submit a transaction to the MobileCoin network and update the relevant TXOs for the view only account that was used to sign the transaction.build\_unsigned\_transaction

A few things to note:

1. If you do not include a tombstone block with the request to build the unsigned transaction, it will default to 10 blocks in the future of where the current network height is. This may or may not give you enough time to successfully sign the transaction and submit it, depending on how long it takes to transfer the signing material and how fast the network is moving. Any future tombstone block may be selected for a transaction, but consensus only accepts ones that are AT MAX 100 blocks from the current block index.
2. If you include recipients that are FOG enabled addresses, the tombstone block height will be locked at 10 blocks in the future because of requirements from FOG. There is currently no workaround for this, but the limit may be increased to allow more time to sign transactions.

## Syncing an Account

Syncing an account is not typically necessary, but will be required in a few different situations

1. You have re-imported an already existing account that has MOB on it
2. You submit a transaction through a different service or instance of Full Service

To start a sync, you must first start from Full Service with the command [create\_view\_only\_account\_sync\_request](../../api-endpoints/v2/account/view-only-account/create\_view\_only\_account\_sync\_request.md)

The result in the response from this call will need to be saved to a json file and sent to the offline machine.

From the offline machine, call the [sync txos endpoint](../../api-endpoints/signer-service/sync\_txos.md) of the signer-serice, which takes in the mnemonic and the list of unsynced txos.

The result of this will be a list of synced txos. This will add any missing KeyImages for txos and Subaddresses required to decode them to the view only account.

You must take the synced txos back to the online machine and call the [sync view only account endpoint](../../api-endpoints/v2/account/view-only-account/sync\_view\_only\_account.md) from full service
