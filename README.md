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
    sudo apt install build-essential cmake protobuf-compiler libprotobuf-dev llvm llvm-dev clang libclang-dev libsqlite3-dev libssl1.1
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

1. Install SGX libraries.

    On Ubuntu:
    ```sh
    wget https://download.01.org/intel-sgx/sgx-linux/2.9.1/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.9.101.2.bin
    chmod +x sgx_linux_x64_sdk_2.9.101.2.bin
    sudo ./sgx_linux_x64_sdk_2.9.101.2.bin --prefix=/opt/intel
    ```

    Put this line in your .bashrc:
    ```sh
    source /opt/intel/sgxsdk/environment
    ```

    This works on more recent Ubuntu distributions, even though it specifies 18.04.


1. Build

    ```sh
    export SGX_MODE=HW IAS_MODE=PROD CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css
    cargo build --release -p mc-full-service
    ```

1. Run

    TestNet Example

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

    See [Parameters](#parameters) for full list of available options.



## Docker Build and Run

1. Pull submodule.

    ```sh
    git submodule init
    git submodule update
    ```

1. Build

    This build takes advantage of features in Docker BuildKit use `DOCKER_BUILDKIT=1` when building this image.

    See the [Dockerfile](./Dockerfile) comments for the full list of available build arguments to customize the build.

    **TestNet Version**

    Use `--build-arg NAMESPACE=test` to configure build to use TestNet enclave measurements.

    ```sh
    DOCKER_BUILDKIT=1 docker build -t mobilecoin/full-service:0.0.0-testnet --progress=plain \
    --build-arg NAMESPACE=test \
    --build-arg BUILD_OPTS=--no-default-features .
    ```

    **MainNet Version**

    Use `--build-arg NAMESPACE=prod` to configure build to use MainNet enclave measurements.

    ```sh
    DOCKER_BUILDKIT=1 docker build -t mobilecoin/full-service:0.0.0 --progress=plain \
        --build-arg NAMESPACE=prod .
    ```

1. Run

    **Volumes**

    This image includes `VOLUME /data` and configures `full-service` to use that path for the wallet and ledger database locations.

    If you want to save your databases outside the default volume you can use `-v path/to/volume:/data` but you must `chown 1000:1000 path/to/volume` so the app running as uid 1000 can access it.

    ```sh
    mkdir -p /opt/full-service/data

    chown 1000:1000 /opt/full-service/data

    docker run -it -p 127.0.0.1:9090:9090 \
        -v /opt/full-service/data:data \
        --name full-service \
        mobilecoin/full-service \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
    ```


    **Listen and Port**

    This image configures `full-service` to listen on the container `0.0.0.0:9090`

    Use `-p 127.0.0.1:9090:9090` to expose the API to you localhost.

    **Run**

    Required parameters are added as command options to the container.

    TestNet Example

    ```sh
    docker run -it -p 127.0.0.1:9090:9090 --name full-service mobilecoin/full-service \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
    ```

    See [Parameters](#parameters) for full list of available options.

## Parameters

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

For usage and API specification, see [API.md](API.md).

For database encryption features, see [DATABASE.md](DATABASE.md).


### Offline (Cold-Wallet) Transaction Flow

Full Service supports offline transactions. This flow is recommended to keep an account key on an air-gapped machine which has never connected to the internet.

The recommended flow to get balance and submit transaction is the following:

1. *ONLINE MACHINE*: Sync ledger by running full service.

    ```sh
    ./target/release/full-service \
        --wallet-db /tmp/wallet-db/wallet.db \
        --ledger-db /tmp/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
    ```

1. *ONLINE MACHINE and USB*: Copy the ledger and the full-service binary to USB.

    ```sh
    cp -r /tmp/ledger-db /media/
    cp ./target/release/full-service /media/
    ```

1. *OFFLINE MACHINE*: Create a ramdisk to store sensitive material.

    * Linux: The following will create a 512 MB ramdisk located at `/keyfs`.

        ```sh
        sudo swapoff -a
        sudo mkdir /keyfs
        sudo mount -t tmpfs -o size=512m tmpfs /keyfs
        ```

    * MacOS: The following will create a 512 MB ramdisk located at `/Volumes/KeyFS`.

        ```sh
        diskutil erasevolume HFS+ 'KeyFS' `hdiutil attach -nomount ram://1048576`
        ```

        For the remaining instructions, we will refer to `/keyfs` as the ramdisk location, so if on MacOS, know that this maps to `/Volumes/KeyFS`.

1. *OFFLINE MACHINE and USB*: Copy the ledger and the full-service binary to the Offline Machine.

    ```sh
    cp /media/ledger-db /keyfs/ledger-db
    cp /media/full-service /keyfs/full-service
    ```

1. *OFFLINE MACHINE*: Run full service in offline mode.

    ```sh
    ./target/release/full-service \
        --wallet-db /keyfs/wallet.db \
        --ledger-db /keyfs/ledger-db/ \
        --offline
    ```

1. *OFFLINE MACHINE*: You can now [create](#create-account) or [import](#import-account) your account, [check your balance](#get-balance-for-a-given-account), [create assigned subaddresses](#create-assigned-subaddress), and [construct transactions](#build-transaction), as outlined in the docs above. Note that your entropy, account key, and wallet.db should always remain only on the ramdisk.

1. *OFFLINE MACHINE and USB*: To send a transaction, you will a Construct a TxProposal using the [`build_transaction`](#build-transaction) endpoint. The output of this call can be written to a file, such as tx_proposal.json, and then copied to the USB.

    ```sh
    curl -s localhost:9090/wallet \
    -d '{
    "method": "build_transaction",
    "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value": "42000000000000"
    }
    }' \
    -X POST -H 'Content-type: application/json' | jq '.result' > /keyfs/tx_proposal.json

    cp /keyfs/tx_proposal.json /media/
    ```

1. *ONLINE MACHINE and USB*: Copy the tx_proposal to the online machine.

    ```sh
    cp /media/tx_proposal.json ./
    ```

1. *ONLINE MACHINE*: Submit transaction to consensus, using the [`submit_transaction`](#submit-transaction) endpoint.

## Contributing

See [CONTRIBUTING](./CONTRIBUTING.md).

### Database Schema

To add or edit tables:

1. Ensure that you have `diesel_cli` installed and that it is using the current sqlite version: `cargo install diesel_cli --no-default-features --features "sqlite sqlite-bundled" --git https://github.com/eranrund/diesel --rev 25592f0383a1d1628db7d2db6c1fb02614a978c1`
1. `cd full-service`
1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run --database-url /tmp/db.db`, and test delete with `diesel migration redo --database-url /tmp/db.db`

Note that full-service/diesel.toml provides the path to the schema.rs which will be updated in a migration.

### Running Tests

    ```
    SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo test
    ```

    Note: providing the `CONSENSUS_ENCLAVE_CSS` allows us to bypass the enclave build.

### Linting

    ```
    RUST_LOG=info SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo clippy --all --all-features
    ```
