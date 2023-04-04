# Running Full Service

## Binaries

1. Download the latest TestNet or MainNet binaries for you desired platform for the latest release from the [github releases page](https://github.com/mobilecoinofficial/full-service/releases).
2.  In a terminal window, navigate to the folder containing the downloaded Full Service binaries.

    * If you downloaded TestNet, run:

    ```
    mkdir -p testnet-dbs
    RUST_LOG=info,mc_connection=info,mc_ledger_sync=info ./full-service \
        --wallet-db ./testnet-dbs/wallet.db \
        --ledger-db ./testnet-dbs/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
        --fog-ingest-enclave-css $(pwd)/ingest-enclave.css \
        --chain-id test
    ```

    * If you downloaded MainNet, run:

    ```
      mkdir -p mainnet-dbs
      RUST_LOG=info,mc_connection=info,mc_ledger_sync=info ./full-service \
        --wallet-db ./mainnet-dbs/wallet.db \
        --ledger-db ./mainnet-dbs/ledger-db/ \
        --peer mc://node1.prod.mobilecoinww.com/ \
        --peer mc://node2.prod.mobilecoinww.com/ \
        --tx-source-url https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/ \
        --tx-source-url https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com/ \
        --fog-ingest-enclave-css $(pwd)/ingest-enclave.css \
        --chain-id main
    ```

{% hint style="info" %}
You can replace the default peers or tx-source-urls if you would prefer to establish your own source of truth.
{% endhint %}

## Configuration with Environment Variables.

All available parameters can be set as Environment Variables. Parameters names are converted to `SCREAMING_SNAKE_CASE` and are prefixed with `MC_`. See `full-service --help` for the full list. CLI arguments take precedence over Environment Variables.

To see a full list of available variables, please check out our [github repoitory](https://github.com/mobilecoinofficial/full-service#parameters)!

{% hint style="info" %}
Any options that can be specified multiple times as a list (`--peer`, `--tx-source-url`) can be specified as comma delimited values.
{% endhint %}

**TestNet example**

```
MC_PEER="mc://node1.test.mobilecoin.com/,mc://node2.test.mobilecoin.com/" \
MC_TX_SOURCE_URL="https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/,https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/" \
MC_WALLET_DB="./testnet-dbs/wallet.db" \
MC_LEDGER_DB="./testnet-dbs/ledger-db/" \
MC_CHAIN_ID="test" \
MC_FOG_INGEST_ENCLAVE_CSS="$(pwd)/ingest-enclave.css" \
    ./full_service
```

## **HTTP Request Service**

The Full Service API is reached at `localhost:9090/wallet/v2` by default using the `POST` method.

1. Install a service, such as [Postman](https://www.postman.com/), to send HTTP requests.
