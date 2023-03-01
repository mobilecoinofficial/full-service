#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

set -e

net=test

# Grab current location and source the shared functions.
# shellcheck source=.shared-functions.sh
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
source "${location}/.shared-functions.sh"

# Setup workdir
WORK_DIR="${WORK_DIR:-"${HOME}/.mobilecoin/${net}"}"
mkdir -p "${WORK_DIR}"

SGX_MODE=HW
IAS_MODE=PROD
CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${WORK_DIR}/consensus-enclave.css")
INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${WORK_DIR}/ingest-enclave.css")

export SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS

echo "  IAS_MODE: ${IAS_MODE}"
echo "  SGX_MODE: ${SGX_MODE}"
echo "  CONSENSUS_ENCLAVE_CSS: ${CONSENSUS_ENCLAVE_CSS}"
echo "  INGEST_ENCLAVE_CSS: ${INGEST_ENCLAVE_CSS}"
echo "building full service documentation..."

cargo doc --package=mc-full-service
