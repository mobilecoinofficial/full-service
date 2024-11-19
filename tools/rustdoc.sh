#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

set -e

net="test"

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

debug "RELEASE_DIR: ${RELEASE_DIR:?}"

SGX_MODE=HW
CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/consensus-enclave.css")
INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/ingest-enclave.css")

export SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS

echo "  SGX_MODE: ${SGX_MODE}"
echo "  CONSENSUS_ENCLAVE_CSS: ${CONSENSUS_ENCLAVE_CSS}"
echo "  INGEST_ENCLAVE_CSS: ${INGEST_ENCLAVE_CSS}"
echo "building full service documentation..."

cargo doc --package=mc-full-service
