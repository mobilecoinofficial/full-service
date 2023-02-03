# Full Service Python API and CLI

## Setup

`$ poetry install`


## CLI usage

First, you should have an instance of full-service running on your local machine.
```
$ cd full-service
$ ./tools/run-fs.sh test
```
See the full-service README for more details.

Check that your CLI is correctly configured by running the status command:
```
$ mob status
Connected to MobileCoin network.
All accounts synced, 1421264 blocks.

Total balance for all accounts:
  0 MOB

Transaction Fees:
  0.0004 MOB
  0.00256 eUSD
```

Create an account.
```
$ mob create
Created a new account.
d7efc1

```

See it in the account list.
```
$ mob list

d7efc1 (synced)
  address 6eRwLkggafsMs8Mef3JPLExksDpG7BRdYtaDhLkNn1c3AkcdZegJXxsxaPGnZZjR8nuz9SmhYHPrZ3yxqfjmbxfefCK6RqXqNfD9w4T9Hb7
  0 MOB
```

Get at least 1 MOB on testnet in order to run unitests, then export your wallet so the
unittests can use it. Store this file in a safe location.

```
$ mob export d7
You are about to export the secret entropy mnemonic for this account:

d7efc1 (synced)
  address 6eRwLkggafsMs8Mef3JPLExksDpG7BRdYtaDhLkNn1c3AkcdZegJXxsxaPGnZZjR8nuz9SmhYHPrZ3yxqfjmbxfefCK6RqXqNfD9w4T9Hb7
  1.0000 MOB

Keep the exported entropy file safe and private!
Anyone who has access to the entropy can spend all the funds in the account.
Really write account entropy mnemonic to a file? (Y/N) Y
Wrote mobilecoin_secret_mnemonic_d7efc1.json.

$ mv mobilecoin_secret_mnemonic_d7efc1.json /path/to/safe/location
```


## Run unittests

Set an environment variable to tell the unittests where your wallet export file is, then run the unittests.
```
$ export MC_WALLET_FILE=/path/to/safe/location/mobilecoin_secret_mnemonic_d7efc1.json.
$ poetry run pytest
```
