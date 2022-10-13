# Full Service Mirror, Full Service, & Ledger Validator Node (LVN)

## Requirements

MobileCoin's full-service, full-service mirrors, and ledger-validator-node are developed using the environment specified in [this Dockerfile](https://github.com/mobilecoinfoundation/mobilecoin/blob/bdd5ded7aff9b8a86bd10c568a1f2bcf1ee20d27/docker/Dockerfile).

## Ledger Validator Node & Full Service

The first step is to launch Full Service and the Ledger Validator Node (LVN)

A service that is capable of syncing the ledger from the consensus network, relaying transactions to it and proxying fog report resolution.

The Ledger Validator Node exposes a GRPC service that provides access to its local ledger, transaction relaying and fog report request relaying.

Using the `--validator` command line argument for `full-service`, this allows running `full-service` on a machine that is not allowed to make outside connections to the internet \
but can connect to a host running the LVN.

1. Run the Ledger Validator Node (LVN)

    NOTE: To run the Ledger Validator Node with TLS, see the section [TLS between full-service and LVN](#tls-between-full-service-and-lvn).

    ```sh
    mkdir -p ./lvn-dbs
    ./bin/mc-validator-service \
       --ledger-db ./lvn-dbs/ledger-db/ \
       --peer mc://node1.prod.mobilecoinww.com/ \
       --peer mc://node2.prod.mobilecoinww.com/ \
       --tx-source-url https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com \
       --tx-source-url https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com \
       --listen-uri insecure-validator://localhost:5554/
    ```

    NOTE: the `insecure-` prefix indicates the connection is going over plaintext, as opposed to TLS. If you wish to run with TLS, skip to the next section.

    At this point the LVN is running and accepting connections on port 5554.


2. Run Full Service

    ```sh
    mkdir -p ./fs-dbs/wallet-db/
    ./bin/full-service \
       --wallet-db ./fs-dbs/wallet-db/wallet.db \
       --ledger-db ./fs-dbs/ledger-db/ \
       --validator insecure-validator://localhost:5554/ \
       --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
    ```

    Notice how `--validator` replaced `--peer` and `--tx-source-url`.

### TLS between full-service and LVN

1. The GRPC connection between `full-service` and `mc-ledger-validator` can optionally be TLS-encrypted. If you wish to use TLS for that, you'll need a certificate file and the matching private key for it. For testing purposes you can generate your own self-signed certificate:

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

2. Now, you can run the LVN with TLS enabled:

    ```sh
    mkdir -p ./lvn-dbs
    ./bin/mc-validator-service \
       --ledger-db ./lvn-dbs/ledger-db/ \
       --peer mc://node1.prod.mobilecoinww.com/ \
       --peer mc://node2.prod.mobilecoinww.com/ \
       --tx-source-url https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com \
       --tx-source-url https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com \
       --listen-uri "validator://localhost:5554/?tls-chain=server.crt&tls-key=server.key"
    ```

    Notice that the `--listen-uri` argument has changed and points to the key and certificate you generated.

3. Once the LVN is running, you will need to run `full-service`:

    ```sh
    mkdir -p ./fs-dbs/wallet-db/
    ./bin/full-service \
       --wallet-db ./fs-dbs/wallet-db/wallet.db \
       --ledger-db ./fs-dbs/ledger-db/ \
       --validator "validator://localhost:5554/?ca-bundle=server.crt&tls-hostname=localhost" \
       --fog-ingest-enclave-css $(pwd)/ingest-enclave.css \
       --listen-port 9090
    ```

    The `--validator` argument has changed to point at the certificate file, and also specify the Common Name that is in the certficiate. Note that if the CN matches the hostname (as in the above example) then this is redundant.## TLS between full-service and LVN

## Full Service Mirror

To use, you will need to start both sides of the mirror.

### End-to-end encryption

It is possible to run the mirror in a mode that causes it to encrypt requests and responses between the private side and the client. In this mode, anyone having access to the public side of the mirror will be unable to tamper with requests/responses or view them. When running in this mode, which is enabled by passing the `--mirror-key` argument to the private side of the mirror, only encrypted requests will be processed and only encrypted responses will be returned.

In order to use this mode, follow the following steps.

1) Ensure that you have NodeJS installed. **The minimum supported version is v12.9.0** (`node -v`)

