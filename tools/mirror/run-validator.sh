#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

set -e

usage()
{
    echo "Usage:"
    echo "${0} <network>"
    echo "    <network> - main|prod|test|local or other"
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

debug "RELEASE_DIR: ${RELEASE_DIR:?}"

case "${net}" in
    test)
        domain_name="test.mobilecoin.com"
        tx_source_base="https://s3-us-west-1.amazonaws.com/mobilecoin.chain"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        if [[ -z "${MC_OFFLINE}" ]]
        then
            MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
            MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
            MC_FOG_INGEST_ENCLAVE_CSS=$(get_css_file "test" "${RELEASE_DIR}/ingest-enclave.css")
        fi
        ;;
    main)
        domain_name="prod.mobilecoinww.com"
        tx_source_base="https://ledger.mobilecoinww.com"

        # Set chain id, peer and tx_sources for 2 nodes.
        MC_CHAIN_ID="${net}"
        if [[ -z "${MC_OFFLINE}" ]]
        then
            MC_PEER="mc://node1.${domain_name}/,mc://node2.${domain_name}/"
            MC_TX_SOURCE_URL="${tx_source_base}/node1.${domain_name}/,${tx_source_base}/node2.${domain_name}/"
            MC_FOG_INGEST_ENCLAVE_CSS=$(get_css_file "prod" "${RELEASE_DIR}/ingest-enclave.css")
        fi
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

export MC_CHAIN_ID MC_PEER MC_TX_SOURCE_URL MC_FOG_INGEST_ENCLAVE_CSS
echo "  MC_CHAIN_ID: ${MC_CHAIN_ID}"
echo "  MC_PEER: ${MC_PEER}"
echo "  MC_TX_SOURCE_URL: ${MC_TX_SOURCE_URL}"
echo "  MC_FOG_INGEST_ENCLAVE_CSS: ${MC_FOG_INGEST_ENCLAVE_CSS}"

echo "Starting validator-service. Log is at /tmp/validator-service.log"
validator_ledger_db="${RELEASE_DIR}/validator/ledger-db"
mkdir -p "${validator_ledger_db}"

"${RELEASE_DIR}/validator-service" \
    --ledger-db "${validator_ledger_db}" \
    --listen-uri "insecure-validator://127.0.0.1:11000/"
