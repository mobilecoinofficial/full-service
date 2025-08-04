#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

set -e

usage()
{
    echo "Usage:"
    echo "${0} <network>|check"
    echo "    <network> - main|prod|test|alpha|local or other"
    echo "                if other, set your own variables"
    echo "                SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS"
    echo "    check"
    echo "                Sets build parameters to those used for network=local, and"
    echo "                runs 'cargo check'"
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
    echo "Detected 'prod' legacy network setting. Using 'main' instead."
    net=main
fi

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

debug "RELEASE_DIR: ${RELEASE_DIR:?}"

"${RELEASE_DIR}/wallet-service-mirror-public" \
    --client-listen-uri http://0.0.0.0:9091/ \
    --mirror-listen-uri "wallet-service-mirror://0.0.0.0/?tls-chain=${RELEASE_DIR}/server.crt&tls-key=${RELEASE_DIR}/server.key" \
    --allow-self-signed-tls
