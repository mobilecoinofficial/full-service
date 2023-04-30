# Running With TLS

The GRPC connection between `full-service` and `validator-service` can optionally be TLS-encrypted. If you wish to use
TLS for that, you'll need a certificate file and the matching private key for it. For testing purposes you can generate
your own self-signed certificate:

```bash
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

{% hint style="info" %}
Note that the `Common Name` needs to match the hostname which you would be using to connect to the public side (that has
the GRPC listening port).
{% endhint %}

* Now, you can run the VS with TLS enabled:

  ```sh
  mkdir -p ./validator-ledger-db
  ./validator-service \
      --ledger-db ./validator-ledger-db/ \
      --peer mc://node1.test.mobilecoin.com/ \
      --peer mc://node2.test.mobilecoin.com/ \
      --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
      --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
      --listen-uri "validator://localhost:5554/?tls-chain=server.crt&tls-key=server.key"
  ```

  Notice that the `--listen-uri` argument has changed and points to the key and certificate you generated.
* Once the VS is running, you will need to run `full-service`:

  ```sh
  mkdir -p wallet-db/
  ./full-service \
     --wallet-db ./wallet-db/wallet.db \
     --ledger-db ./ledger-db/ \
     --validator "validator://localhost:5554/?ca-bundle=server.crt&tls-hostname=localhost" \
     --fog-ingest-enclave-css $(pwd)/ingest-enclave.css \
     --listen-port 9090
  ```

  The `--validator` argument has changed to point at the certificate file, and also specify the Common Name that is in
  the certficiate. Note that if the CN matches the hostname (as in the above example) then this is redundant.
