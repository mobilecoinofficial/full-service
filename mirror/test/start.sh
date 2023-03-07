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
            net="${2}"
            shift 2
            ;;
        *)
            echo "${1} unknown option"
            usage
            exit 1
            ;;
    esac
done

if [[ -z "${net}" ]]
then
    echo "--network <main|test> is required"
    exit 1
fi

# Grab current location and source the shared functions.
# shellcheck source=.shared-functions.sh
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
source "${location}/../../tools/.shared-functions.sh"

test_dir=/tmp/mirror_test
bin_dir="${CARGO_TARGET_DIR}"/release

echo "${GIT_BASE}"

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

popd >/dev/null

pwd

app="full-service"
log="${test_dir}/${app}.log"
pid="${test_dir}/${app}.pid"
if [[ -f "${bin_dir}/full-service" ]]
then

    if [[ "$(check_pid_file ${pid})" == "running" ]]
    then
        echo "${app} is already running"
    else
        echo "- Starting full-service with validator-service"
        exec "${GIT_BASE}"/tools/run-fs.sh --validator "${net}" > "${log}" 2>&1 &
        echo $! > "${pid}"
    fi
else
    echo "${bin_dir}/full-service not found, did you build with tools/build-fs.sh?"
    exit 1
fi


for v in 1 2
do
    # Set wallet url
    case "${v}" in
        1)
            wallet_service_uri="http://127.0.0.1:9090/wallet"
            ;;
        2)
            wallet_service_uri="http://127.0.0.1:9090/wallet/v2"
            ;;
    esac

    app="wallet-service-mirror-private"
    log="${test_dir}/v${v}-${app}.log"
    pid="${test_dir}/v${v}-${app}.pid"
    if [[ -f "${bin_dir}/${app}" ]]
    then
        if [[ "$(check_pid_file ${pid})" == "running" ]]
        then
            echo "v${v}-${app} is already running"
        else
            echo "- Starting v${v} ${app}"
            exec "${bin_dir}/${app}" \
                --mirror-public-uri "wallet-service-mirror://127.0.0.1:1000${v}/?ca-bundle=${test_dir}/server.crt&tls-hostname=localhost" \
                --wallet-service-uri "${wallet_service_uri}" \
                --mirror-key "${test_dir}/mirror-private.pem" \
                > "${log}" 2>&1 &
            echo $! > "${pid}"
        fi
    else
        echo "${bin_dir}/${log} not found, did you build with tools/build-fs.sh?"
        exit 1
    fi

    app="wallet-service-mirror-public"
    log="${test_dir}/v${v}-${app}.log"
    pid="${test_dir}/v${v}-${app}.pid"
    if [[ -f "${bin_dir}/${app}" ]]
    then
        if [[ "$(check_pid_file ${pid})" == "running" ]]
        then
            echo "v${v} ${app} is already running"
        else
            echo "- Starting v${v} ${app}"
            exec "${bin_dir}/${app}" \
                --client-listen-uri "http://${LISTEN_ADDR}:909${v}/" \
                --mirror-listen-uri "wallet-service-mirror://127.0.0.1:1000${v}/?tls-chain=${test_dir}/server.crt&tls-key=${test_dir}/server.key" \
                --allow-self-signed-tls \
                > "${log}" 2>&1 &
            echo $! > "${pid}"
        fi
    else
        echo "${bin_dir}/${app} not found, did you build with tools/build-fs.sh?"
        exit 1
    fi
done

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
