#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# To use this script, run build-signer test or build-signer main

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

set -e

usage()
{
    echo "Usage:"
    echo "${0} <network>"
    echo "    <network> - main|prod|test|alpha|local or other"
    echo "                if other, set your own variables"
    echo "                SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS"
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

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

debug "RELEASE_DIR: ${RELEASE_DIR:?}"

case ${net} in
    test|prod|main)
        echo "Setting '${net}' SGX, IAS and enclave values"

        # CBB: we should replicate the "prod" css bucket to "main", then we can
        #      get rid of this workaround.
        if [[ "${net}" == "main" ]]
        then
            echo "Detected \"main\" network, setting css urls to use \"prod\""
            net="prod"
        fi

        SGX_MODE=HW
        IAS_MODE=PROD
        CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/consensus-enclave.css")
        INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/ingest-enclave.css")
    ;;
    alpha)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=DEV
        # CBB: same pattern as run - prompt user to provide their own css files.
        # alpha needs a css repository setup
        CONSENSUS_ENCLAVE_CSS="$(get_css_file "${net}" "${RELEASE_DIR}/consensus-enclave.css")"
        INGEST_ENCLAVE_CSS="$(get_css_file "${net}" "${RELEASE_DIR}/ingest-enclave.css")"
    ;;
    local)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=SW
        IAS_MODE=DEV
        CONSENSUS_ENCLAVE_CSS="${RELEASE_DIR}/consensus-enclave.css"
        INGEST_ENCLAVE_CSS="${RELEASE_DIR}/ingest-enclave.css"
    ;;
    *)
        echo "Using current environment's SGX, IAS and enclave values"
    ;;
esac

export SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS

echo "  IAS_MODE: ${IAS_MODE}"
echo "  SGX_MODE: ${SGX_MODE}"
echo "  CONSENSUS_ENCLAVE_CSS: ${CONSENSUS_ENCLAVE_CSS}"
echo "  INGEST_ENCLAVE_CSS: ${INGEST_ENCLAVE_CSS}"

echo "building transaction signer..."
cd transaction-signer
# shellcheck disable=SC2086 # split away - Use BUILD_OPTIONS to set additional build options
cargo build --release ${BUILD_OPTIONS:-}

target_dir=${CARGO_TARGET_DIR:-"target"}
echo "  binaries are available in ${target_dir}/release"
