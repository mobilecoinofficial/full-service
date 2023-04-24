#!/bin/bash

set -e
set -o pipefail


echo "Checking block height - wait for full-service to start"
sleep 15

curl_post()
{
    curl --connect-timeout 2 -sSL -X POST -H 'Content-type: application/json' http://localhost:9090/wallet/v2 --data '{"method": "get_wallet_status", "jsonrpc": "2.0", "id": 1}' 2>/dev/null
}

# wait for blocks
wallet_json=$(curl_post)
network_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.network_block_height)
local_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.local_block_height)

while [[ "${local_block_height}" != "${network_block_height}" ]]
do
    echo "- Waiting for blocks to download ${local_block_height} of ${network_block_height}"

    wallet_json=$(curl_post)
    network_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.network_block_height)
    local_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.local_block_height)

    sleep 10
done

echo "full-service sync is done"