1) Generate a keypair: `./bin/generate-rsa-keypair`. This will generate two files: `mirror-client.pem` and `mirror-private.pem`.

### TLS Connection

In order to have a tls connection between the public and private sides of the mirror, you need to use a certificate pair. For testing, you can generate these with `openssl req -x509 -sha256 -nodes -newkey rsa:2048 -days 365 -keyout server.key -out server.crt`.

Note that the `Common Name` needs to match the hostname which you would be using to connect to the public side (that has the GRPC listening port).

### Public Mirror

If you would like to run this without end to end encryption use the following command

```sh
./bin/wallet-service-mirror-public --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri "insecure-wallet-service-mirror://0.0.0.0/"
```

To run with encryption, use the following command

```sh
./bin/wallet-service-mirror-public --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri "wallet-service-mirror://0.0.0.0/?tls-chain=server.crt&tls-key=server.key" --allow-self-signed-tls
```


### Private Mirror

If you would like to run this without end to end encryption use the following command

```sh
./bin/wallet-service-mirror-private --mirror-public-uri "insecure-wallet-service-mirror://localhost/" --wallet-service-uri http://localhost:9090/wallet
```

To run with encryption, use the following command

```sh
./bin/wallet-service-mirror-private --mirror-public-uri "wallet-service-mirror://localhost/?ca-bundle=server.crt&tls-hostname=localhost" --wallet-service-uri http://localhost:9090/wallet --mirror-key mirror-private.pem
```

NOTE: Notice the --mirror-key flag with the mirror-private.pem file, generated with the generate-rsa-keypair binary.

Once launched, without end to end encryption, you can test it using curl:

Get block information (for block 0):

```
curl -X POST -H 'Content-Type: application/json' -d '{"method": "get_block", "params": {"block_index": "0"}, "jsonrpc": "2.0", "id": 1}' http://localhost:9091/unencrypted-request
```
Returns:
```
{"method":"get_block","result":{"block":{"id":"dba9b5bb61dc3941c6730a4c5e9b81f30f9def32abd4251d0715100072a7425e","version":"0","parent_id":"0000000000000000000000000000000000000000000000000000000000000000","index":"0","cumulative_txo_count":"16","root_element":{"range":{"from":"0","to":"0"},"hash":"0000000000000000000000000000000000000000000000000000000000\
000000"},"contents_hash":"882cea8bf5e082294ae1707ad2841c6f4846ece978d077f15bc090ac97885e81"},"block_contents":{"key_images":[],"outputs":[{"amount":{"commitment":"3a72e2231c1462354dfe6d4c289d05c67a528dfcdba52d8d87c07914c507dc5f","masked_value":"28067792405079518"},"target_key":"8c43d0e80adcf7c8a59f6350d010f7b257f2d6454efa7ca693eb92180a06ee6c","public_key":\
"50c5916be94c0dcba5054fe2852422ec7c5e208cb31355b8e74e8c4ed007a60b","e_fog_hint":"05e32fee11b4612c9fd54f97e9662c8e576ab91d062c62295974cdd940d0a257eb8ce687e9bbbf8e6dccb0ec16bf15ad6902f9c249d2fe1ed198918ec1c614a48b299c657aa32b9e5c3580f24c07e354b31e0100"},{"amou...
```

For the full API documentation, please see the [Full Service API](https://mobilecoin.gitbook.io/full-service-api/).

To test with encryption, please use the [example client](https://github.com/mobilecoinofficial/full-service-mirror/blob/master/example-client.js).

```
node example-client.js 127.0.0.1 9091 mirror-client.pem '{"method": "get_block", "params": {"block_index": "0"}, "jsonrpc": "2.0", "id": 1}'
```
