---
description: Set up your environment to run full service on Mac or Linux.
---

# Environment Setup

## Binaries

1. Download [TestNet or MainNet binaries](https://github.com/mobilecoinofficial/full-service/releases). 
2. In a terminal window, navigate to your downoads folder to run the Full Service binaries directory that you just downloaded.

   * If you downloaded TestNet, run:

   ```text
   mkdir -p testnet-dbs
   RUST_LOG=info,mc_connection=info,mc_ledger_sync=info ./full-service \
       --wallet-db ./testnet-dbs/wallet.db \
       --ledger-db ./testnet-dbs/ledger-db/ \
       --peer mc://node1.test.mobilecoin.com/ \
       --peer mc://node2.test.mobilecoin.com/ \
       --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
       --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
       --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
   ```

   * If you downloaded MainNet, run:

   ```text
     mkdir -p mainnet-dbs
     RUST_LOG=info,mc_connection=info,mc_ledger_sync=info ./full-service \
       --wallet-db ./mainnet-dbs/wallet.db \
       --ledger-db ./mainnet-dbs/ledger-db/ \
       --peer mc://node1.prod.mobilecoinww.com/ \
       --peer mc://node2.prod.mobilecoinww.com/ \
       --tx-source-url https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/ \
       --tx-source-url https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com/ \
       --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
   ```

{% hint style="info" %}
Replace our default peers or tx-source-urls if you would prefer to establish your own source of truth.
{% endhint %}

## SQLite3

1. Confirm [Homebrew](https://brew.sh/) is installed.
2. Run `brew info sqlite` to see which version, if any, you have installed.
   * If you do not yet have sqlite, run `brew install sqlite`
   * If your version is outdated, run `brew upgrade sqlite`

## **HTTP Request Service**

1. Install a service, such as [Postman](https://www.postman.com/), to send HTTP requests.

## API Key

You can add an optional API key to full service by adding a `.env` file to the root of this repo. The variable you need to set is: `MC_API_KEY="<api key of your choosing>"`. If you set this env var, you must provide the `X-API-KEY` header in your requests to full-service.

