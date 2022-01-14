# Ledger Validator Node

A service that is capable of syncing the ledger from the consensus network, relaying transactions to it and proxying fog report resolution.

The Ledger Validator Node exposes a GRPC service that provides access to its local ledger, transaction relaying and fog report request relaying.

Using the `--validator` command line argument for `full-service`, this allows running `full-service` on a machine that is not allowed to make outside connections to the internet but can connect to a host running the LVN.

## Basic Usage

1. The first step is to build `full-service`, follow the instructions [here](../README.md#build-and-run).
Note that when building full-service you need to change the build command (`cargo build --release -p mc-full-service`) to `cargo build --release -p mc-full-service --no-default-features` if you are going to run it on a machine that cannot create outgoing TCP connections to the internet.

1. Build the validator:
```sh
    SGX_MODE=HW \
    IAS_MODE=PROD \
    CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css \
    cargo build --release -p mc-validator-service
```

1. Run the LVN


TestNet example

```sh
./target/release/mc-validator-service \
        --ledger-db /tmp/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
        --listen-uri insecure-validator://0.0.0.0:5554/
```

NOTE: the `insecure-` prefix indicates the connection is going over plaintext, as opposed to TLS.

At this point the LVN is running and accepting connections on port 5554.

1. Run full-service

```sh
    mkdir -p /tmp/wallet-db/
    ./target/release/full-service \
        --wallet-db /tmp/wallet-db/wallet.db \
        --ledger-db /tmp/ledger-db/ \
        --validator insecure-validator://127.0.0.1:5554/
        --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
```

Notice how `--validator` replaced `--peer` and `--tx-source-url`.


## TLS between full-service and LVN

The GRPC connection between `full-service` and `mc-ledger-validator` can optionally be TLS-encrypted. If you wish to use TLS for that, you'll need a certificate file and the matching private key for it. For testing purposes you can generate your own self-signed certificate:

```
$ openssl req -x509 -sha256 -nodes -newkey rsa:2048 -days 365 -keyout server.key -out server.crt

Generating a 2048 bit RSA private key
....................+++
.............+++
writing new private key to 'server.key'
-----
You are about to be asked to enter information that will be incorporated
into your certificate request.
What you are about to enter is what is called a Distinguished Name or a DN.
There are quite a few fields but you can leave some blank
For some fields there will be a default value,
If you enter '.', the field will be left blank.
-----
Country Name (2 letter code) []:US
State or Province Name (full name) []:California
Locality Name (eg, city) []:San Francisco
Organization Name (eg, company) []:My Test Company
Organizational Unit Name (eg, section) []:Test Unit
Common Name (eg, fully qualified host name) []:localhost
Email Address []:test@test.com
```

Note that the `Common Name` needs to match the hostname which you would be using to connect to the public side (that has the GRPC listening port).

Now, you can run the LVN with TLS enabled:
```sh
./target/release/mc-validator-service \
        --ledger-db /tmp/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
        --listen-uri "validator://0.0.0.0:5554/?tls-chain=server.crt&tls-key=server.key"
```
Notice that the `--listen-uri` argument has changed and points to the key and certificate you generated.

Once the LVN is running, you will need to run `full-service`:
```sh
    mkdir -p /tmp/wallet-db/
    ./target/release/full-service \
        --wallet-db /tmp/wallet-db/wallet.db \
        --ledger-db /tmp/ledger-db/ \
        --validator "validator://localhost:5554/?ca-bundle=server.crt&tls-hostname=localhost"
        --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
```
The `--validator` argument has changed to point at the certificate file, and also specify the Common Name that is in the certficiate. Note that if the CN matches the hostname (as in the above example) then this is redundant.
