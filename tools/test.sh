#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

# RUSTFLAGS="-C instrument-coverage" \

# Grab current location and source the shared functions.
# shellcheck source=.shared-functions.sh
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
source "${location}/.shared-functions.sh"

net="test"
WORK_DIR="${WORK_DIR:-"${HOME}/.mobilecoin/${net}"}"
mkdir -p "${WORK_DIR}"

SGX_MODE=SW
IAS_MODE=DEV
CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${WORK_DIR}/consensus-enclave.css")
export SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS

cargo test $@
