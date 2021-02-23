# Full Service

A MobileCoin service for wallet implementations.

* You must read and accept the [Terms of Use for MobileCoins and MobileCoin Wallets](./TERMS-OF-USE.md) to use MobileCoin Software.
* Please note that currently, the MobileCoin Wallet is not available for download or use by U.S. persons or entities, persons or entities located in the U.S., or persons or entities in other prohibited jurisdictions.

#### Note to Developers
* MobileCoin Full Service is a prototype. Expect substantial changes before the release.
* Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for notes on contributing bug reports and code.

##### License

MobileCoin Full Service is available under open-source licenses. Look for the [LICENSE](./LICENSE) file for more information.

## Build and Run

1. Install Rust from https://www.rust-lang.org/tools/install

1. Install dependencies.

    On Ubuntu:
    ```sh
    sudo apt install build-essential cmake protobuf-compiler llvm libclang-dev libsqlite3-dev libssl1.1
    ```

    On Mac:
    ```sh
    brew bundle
    ```

1. Pull submodule.

    ```sh
    git submodule init
    git submodule update
    ```

1. Get the appropriate published enclave measurement, and save to `$(pwd)/consensus-enclave.css`

    ```sh
    NAMESPACE=test
    SIGNED_ENCLAVE_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
    curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${SIGNED_ENCLAVE_URI}
    ```

1. Build

    ```sh
    SGX_MODE=HW IAS_MODE=PROD CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo build --release -p mc-full-service
    ```

1. Run

    ```sh
    mkdir -p /tmp/wallet-db/
    ./target/release/full-service \
        --wallet-db /tmp/wallet-db/wallet.db \
        --ledger-db /tmp/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
    ```

   | Param            | Purpose                  | Requirements              |
   | :--------------- | :----------------------- | :------------------------ |
   | `wallet-db`      | Path to wallet file      | Created if does not exist |
   | `ledger-db`      | Path to ledger directory | Created if does not exist |
   | `peer`           | URI of consensus node. Used to submit <br /> transactions and to check the network <br /> block height. | MC URI format |
   | `tx-source-url`  | S3 location of archived ledger. Used to <br /> sync transactions to the local ledger. | S3 URI format |

   | Opional Param | Purpose                  | Requirements              |
   | :------------ | :----------------------- | :------------------------ |
   | `listen-host` | Host to listen on.      | Default: 127.0.0.1 |
   | `listen-port` | Port to start webserver on. | Default: 9090 |
   | `ledger-db-bootstrap` | Path to existing ledger_db that contains the origin block, <br /> used when initializing new ledger dbs. |  |
   | `quorum-set` | Quorum set for ledger syncing. | Default includes all `peers` |
   | `num-workers` | Number of worker threads to use for view key scanning. | Defaults to number of logical CPU cores. |
   | `poll-interval` | How many seconds to wait between polling for new blocks. | Default: 5 |
   | `offline` | Use Full Service in offline mode. This mode does not download new blocks or submit transactions. | |
   | `fog-ingest-enclave-css` | Path to the Fog ingest enclave sigstruct CSS file. | Needed in order to enable sending transactions to fog addresses. |

## Usage

For usage and API specification, see [API_v1.md](./API_v1.md).

## Contributing

See [CONTRIBUTING](./CONTRIBUTING.md).

### Database Schema

To add or edit tables:

1. `cd full-service`
1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run --database-url /tmp/db.db`, and test delete with `diesel migration redo --database-url /tmp/db.db`

Note that full-service/diesel.toml provides the path to the schema.rs which will be updated in a migration.

### Running Tests

    ```
    SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo test
    ```

    Note: providing the CONSENESUS_ENCLAVE_CSS allows us to bypass the enclave build.

### Linting

    ```
    RUST_LOG=info SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo clippy --all --all-features
    ```
