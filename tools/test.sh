#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

# RUSTFLAGS="-C instrument-coverage" \

# must set net before sourcing shared functions
net="test"

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )

# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

mkdir -p "${RELEASE_DIR:?}"
debug "RELEASE_DIR: ${RELEASE_DIR}"

SGX_MODE=HW
CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/consensus-enclave.css")
INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/ingest-enclave.css")
export SGX_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS

echo "CONSENSUS_ENCLAVE_CSS: ${CONSENSUS_ENCLAVE_CSS}"
echo "INGEST_ENCLAVE_CSS: ${INGEST_ENCLAVE_CSS}"

cargo test "$@"
