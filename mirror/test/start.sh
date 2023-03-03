#!/bin/bash
set -e
#
# Run from a `./mob prompt` shell
# Use tools/build-fs.sh to build the binaries

export RUST_LOG=info

usage()
{
    echo "Usage:"
    echo "${0} --network <main|test>"
}

while (( "$#" ))
do
    case "${1}" in
        --help | -h)
            usage
            exit 0
            ;;
        --network)
            network="${2}"
            shift 2
            ;;
        *)
            echo "${1} unknown option"
            usage
            exit 1
            ;;
    esac
done

if [[ -z "${network}" ]]
then
    echo "--network <main|test> is required"
    exit 1
fi

# 1: pid file to check
check_pid_file()
{
    if [[ -f "${1}" ]]
    then
        pid=$(cat "${1}")
        if ps -p "${pid}" > /dev/null
        then
            echo "running"
        else
            echo "not running"
        fi
    fi
}

test_dir=/tmp/mirror_test
git_base=$(git rev-parse --show-toplevel)
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"${git_base}/target/docker"}
bin_dir="${CARGO_TARGET_DIR}"/release

mkdir -p "${test_dir}"
pushd "${test_dir}" >/dev/null

if [[ ! (-f "server.key" && -f "server.crt") ]]
then
    echo "- Generate self-signed TLS cert"
    openssl req -x509 -sha256 -nodes -newkey rsa:2048 -days 365 -keyout server.key -out server.crt -subj "/C=US/ST=CA/L=SF/O=MobileCoin/OU=IT/CN=localhost"
else
    echo "- TLS cert already exists"
fi

if [[ -f "${bin_dir}/generate-rsa-keypair" ]]
then
    if [[ ! (-f "mirror-client.pem" && -f "mirror-private.pem") ]]
    then
        echo "- Generate mirror message keypair"
        "${bin_dir}"/generate-rsa-keypair
    else
        echo "- Mirror message keypair already exists"
    fi
else
    echo "${bin_dir}/generate-rsa-keypair not found, did you build with tools/build-fs.sh?"
    exit 1
fi

if [[ -f "${bin_dir}/full-service" ]]
then
    if [[ "$(check_pid_file .full-service.pid)" == "running" ]]
    then
        echo "full-service is already running"
    else
        echo "- Starting full-service with validator-service"
        exec "${git_base}"/tools/run-fs.sh --validator "${network}" >./full-service.log 2>&1 &
        echo $! >.full-service.pid
    fi
else
    echo "${bin_dir}/full-service not found, did you build with tools/build-fs.sh?"
    exit 1
fi

if [[ -f "${bin_dir}/wallet-service-mirror-private" ]]
then
    if [[ "$(check_pid_file .mirror-private.pid)" == "running" ]]
    then
        echo "wallet-service-mirror-private is already running"
    else
        echo "- Starting wallet-service-mirror-private"
        exec "${bin_dir}"/wallet-service-mirror-private \
            --mirror-public-uri "wallet-service-mirror://127.0.0.1:10000/?ca-bundle=server.crt&tls-hostname=localhost" \
            --wallet-service-uri http://127.0.0.1:9090/wallet/v2 \
            --mirror-key ./mirror-private.pem \
            >./mirror-private.log 2>&1 &
        echo $! >.mirror-private.pid
    fi
else
    echo "${bin_dir}/wallet-service-mirror-private not found, did you build with tools/build-fs.sh?"
    exit 1
fi

if [[ -f "${bin_dir}/wallet-service-mirror-public" ]]
then
    if [[ "$(check_pid_file .mirror-public.pid)" == "running" ]]
    then
        echo "wallet-service-mirror-public is already running"
    else
        echo "- Starting wallet-service-mirror-public"
        exec "${bin_dir}"/wallet-service-mirror-public \
            --client-listen-uri "http://127.0.0.1:9091/" \
            --mirror-listen-uri "wallet-service-mirror://127.0.0.1:10000/?tls-chain=server.crt&tls-key=server.key" \
            --allow-self-signed-tls \
            >./wallet-service-mirror-public.log 2>&1 &
        echo $! >.mirror-public.pid
    fi
else
    echo "${bin_dir}/wallet-service-mirror-private not found, did you build with tools/build-fs.sh?"
    exit 1
fi

# wait for ready
curl_post()
{
    curl --connect-timeout 2 -sSL -X POST -H 'Content-type: application/json' http://localhost:9090/wallet/v2 --data '{"method": "get_wallet_status", "jsonrpc": "2.0", "id": 1}' 2>/dev/null
}

while [[ $(curl_post | jq -r .result.wallet_status.is_synced_all) != "true" ]]
do
    echo "- Waiting for full-service to sync"
    sleep 10
done

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

echo "- All services started. Use ./stop.sh to shutdown all services."

popd 2>/dev/null
