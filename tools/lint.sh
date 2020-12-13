#!/bin/bash

# Copyright (c) 2018-2020 MobileCoin Inc.

set -e

: SGX_MODE=${SGX_MODE:-SW}
: IAS_MODE=${IAS_MODE:-DEV}
: CONSENSUS_ENCLAVE_CSS=${CONSENSUS_ENCLAVE_CSS:?"Must provide CONSENSUS_ENCLAVE_CSS to bypass enclave build"}

if [[ ! -z "$1" ]]; then
    cd "$1"
fi

for toml in $(grep --exclude-dir cargo --exclude-dir rust-mbedtls --include=Cargo.toml -r . -e '\[workspace\]' | cut -d: -f1); do
  pushd $(dirname $toml) >/dev/null
  echo "Linting in $PWD"
  cargo fmt -- --unstable-features --check
  cargo clippy --all --all-features
  echo "Linting in $PWD complete."
  popd >/dev/null
done
