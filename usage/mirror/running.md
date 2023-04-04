# Running

In the examples below we assume that full-service, and both the public and private sides are all running on the same machine. In a real-world scenario the public and private sides would run on separate machines. The following TCP ports are in play:

* 9090: The port full-service listens on for incoming connections.
* 9091: The default port `wallet-service-mirror-public` listens on for incoming HTTP client requests.
* 10080: The default port the mirror uses for GRPC connections.

### Start Full Service

The first step in running the mirror is to have a full-service instance running, and accessible from where the private side of the mirror would be running.

### Start the Public Mirror

To start the public mirror, run the following:

```bash
./wallet-service-mirror-public \
    --client-listen-uri http://0.0.0.0:9091/ \
    --mirror-listen-uri "insecure-wallet-service-mirror://0.0.0.0/"
```

### Start the Private Mirror

To start the private mirror, run the following:

```sh
./bin/wallet-service-mirror-private \
  --mirror-public-uri "insecure-wallet-service-mirror://localhost/" \
  --wallet-service-uri http://localhost:9090/wallet/v2
```

{% hint style="info" %}
Notice the --wallet-service-uri flag is targeting wallet/v2. If you would rather target v1 endpoints, remove `/v2` from the end. ie: `http://localhost:9090/wallet`.
{% endhint %}

### Test Request

Once launched, you can test it using curl:

```bash
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

Returns:

```json
{"method":"get_block","result":{"block":{"id":"dba9b5bb61dc3941c6730a4c5e9b81f30f9def32abd4251d0715100072a7425e","version":"0","parent_id":"0000000000000000000000000000000000000000000000000000000000000000","index":"0","cumulative_txo_count":"16","root_element":{"range":{"from":"0","to":"0"},"hash":"0000000000000000000000000000000000000000000000000000000000\
000000"},"contents_hash":"882cea8bf5e082294ae1707ad2841c6f4846ece978d077f15bc090ac97885e81"},"block_contents":{"key_images":[],"outputs":[{"amount":{"commitment":"3a72e2231c1462354dfe6d4c289d05c67a528dfcdba52d8d87c07914c507dc5f","masked_value":"28067792405079518"},"target_key":"8c43d0e80adcf7c8a59f6350d010f7b257f2d6454efa7ca693eb92180a06ee6c","public_key":\
"50c5916be94c0dcba5054fe2852422ec7c5e208cb31355b8e74e8c4ed007a60b","e_fog_hint":"05e32fee11b4612c9fd54f97e9662c8e576ab91d062c62295974cdd940d0a257eb8ce687e9bbbf8e6dccb0ec16bf15ad6902f9c249d2fe1ed198918ec1c614a48b299c657aa32b9e5c3580f24c07e354b31e0100"},{"amou...
```

For supported requests, the response types are identical to the ones used by `full-service`
