## Wallet Service Mirror

The `wallet-service-mirror` crate consists of two standalone executables, that when used together allow for exposing limited, read-only data from a `full-service` wallet service instance. As explained below, this allows exposing some data from `full-service` from a machine that does not require any incoming connections from the outside world.

The mirror consists of two sides:
   1) A private side. The private side of the mirror runs alongside `full-service` and forms outgoing connections to both `full-service` and to the public side of the mirror. It then proceeds to poll the public side for any requests that should be forwarded to `full-service`, forwards them, and at the next poll opportunity returns any replies. Note how the private side only forms outgoing connections and does not open any listening ports.

   Please Note:
   The set of available requests is defined in the variable `SUPPORTED_ENDPOINTS`, in the [private main file](src/private/main.rs). It is likely you will want to change the `SUPPORTED_ENDPOINTS` to include desired features like sending transactions.
   2) A public side. The public side of the mirror accepts incoming HTTP connections from clients, and poll requests from the private side over GRPC. The client requests are then forwarded over the GRPC channel to the private side, which in turn forwards them to `full-service` and returns the responses.


### Example usage

In the examples below we assume that full-service, and both the public and private sides are all running on the same machine. In a real-world scenario the public and private sides would run on separate machines. The following TCP ports are in play:
   - 9090: The port full-service listens on for incoming connecitons.
   - 9091: The default port `wallet-service-mirror-public` listens on for incoming HTTP client requests.
   - 10080: The default port the mirror uses for GRPC connections.

The first step in running the mirror is to have a full-service instance running, and accessible from where the private side of the mirror would be running. Once full-service is running and set up, start the private side of the mirror:

```
cargo run -p mc-wallet-service-mirror --bin wallet-service-mirror-private -- --mirror-public-uri insecure-wallet-service-mirror://127.0.0.1/ --wallet-service-uri http://localhost:9090/wallet
```


This starts the private side of the mirror, telling it to connect to `full-service` on localhost:9090 and the public side on `127.0.0.1:10080` (10080 is the default port for the `insecure-wallet-service-mirror` URI scheme).

At this point you would see a bunch of error messages printed as the private side attemps to initiate outgoing GRPC connections to the public side. This is expected since the public side is not running yet.

Now, start the public side of the mirror:

```
cargo run -p mc-wallet-service-mirror --bin wallet-service-mirror-public -- --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri insecure-wallet-service-mirror://127.0.0.1/
```

Once started, the private side should no longer show errors and the mirror should be up and running. You can now send client requests, for example - query the genesis block information:

Query block details:

```
curl -s localhost:9091/unencrypted-request \
  -d '{
        "method": "get_block",
        "params": {
          "block_index": "0"
        },
        "jsonrpc": "2.0",
        "id": 1
      }' \
  -X POST -H 'Content-type: application/json' | jq
```

For supported requests, the response types are identical to the ones used by `full-service`.


### TLS between the mirror sides

The GRPC connection between the public and private side of the mirror can optionally be TLS-encrypted. If you wish to use TLS for that, you'll a certificate file and the matching private key for it. For testing purposes you can generate your own self-signed certificate:

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

After the certificate has been geneerated, you can start the public side of the mirror and instruct it to listen using TLS:
```
cargo run -p mc-wallet-service-mirror --bin wallet-service-mirror-public -- --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri 'wallet-service-mirror://127.0.0.1/?tls-chain=server.crt&tls-key=server.key' --allow-self-signed-tls
```

Notice that the `mirror-listen-uri` has been changed from `insecure-wallet-service-mirror` to `wallet-service-mirror`, and that it now contains a `tls-chain` and `tls-key` parameters pointing at the certificate chain file and the matching private key file. The default port for the `wallet-service-mirror` scheme is 10043.

The private side of the bridge also needs to be aware that TLS is now used:
```
cargo run -p mc-wallet-service-mirror --bin wallet-service-mirror-private -- --mirror-public-uri 'wallet-service-mirror://localhost/?ca-bundle=server.crt' --wallet-service-uri http://localhost:9090/wallet
```

Notice that the `mirror-public-uri` parameter has changed to reflect the TLS certificate chain.


### TLS between the public side of the mirror and its HTTP clients

Currently, due to `ring` crate version conflicts, it is is not possible to enable the `tls` feature on `rocket` (the HTTP server used by `mc-wallet-service-mirror-public`). If you want to provide TLS encryption for clients, you would need to put `mc-wallet-service-mirror-public` behind a reverse proxy such as `nginx` and have that take care of your TLS needs.


### End-to-end encryption

It is possible to run the mirror in a mode that causes it to encrypt requests and responses between the private side and the client. In this mode, anyone having access to the public side of the mirror will be unable to tamper with requests/responses or view them. When running in this mode, which is enabled by passing the `--mirror-key` argument to the private side of the mirror, only encrypted requests will be processed and only encrypted responses will be returned.

In order to use this mode, follow the following steps.
1) Ensure that you have NodeJS installed. **The minimum supported version is v12.9.0** (`node -v`)
1) Generate a keypair by running the `generate-rsa-keypair` binary. This will generate two files: `mirror-client.pem` and `mirror-private.pem`.
1) Run the public side of the mirror as usual, for example: `cargo run -p mc-wallet-service-mirror --bin wallet-service-mirror-public -- --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri insecure-wallet-service-mirror://127.0.0.1/`
1) Copy the `mirror-private.pem` file to where you would be running the private side of the mirror, and run it: `cargo run -p mc-wallet-service-mirror --bin wallet-service-mirror-private -- --mirror-public-uri insecure-wallet-service-mirror://127.0.0.1/ --wallet-service-uri http://localhost:9090/wallet --mirror-key mirror-private.pem`. Notice the addition of the `--mirror-key` argument.
1) Issue a response using the sample client:
   - To get block data: `node example-client.js 127.0.0.1 9091 mirror-client.pem '{"method": "get_block", "params": {"block_index": "0"}, "jsonrpc": "2.0", "id": 1}' | jq`
