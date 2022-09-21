#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
PROJECT_DIR=$SCRIPT_DIR/../mobilecoin

CARGO_FLAGS=--release
LEDGER_BASE=$PROJECT_DIR/target/sample_data/ledger

ENCLAVE_PEM=$PROJECT_DIR/Enclave_private.pem
if test -f "$ENCLAVE_PEM"; then
    echo "$ENCLAVE_PEM exists."
else
    openssl genrsa -out $ENCLAVE_PEM -3 3072
fi

cd $PROJECT_DIR && CONSENSUS_ENCLAVE_PRIVKEY=$ENCLAVE_PEM cargo build -p mc-consensus-service -p mc-ledger-distribution -p mc-admin-http-gateway -p mc-util-grpc-admin-tool -p mc-mint-auditor -p mc-crypto-x509-test-vectors -p mc-consensus-mint-client -p mc-util-seeded-ed25519-key-gen $CARGO_FLAGS   
