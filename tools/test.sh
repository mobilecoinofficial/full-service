#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

# RUSTFLAGS="-C instrument-coverage" \

if ! [[ -f "$(pwd)/consensus-enclave.css" ]]; then
  echo "Please copy the consensus-enclave.css to $(pwd)"
  exit 1
fi

echo "Testing in $PWD"
SGX_MODE=SW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css \
cargo test $1
echo "Testing in $PWD complete."