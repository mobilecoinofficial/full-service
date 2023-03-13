#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# To use this script, run build-fs test or build-fs main

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

# use main instead of legacy prod
if [[ "${net}" == "prod" ]]
then
    echo "Detected \"prod\" legacy network setting. Using \"main\" instead."
    net=main
fi

# Grab current location and source the shared functions.
# shellcheck source=.shared-functions.sh
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
source "${location}/.shared-functions.sh"

# Setup workdir - set in .shared-functions.sh
mkdir -p "${WORK_DIR}"

case ${net} in
    test)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=PROD
        CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${WORK_DIR}/consensus-enclave.css")
        INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${WORK_DIR}/ingest-enclave.css")
        ;;
    main)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=PROD
        CONSENSUS_ENCLAVE_CSS=$(get_css_file prod "${WORK_DIR}/consensus-enclave.css")
        INGEST_ENCLAVE_CSS=$(get_css_file prod "${WORK_DIR}/ingest-enclave.css")
        ;;
    alpha)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=DEV
        # CBB: same pattern as run - prompt user to provide their own css files.
        # alpha needs a css repository setup
        CONSENSUS_ENCLAVE_CSS="$(get_css_file "${net}" "${WORK_DIR}/consensus-enclave.css")"
        INGEST_ENCLAVE_CSS="$(get_css_file "${net}" "${WORK_DIR}/ingest-enclave.css")"
        ;;
    local)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=SW
        IAS_MODE=DEV

        INGEST_ENCLAVE_CSS="${WORK_DIR}/ingest-enclave.css"
        if test -f "${WORK_DIR}/consensus-enclave.css"; then
            CONSENSUS_ENCLAVE_CSS="${WORK_DIR}/consensus-enclave.css"
        else
            CONSENSUS_ENCLAVE_CSS=`pwd`/mobilecoin/target/docker/release/consensus-enclave.css
        fi
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

echo "building full service..."
# shellcheck disable=SC2086 # split away - Use BUILD_OPTIONS to set additional build options
cargo build --release ${BUILD_OPTIONS}


# target_dir=${CARGO_TARGET_DIR:-"target"}
# echo "  binaries are available in ${target_dir}/release"

# echo "  copy css files to ${target_dir}/release for later packaging (docker, tar)"
# cp "${CONSENSUS_ENCLAVE_CSS}" "${target_dir}/release"
# cp "${INGEST_ENCLAVE_CSS}" "${target_dir}/release"
