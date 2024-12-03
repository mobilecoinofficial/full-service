#!/bin/bash

set -e
set -o pipefail


echo "-- Checking block height - wait for full-service to start"

TARGET_FS_URL=${TARGET_FS_URL:-"http://localhost:9090/wallet/v2"}

curl_post()
{
    curl --connect-timeout 2 -sSfL -X POST -H 'Content-type: application/json' "${TARGET_FS_URL}" --data '{"method": "get_wallet_status", "jsonrpc": "2.0", "id": 1}' 2>/dev/null
}

count=0
while ! curl_post
do
    echo "-- Waiting for full-service to respond"
    count=$((count + 1))
    if [[ "${count}" -gt 300 ]]
    then
        echo "Full-service did not start"
        exit 1
    fi
    sleep 2
done

# wait for blocks
wallet_json=$(curl_post)
network_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.network_block_height)
local_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.local_block_height)

while [[ "${local_block_height}" != "${network_block_height}" ]]
do
    echo "-- Waiting for blocks to download ${local_block_height} of ${network_block_height}"

    wallet_json=$(curl_post)
    network_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.network_block_height)
    local_block_height=$(echo "${wallet_json}" | jq -r .result.wallet_status.local_block_height)

    sleep 10
done

echo "-- full-service sync is done"
