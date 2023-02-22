# Python API and CLI for MobileCoin Full-Service Wallet

## Installation

`$ pip install mobilecoin`


## CLI usage

First, you should have an instance of the full-service wallet running on your local machine. Setup instructions are at https://github.com/mobilecoinofficial/full-service.

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


## Run unittests and integration tests.

Set an environment variable to tell the unittests where your wallet export file is.
```
$ export MC_WALLET_FILE='/path/to/safe/location/mobilecoin_secret_mnemonic_d7efc1.json'
```

Optionally set environment variables to run fog tests.
```
$ export MC_FOG_REPORT_URL='fog://fog.test.mobilecoin.com'
$ export MC_FOG_AUTHORITY_SPKI='MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=='
```

With full-service running, start the integration tests.
```
$ poetry run pytest -v
```
