#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

set -e
set -o pipefail

net="test"

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

debug "RELEASE_DIR: ${RELEASE_DIR:?}"

SGX_MODE=SW
CONSENSUS_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/consensus-enclave.css")
INGEST_ENCLAVE_CSS=$(get_css_file "${net}" "${RELEASE_DIR}/ingest-enclave.css")
export SGX_MODE CONSENSUS_ENCLAVE_CSS INGEST_ENCLAVE_CSS

# Find all Cargo.toml files with a workspace and run cargo fmt and cargo clippy on them
grep -l -r --exclude-dir .mob --exclude-dir target --exclude-dir mobilecoin --exclude-dir ledger-mob --include=Cargo.toml -e '\[workspace\]' | while IFS= read -r toml
do
    pushd "$(dirname "${toml}")" >/dev/null
    echo "Linting in ${PWD}"

    cargo fmt -- --unstable-features --check

    cargo clippy --all --all-features
    echo "Linting in ${PWD} complete."
    popd >/dev/null
done
