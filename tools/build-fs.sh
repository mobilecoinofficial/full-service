#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# To use this script, run build-fs test or build-fs main
# If you want to just run `cargo check` run build-fs check

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
    echo "${0} <network>|check"
    echo "    <network> - main|prod|test|alpha|local or other"
    echo "                if other, set your own variables"
    echo "                SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS"
    echo "    check"
    echo "                Sets build parameters to those used for network=local, and"
    echo "                runs `cargo check`"
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
# shellcheck source=.shared-functions.sh
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
source "${location}/.shared-functions.sh"

# Setup release dir - set in .shared-functions.sh
mkdir -p "${RELEASE_DIR}"

# link workdir to release dir path
if [[ "${AM_I_IN_MOB_PROMPT}" == "yes" ]]
then
    # migrate wallet/ledger db to release_dir and remove workdir to make room
    # for the symlink
    if [[ ! -L "${WORK_DIR}" && -d "${WORK_DIR}" ]]
    then
        if [[ -d "${WORK_DIR}/wallet-db" ]]
        then
            mv "${WORK_DIR}/wallet-db" "${RELEASE_DIR}/wallet-db"
        fi
        if [[ -d "${WORK_DIR}/ledger-db" ]]
        then
            mv "${WORK_DIR}/ledger-db" "${RELEASE_DIR}/ledger-db"
        fi
        rm -rf "${WORK_DIR}"
    fi

    # create parent workdir and link release_dir to work_dir
    mkdir -p "$(dirname "${WORK_DIR}")"

    # At this point WORK_DIR can only be a symlink, remove it and create a new one.
    rm -f "${WORK_DIR}"

    # this needs to be a relative link from GIT_BASE
    ln -s -r "${RELEASE_DIR}" "${WORK_DIR}"
fi

case ${net} in
    test)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=PROD
        CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/consensus-enclave.css")
        INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/ingest-enclave.css")
        ;;
    main)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=PROD
        CONSENSUS_ENCLAVE_CSS=$(get_css_file prod "${RELEASE_DIR}/consensus-enclave.css")
        INGEST_ENCLAVE_CSS=$(get_css_file prod "${RELEASE_DIR}/ingest-enclave.css")
        ;;
    alpha)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=HW
        IAS_MODE=DEV
        # User must provide their own .css files
        CONSENSUS_ENCLAVE_CSS="${RELEASE_DIR}/consensus-enclave.css"
        INGEST_ENCLAVE_CSS="${RELEASE_DIR}/ingest-enclave.css"
        ;;
    check | local)
        echo "Setting '${net}' SGX, IAS and enclave values"
        SGX_MODE=SW
        IAS_MODE=DEV

        INGEST_ENCLAVE_CSS="${RELEASE_DIR}/ingest-enclave.css"
        if [[ -f "${RELEASE_DIR}/consensus-enclave.css" ]]
        then
            CONSENSUS_ENCLAVE_CSS="${RELEASE_DIR}/consensus-enclave.css"
        else
            CONSENSUS_ENCLAVE_CSS=$(pwd)/mobilecoin/target/docker/release/consensus-enclave.css
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

if [[ "${net}" == "check" ]]
then
    echo "Just doing cargo check - no binaries will be built"
    cargo check
    exit
fi

echo "building full service..."
# shellcheck disable=SC2086 # split away - Use BUILD_OPTIONS to set additional build options
cargo build --release ${BUILD_OPTIONS}

if [[ "${AM_I_IN_MOB_PROMPT}" == "no" ]]
then
    cp "${RELEASE_DIR}/full-service" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/generate-rsa-keypair" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/signer" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/signer-service" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/validator-service" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/wallet-service-mirror-private" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/wallet-service-mirror-public" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/consensus-enclave.css" "${WORK_DIR}/"
    cp "${RELEASE_DIR}/ingest-enclave.css" "${WORK_DIR}/"
    echo "  Binaries are available in ${RELEASE_DIR} and ${WORK_DIR}"
else
    echo "  Binaries are available in ${RELEASE_DIR}"
fi
