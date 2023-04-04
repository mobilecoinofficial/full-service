# Running With TLS

### Generate TLS Credentials

In order to have a tls connection between the public and private sides of the mirror, you need to use a certificate pair. For testing, you can generate these with

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
Note that the `Common Name` needs to match the hostname which you would be using to connect to the public side (that has the GRPC listening port).
{% endhint %}

### Start the Public Mirror

To run with encryption, use the following command

```sh
./wallet-service-mirror-public \
  --client-listen-uri http://0.0.0.0:9091/ \
  --mirror-listen-uri "wallet-service-mirror://0.0.0.0/?tls-chain=server.crt&tls-key=server.key" \
  --allow-self-signed-tls
```

### Start the Private Mirror

To run with encryption, use the following command

```sh
./bin/wallet-service-mirror-private \
  --mirror-public-uri "wallet-service-mirror://localhost/?ca-bundle=server.crt&tls-hostname=localhost" \
  --wallet-service-uri http://localhost:9090/wallet/v2
```
