#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

# RUSTFLAGS="-C instrument-coverage" \

set -e

if [[ ! -z "$1" ]]; then
    cd "$1"
fi

echo "Testing in $PWD"
CARGO_INCREMENTAL=0
RUSTFLAGS='-Cinstrument-coverage'
LLVM_PROFILE_FILE="../target/profraw/json5format-%m.profraw"
SGX_MODE=SW
IAS_MODE=DEV
CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css
export CARGO_INCREMENTAL RUSTFLAGS LLVM_PROFILE_FILE SGX_MODE IAS_MODE CONSENSUS_ENCLAVE_CSS

cargo test
echo "Testing in $PWD complete."

echo "Building coverage report (lcov) html to target/coverage/results"
mkdir -p ./target/coverage
grcov . --binary-path ./target/debug/deps/ --source-dir . -t lcov --branch --ignore-not-existing \
--ignore "../*" --ignore "/*" --ignore "mobilecoin/*" --ignore "target/*" --ignore "*/e2e_tests/*" \
-o ./target/coverage/tests.lcov
genhtml ./target/coverage/tests.lcov --show-details --prefix "$PWD" --output-directory ./target/coverage/results

if [[ "${OSTYPE}" == "darwin"* ]]; then
  open ./target/coverage/results/index.html
else
  echo "test output written to target/coverage/results/index.html"
fi
