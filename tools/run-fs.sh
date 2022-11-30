#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

set -e

# Overrides and Options
#  - DEBUG or RUNNER_DEBUG print debug messages
#  - WORK_DIR - Set download directory for .css files.
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
    echo "${0} [--build] <network>"
    echo "    --build - optional: build with build-fs.sh"
    echo "    <network> - main|prod|test|alpha|local or other"
    echo "                if other, set your own variables"
    echo "                MC_CHAIN_ID MC_PEER MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS"
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


# Grab current location and source the shared functions.
# shellcheck source=.shared-functions.sh
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
source "${location}/.shared-functions.sh"

# Setup workdir
WORK_DIR="${WORK_DIR:-"${HOME}/.mobilecoin/${net}"}"
mkdir -p "${WORK_DIR}"

# Set default database directories
WALLET_DB_DIR="${WALLET_DB_DIR:-"${WORK_DIR}/wallet-db"}"
LEDGER_DB_DIR="${LEDGER_DB_DIR:-"${WORK_DIR}/ledger-db"}"
mkdir -p "${WALLET_DB_DIR}"
mkdir -p "${LEDGER_DB_DIR}"

# Set vars for all networks
MC_WALLET_DB="${WALLET_DB_DIR}/wallet.db"
MC_LEDGER_DB="${LEDGER_DB_DIR}"

case "${net}" in
    test)
        domain_name="test.mobilecoin.com"
        tx_source_base="https://s3-us-west-1.amazonaws.com/mobilecoin.chain"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
        MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
        MC_FOG_INGEST_ENCLAVE_CSS=$(get_css_file "test" "${WORK_DIR}/ingest-enclave.css")

        ;;
    prod|main)
        # CBB: we should replicate the "prod" css bucket to "main", then we can
        #      get rid of this workaround.
        if [[ "${net}" == "main" ]]
        then
            echo "Detected \"main\" network, setting css urls to use \"prod\""
            net="prod"
        fi

        domain_name="prod.mobilecoinww.com"
        tx_source_base="https://ledger.mobilecoinww.com"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
        MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
        MC_FOG_INGEST_ENCLAVE_CSS=$(get_css_file "prod" "${WORK_DIR}/ingest-enclave.css")
    ;;
    alpha)
        echo "alpha network doesn't yet publish enclave css measurements, manually download or copy ${WORK_DIR}/ingest-enclave.css"

        domain_name="alpha.development.mobilecoin.com"
        tx_source_base="https://s3-eu-central-1.amazonaws.com/mobilecoin.eu.development.chain"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
        MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
        MC_FOG_INGEST_ENCLAVE_CSS="${WORK_DIR}/ingest-enclave.css"
    ;;
    local)
        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        MC_PEER="mc-insecure://localhost:3200/,mc-insecure://localhost:3201/"
        MC_TX_SOURCE_URL="http://localhost:4566/node-0-ledger/,http://localhost:4566/node-1-ledger/"
        MC_FOG_INGEST_ENCLAVE_CSS="${INGEST_ENCLAVE_CSS}"
    ;;
    *)
        echo "Using current environment's SGX, IAS and enclave values"
        echo "Set MC_CHAIN_ID, MC_PEER, MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS as appropriate"
    ;;
esac

echo "Setting '${net}' environment values"

export MC_CHAIN_ID MC_PEER MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS MC_WALLET_DB MC_LEDGER_DB

echo "  MC_CHAIN_ID: ${MC_CHAIN_ID}"
echo "  MC_PEER: ${MC_PEER}"
echo "  MC_TX_SOURCE_URL: ${MC_TX_SOURCE_URL}"
echo "  MC_FOG_INGEST_ENCLAVE_CSS: ${MC_FOG_INGEST_ENCLAVE_CSS}"
echo "  MC_WALLET_DB: ${MC_WALLET_DB}"
echo "  MC_LEDGER_DB: ${MC_LEDGER_DB}"

# Optionally call build-fs.sh to build the current version.
if [[ -n "${build}" ]]
then
    "${location}/build-fs.sh" "${net}"
fi

target_dir=${CARGO_TARGET_DIR:-"target"}
echo "  executing ${target_dir}/release/full-service"
"${target_dir}/release/full-service"
