#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

set -e

# Overrides and Options
#  - DEBUG or RUNNER_DEBUG print debug messages
#  - RELEASE_DIR - Set download directory for .css files.
#     - Must be a fully qualified path.
#  - CSS_BASE_URL - Set the base http url to get .css index and files.
#  - CSS_JSON_FILE - defaults to production.json
#     - Override to get measurements for a specific mobilecoin release.
#       production-v3.0.0.json
#  - BUILD_OPTIONS - add additional cargo build options.
#     - Add "--locked" for production builds.

usage()
{
    echo "Usage:"
    echo "${0} [--build] [--ledger-validator] <network>"
    echo "    --build - optional: build with build-fs.sh"
    echo "    <network> - main|prod|test|alpha|local or other"
    echo "                if other, set your own variables"
    echo "                MC_CHAIN_ID MC_PEER MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS"
    echo "    --validator - optional: run with a local validator-service setup. Will start the validator-service on port 10001."
}

while (( "$#" ))
do
    case "${1}" in
        --help | -h)
            usage
            exit 0
            ;;
        --build)
            build=1
            shift 1
            ;;
        --download-ledger)
            download_ledger=1
            shift 1
            ;;
        --offline)
            export MC_OFFLINE=true
            shift 1
            ;;
        --validator)
            validator=1
            shift 1
            ;;
        *)
            net="${1}"
            shift 1
            ;;
    esac
done

if [[ -z "${net}" ]]
then
    echo "ERROR: <network> is not set"
    usage
    exit 1
fi

# use main instead of legacy prod
if [[ "${net}" == "prod" ]]
then
    echo "Detected \"prod\" legacy network setting. Using \"main\" instead."
    net=main
fi

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

# Set vars for all networks
MC_WALLET_DB="${WALLET_DB_DIR:?}/wallet.db"
MC_LEDGER_DB="${LEDGER_DB_DIR:?}"
MC_LISTEN_HOST="${LISTEN_ADDR:?}"

case "${net}" in
    test)
        domain_name="test.mobilecoin.com"
        tx_source_base="https://s3-us-west-1.amazonaws.com/mobilecoin.chain"
        ledger_source="https://mcdevus1ledger.blob.core.windows.net/test/data.mdb"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        if [[ -z "${MC_OFFLINE}" ]]
        then
            MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
            MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
            MC_FOG_INGEST_ENCLAVE_CSS=$(get_css_file "test" "${ENCLAVE_RELEASE_TAG}" "${RELEASE_DIR}/ingest-enclave.css")
        fi
        ;;
    main)
        domain_name="prod.mobilecoinww.com"
        tx_source_base="https://ledger.mobilecoinww.com"
        ledger_source="https://mcdeveu1ledger.blob.core.windows.net/main/data.mdb"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        if [[ -z "${MC_OFFLINE}" ]]
        then
            MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
            MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
            MC_FOG_INGEST_ENCLAVE_CSS=$(get_css_file "prod" "${ENCLAVE_RELEASE_TAG}" "${RELEASE_DIR}/ingest-enclave.css")
        fi
        ;;
    alpha)
        echo "alpha network doesn't yet publish enclave css measurements, manually download or copy ${RELEASE_DIR}/ingest-enclave.css"

        domain_name="alpha.development.mobilecoin.com"
        tx_source_base="https://s3-eu-central-1.amazonaws.com/mobilecoin.eu.development.chain"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        if [[ -z "${MC_OFFLINE}" ]]
        then
            MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
            MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
        fi
        MC_FOG_INGEST_ENCLAVE_CSS="${RELEASE_DIR}/ingest-enclave.css"
        ;;
    local)
        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        if [[ -z "${MC_OFFLINE}" ]]
        then
            MC_PEER="insecure-mc://localhost:3200/,insecure-mc://localhost:3201/"
            MC_TX_SOURCE_URL="file://$PWD/mobilecoin/target/docker/release/mc-local-network/node-ledger-distribution-0,file://$PWD/mobilecoin/target/docker/release/mc-local-network/node-ledger-distribution-1"
        fi
        MC_FOG_INGEST_ENCLAVE_CSS="${RELEASE_DIR}/ingest-enclave.css"
        ;;
    *)
        echo "Using current environment's SGX, IAS and enclave values"
        echo "Set MC_CHAIN_ID, MC_PEER, MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS as appropriate"
        ;;
esac

echo "Setting '${net}' environment values"

export MC_CHAIN_ID MC_PEER MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS MC_WALLET_DB MC_LEDGER_DB MC_LISTEN_HOST

echo "  MC_CHAIN_ID: ${MC_CHAIN_ID}"
if [[ -z "${MC_OFFLINE}" ]]
then
    echo "  MC_PEER: ${MC_PEER}"
    echo "  MC_TX_SOURCE_URL: ${MC_TX_SOURCE_URL}"
else
    echo "  running with --offline"
fi
echo "  MC_FOG_INGEST_ENCLAVE_CSS: ${MC_FOG_INGEST_ENCLAVE_CSS}"
echo "  MC_WALLET_DB: ${MC_WALLET_DB}"
echo "  MC_LEDGER_DB: ${MC_LEDGER_DB}"
echo "  MC_LISTEN_HOST: ${MC_LISTEN_HOST}"


# Optionally call build-fs.sh to build the current version.
if [[ -n "${build}" ]]
then
    "${location}/build-fs.sh" "${net}"
fi

if [[ -n "${ledger_source}" && -n "${download_ledger}" ]]
then
    if [[ -e "${MC_LEDGER_DB}/data.mdb" ]]
    then
        echo "Ledger already exists at ${MC_LEDGER_DB}/data.mdb"
        echo "Remove ${MC_LEDGER_DB}/data.mdb or remove --download-ledger flag"
        exit 1
    else
        echo "Downloading ledger from ${ledger_source}"
        curl -SLf "${ledger_source}" -o "${MC_LEDGER_DB}/data.mdb"
    fi
fi

# start validator and unset envs for full-service
if [[ -n "${validator}" ]]
then
    running=$(check_pid_file /tmp/.validator-service.pid)
    if [[ "${running}" == "running" ]]
    then
        echo "validator-service is already running"
    else
        # Make sure that the validator executable exists.
        if [[ ! -f "${RELEASE_DIR}/validator-service" ]]
        then
            echo "${RELEASE_DIR}/validator-service not found."
            exit 1
        fi

        echo "Starting validator-service. Log is at /tmp/validator-service.log"
        validator_ledger_db="${GIT_BASE}/.mob/${net}-db/validator/ledger-db"
        mkdir -p "${validator_ledger_db}"

        # Override
        "${RELEASE_DIR}/validator-service" \
            --ledger-db "${validator_ledger_db}" \
            --listen-uri "insecure-validator://127.0.0.1:11000/" \
            >/tmp/validator-service.log 2>&1 &

        echo $! >/tmp/.validator-service.pid
        echo "Pausing 10 seconds to wait for validator to come up."
        sleep 10
    fi

    export MC_VALIDATOR="insecure-validator://127.0.0.1:11000/"
    unset MC_TX_SOURCE_URL
    unset MC_PEER
fi

echo "Starting Full-Service Wallet from ${RELEASE_DIR}"
"${RELEASE_DIR}/full-service"
