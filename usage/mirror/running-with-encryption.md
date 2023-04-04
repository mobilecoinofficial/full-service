# Running With Encryption

It is possible to run the mirror in a mode that causes it to encrypt requests and responses between the private side and the client. In this mode, anyone having access to the public side of the mirror will be unable to tamper with requests/responses or view them. When running in this mode, which is enabled by passing the `--mirror-key` argument to the private side of the mirror, only encrypted requests will be processed and only encrypted responses will be returned.

In order to use this mode, follow the following steps.

1. Ensure that you have NodeJS installed. **The minimum supported version is v12.9.0** (`node -v`)
2. Generate a keypair: `./bin/generate-rsa-keypair`. This will generate two files: `mirror-client.pem` and `mirror-private.pem`.

To run with encryption (tls is optional but recommended), use the following command to start the private service:

```sh
./bin/wallet-service-mirror-private \
  --mirror-public-uri "wallet-service-mirror://localhost/?ca-bundle=server.crt&tls-hostname=localhost" \
  --wallet-service-uri http://localhost:9090/wallet/v2 \
  --mirror-key mirror-private.pem
```

{% hint style="info" %}
Notice the --mirror-key flag with the mirror-private.pem file, generated with the generate-rsa-keypair binary.
{% endhint %}

To test with encryption, please use the [example mirror-client.js](https://github.com/mobilecoinofficial/full-service/blob/main/mirror/test/mirror-client.js).

```bash
node example-client.js 127.0.0.1 9091 mirror-client.pem \
'{"method": "get_block", "params": {"block_index": "0"}, "jsonrpc": "2.0", "id": 1}'
```
