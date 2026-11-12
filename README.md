[![codecov](https://codecov.io/github/mobilecoinofficial/full-service/branch/main/graph/badge.svg?token=KqBsyfOOHW)](https://codecov.io/github/mobilecoinofficial/full-service)
# Full Service

A MobileCoin service for wallet implementations.

The Full-Service Node provides ledger syncing and validation, account management, and funds transfer and receiving. It uses a JSONRPC API, so you can connect to it from command line tools or build services around its functionality. It serves the use cases of single user (and is the backing to the MobileCoin Desktop Wallet), while also serving high performance, multi-account, multi-subaddress needs (such as backing merchant services platforms).

### For installation and usage instructions, get started with Full Service [here](https://mobilecoin.gitbook.io/full-service-api/usage/environment-setup)!


* You must read and accept the [Terms of Use for MobileCoins and MobileCoin Wallets](./TERMS-OF-USE.md) to use
  MobileCoin Software.

#### Note to Developers

* Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for notes on contributing bug reports and code.

## Table of Contents

* [License](#license)
* [Usage and Documentation](#usage-and-documentation)
* [Run the wallet service](#run-the-wallet-service)
* [Build with Docker](#build-with-docker)
* [Build locally](#build-locally)
* [Build your own docker image](#build-your-own-docker-image)
* [Parameters](#parameters)
* [API Key](#api-key)
* [Exit Codes](#exit-codes)
* [Contributing](#contributing)
* [Database Schema](#database-schema)
* [Running Tests](#running-tests)
* [Linting](#linting)

## License

MobileCoin Full Service is available under open-source licenses. Look for the [LICENSE](./LICENSE) file for more
information.

## Usage and Documentation

For documentation, usage, and API specification, please see the full API documentation at: [Full Service API](https://mobilecoin.gitbook.io/full-service-api/)

Database encryption features are also described in the [Database Encryption](https://mobilecoin.gitbook.io/full-service-api/usage/database-usage#database-encryption)
section of the full API docs.


## Run the wallet service

You can run the wallet service directly from the official docker image.
Lookup the latest release tag (vX.Y.Z-testnet or vX.Y.Z-mainnet) on the [full-service dockerhub page](https://hub.docker.com/r/mobilecoin/full-service/tags?page=1&name=testnet) and update the `docker pull` and `docker run` commands below accordingly.

Here are the commands to run a testnet full-service:
```sh
docker pull mobilecoin/full-service:v2.5.0-testnet
mkdir -p -m go+w ~/.mobilecoin/test
docker run -it -p 127.0.0.1:9090:9090 \
   --volume ~/.mobilecoin/test:/data \
   mobilecoin/full-service:v2.5.0-testnet \
   --peer mc://node1.test.mobilecoin.com/ \
   --peer mc://node2.test.mobilecoin.com/ \
   --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
   --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
   --ledger-db /data/ledger \
   --wallet-db /data/wallet/wallet.db \
   --chain-id test
```

And here is the mainnet [full-service dockerhub page](https://hub.docker.com/r/mobilecoin/full-service/tags?page=1&name=mainnet) and commands to run:
```sh
docker pull mobilecoin/full-service:v2.5.0-mainnet
mkdir -p -m go+w ~/.mobilecoin/main
docker run -it -p 127.0.0.1:9090:9090 \
   --volume ~/.mobilecoin/main:/data \
   mobilecoin/full-service:v2.5.0-mainnet \
   --peer mc://node1.prod.mobilecoinww.com/ \
   --peer mc://node2.prod.mobilecoinww.com/ \
   --tx-source-url https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/ \
   --tx-source-url https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com/ \
   --ledger-db /data/ledger \
   --wallet-db /data/wallet/wallet.db \
   --chain-id main
```

These commands expect that you have your ledger and wallet databases located at `~/.mobilecoin/test` or `~/.mobilecoin/main`.

## Build with Docker

You can build the wallet service using the `mob` tool, which creates a docker container set up for correct compliation. This is called the "builder image".

**Prerequisites**
- git
- docker
- a secret passphrase
- bash (to run outside the `mob` container)

**Instructions**
1. Clone the full-service repository using git
    ```sh
    git clone https://github.com/mobilecoinofficial/full-service.git
    ```
1. From the full-service project root, pull the submodule.
    ```sh
    git submodule update --init --recursive
    ```
1. Now that the submodule is pulled, you can use the `mob` tool by running it from the full-service project root
    ```sh
    ./mob prompt --tag=latest
    ```
1. Build the MobileCoin full-service wallet for testnet. Substitute `test` with `main` to build for mainnet.
    ```sh
    tools/build-fs.sh test
    ```

## Build locally

Note: Full-Service and mobilecoin are not currently compatible with Xcode 13 or higher (the Xcode that ships with OSX Monterey and later). Make sure you are using Xcode 12 before building and running Full-service. You can [download Xcode 12 from Apple's developer downloads page](https://developer.apple.com/download/all/?q=xcode%2012).

Download the latest Xcode 12 and add it to your applications folder.

If you are on OSX Monterey or higher, you will need to fake the version to get OSX to allow you to open it.  Follow these steps (for Xcode 12.5.1):

```sh
# Change build version to Xcode 13.1
/usr/libexec/PlistBuddy -c 'Set :CFBundleVersion 19466' /Applications/Xcode_12.5.1.app/Contents/Info.plist

# Open Xcode (system will check build version and cache it)
open /Applications/Xcode_12.5.1.app/

# Revert Xcode's build version
/usr/libexec/PlistBuddy -c 'Set :CFBundleVersion 18212' /Applications/Xcode_12.5.1.app/Contents/Info.plist
```

Then set your system to use it with:
```sh
sudo xcode-select -s /Applications/Xcode_12.5.1.app/Contents/Developer
```

1. Install Rust from https://www.rust-lang.org/tools/install

2. Install dependencies.

    On Ubuntu:
    ```sh
    sudo apt install build-essential cmake protobuf-compiler libprotobuf-dev llvm llvm-dev clang libclang-dev libsqlite3-dev libssl-dev lcov
    ```

    On Arch Linux:
    ```sh
    sudo pacman -S base-devel cmake protobuf llvm-libs clang sqlite sqlcipher openssl-1.1
    ```

    On MacOS:
    ```sh
    brew bundle
    ```

    After openSSL has been installed with brew on MacOS, you may need to set some environment variables to allow the rust compiler to find openSSL

    Ubuntu:
    ```sh
    PATH="/usr/local/opt/openssl@3/bin:$PATH"
    LDFLAGS="-L/usr/local/opt/openssl@3/lib"
    CPPFLAGS="-I/usr/local/opt/openssl@3/include"
    PKG_CONFIG_PATH="/usr/local/opt/openssl@3/lib/pkgconfig"
    ```

    The `ulimit` command fixes an issue related to shell resource usage. 

    MacOS:
    ```sh
    echo 'ulimit -n 4096' >> ~/.bash_profile
    echo 'export PATH="/opt/homebrew/opt/openssl@3/bin:$PATH"' >> ~/.bash_profile
    source ~/.bash_profile
    export LDFLAGS="-L/opt/homebrew/opt/openssl@3/lib"
    export CPPFLAGS="-I/opt/homebrew/opt/openssl@3/include"
    export PKG_CONFIG_PATH="/opt/homebrew/opt/openssl@3/lib/pkgconfig"
    ```

    Finally, for both:
    ```sh
    rustup component add llvm-tools-preview
    ```

3. Pull submodule.

    ```sh
    git submodule update --init --recursive
    ```

4. Install SGX libraries (required for linux distros; not required for MacOS).

    On Ubuntu:
    ```sh
    wget https://download.01.org/intel-sgx/sgx-linux/2.9.1/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.9.101.2.bin
    chmod +x sgx_linux_x64_sdk_2.9.101.2.bin
    sudo ./sgx_linux_x64_sdk_2.9.101.2.bin --prefix=/opt/intel
    ```

    Put this line in your .bashrc or .zhrc:
    ```sh
    source /opt/intel/sgxsdk/environment
    ```
    This works on more recent Ubuntu distributions, even though it specifies 18.04.

5. Put this line in your .bashrc or .zshrc to specify where OPENSSL is installed.

    Ubuntu:
    ```sh
    export OPENSSL_ROOT_DIR="/usr/local/opt/openssl@3"
    ```

    Arch Linux:
    ```sh
    export OPENSSL_LIB_DIR=/usr/lib/openssl-1.1 OPENSSL_INCLUDE_DIR=/usr/include/openssl-1.1
    ```

    OSX:
    ```sh
    echo 'export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl\@3"' >> ~/.bash_profile
    ```

7. Build

    ```sh
    ./tools/build-fs.sh test
    ```

8. Set database password if using encryption.

    ```sh
    read -rs MC_PASSWORD
    export MC_PASSWORD=$MC_PASSWORD
    ```

9. Run

    ```sh
    ./tools/run-fs.sh test
    ```

See [Parameters](#parameters) for full list of available options.


## Build your own docker image

1. Pull submodule.

    ```sh
    git submodule update --init --recursive
    ```

    2. Build

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

3. Run

   **Volumes**

   This image includes `VOLUME /data` and configures `full-service` to use that path for the wallet and ledger database
   locations.

   If you want to save your databases outside the default volume you can use `-v path/to/volume:/data` but you
   must `chown 1000:1000 path/to/volume` so the app running as uid 1000 can access it.

   Then run your image as above.



## Parameters

These are the parameters to the full-service executable.

| Param            | Purpose                  | Requirements              |
| :--------------- | :----------------------- | :------------------------ |
| `ledger-db`      | Path to ledger directory | Created if does not exist |
| `peer`           | URI of consensus node. Used to submit <br /> transactions and to check the network <br /> block height. | MC URI format |
| `tx-source-url`  | S3 location of archived ledger. Used to <br /> sync transactions to the local ledger. | S3 URI format |
| `chain-id`       | The chain id of the network we expect to interact with | String |

| Optional Param | Purpose                      | Requirements              |
| :------------- | :--------------------------- | :------------------------ |
| `wallet-db`    | Path to wallet file. If not set, will disable any endpoints that require a wallet_db  | Created if does not exist |
| `watcher-db`   | Path to watcher directory    | Created if does not exist |
| `listen-host`  | Host to listen on.           | Default: 127.0.0.1 |
| `listen-port`  | Port to start webserver on.  | Default: 9090 |
| `ledger-db-bootstrap` | Path to existing ledger_db that contains the origin block, <br /> used when initializing new ledger dbs. |  |
| `quorum-set` | Quorum set for ledger syncing. | Default includes all `peers` |
| `poll-interval` | How many seconds to wait between polling for new blocks. | Default: 5 |
| `offline` | Use Full Service in offline mode. This mode does not download new blocks or submit transactions. | |
| `fog-ingest-enclave-css` | Path to the Fog ingest enclave sigstruct CSS file. | Needed in order to enable sending transactions to fog addresses. |
| `allowed-origin`         | URL of the client for CORS headers. '\*' to allow all origins                                            | If not provided, no CORS headers will be set                     |

### Parameters as Environment Variables
All available parameters can be set as Environment Variables. Parameters names are converted to `SCREAMING_SNAKE_CASE` and are prefixed with `MC_`. See `full-service --help` for the full list. CLI arguments take precedence over Environment Variables.

Any options that can be specified multiple times as a list (`--peer`, `--tx-source-url`) can be specified as comma delimited values.

**TestNet example**
```sh
MC_PEER="mc://node1.test.mobilecoin.com/,mc://node2.test.mobilecoin.com/" \
MC_TX_SOURCE_URL="https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/,https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/" \
MC_WALLET_DB="./testnet-dbs/wallet.db" \
MC_LEDGER_DB="./testnet-dbs/ledger-db/" \
MC_CHAIN_ID="test" \
MC_FOG_INGEST_ENCLAVE_CSS="$(pwd)/ingest-enclave.css" \
    ./full_service
```


## API Key

You can add an optional API key to full service by adding a `.env` file to the root of this repo. The variable you need to set is: `MC_API_KEY="<api key of your choosing>"`. If you set this env var, you must provide the `X-API-KEY` header in your requests to full-service.

## Exit Codes

The process exit code indicates why it exited:

| Code | Meaning                              |
| :--- | :----------------------------------- |
| 1    | Unknown error                        |
| 2    | Could not connect to database.       |
| 3    | Wrong database password.             |
| 4    | Connecting from a banned IP address. |
| 101  | Rust Panic.                          |


## Contributing

See [CONTRIBUTING](./CONTRIBUTING.md).


## Database Schema

To add or edit tables:

1. Ensure that you have `diesel_cli` installed and that it is using the current sqlite
   version:

   ```sh
   cargo install --git="https://github.com/mobilecoinofficial/diesel" --rev="026f6379715d27c8be48396e5ca9059f4a263198" diesel_cli --no-default-features --features sqlite
   ```

1. `cd full-service`

1. Create an empty version of the base db `diesel migration run --database-url $MIGRATION_TEST_DB`

1. Create a migration with `diesel migration generate <migration_name>`

1. Edit the migrations/<migration_name>/up.sql and down.sql.

1. Run the migration with `diesel migration run --database-url $MIGRATION_TEST_DB`, and test the
   inverse operation with `diesel migration redo --database-url $MIGRATION_TEST_DB`

Make sure that the following is still present in `schema.rs` before committing changes.

```graphql
table! {
    __diesel_schema_migrations(version) {
        version -> Text,
        run_on -> Timestamp,
    }
}
```

Note that full-service/diesel.toml provides the path to the schema.rs which will be updated in a migration.


## Running Tests

The simple way:
```sh
./tools/test.sh
```

Under the covers, this runs:
```sh
SGX_MODE=HW \
IAS_MODE=DEV \
CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css \
CARGO_INCREMENTAL=0 \
RUSTFLAGS='-Cinstrument-coverage' \
LLVM_PROFILE_FILE="../target/profraw/json5format-%m.profraw" \
cargo test
```

Note: providing the `CONSENSUS_ENCLAVE_CSS` allows us to bypass the enclave build.

Also note: On OSX there is sometimes weird behavior when first running the test suite where some tests will fail.  Opening a new terminal tab and running them again typically resolves this.

## Linting

```sh
RUST_LOG=info \
SGX_MODE=HW \
IAS_MODE=DEV \
CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css \
cargo clippy --all --all-features
```
