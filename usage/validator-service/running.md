# Running

To start the validator service, us the following command:

```sh
./validator-service \
    --ledger-db validator-ledger-db/ \
    --peer mc://node1.test.mobilecoin.com/ \
    --peer mc://node2.test.mobilecoin.com/ \
    --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
    --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
    --listen-uri insecure-validator://localhost:5554/
```

At this point the VS is running and accepting connections on port 5554

### Run Full Service

```sh
mkdir -p ./wallet-db/
./full-service \
    --wallet-db wallet-db/wallet.db \
    --ledger-db ledger-db/ \
    --validator insecure-validator://localhost:5554/ \
    --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
```

{% hint style="info" %}
Notice how `--validator` replaced `--peer` and `--tx-source-url`
{% endhint %}
