#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

set -e

if [[ ! -z "$1" ]]; then
    cd "$1"
fi

echo "Testing in $PWD"
SGX_MODE=SW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo test
echo "Testing in $PWD complete."
