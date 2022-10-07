#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# TODO: add a way to specify a different sigstruct directory

# Net can be main/test/local
if [ $# -eq 0 ]; then
NET="test"
else
NET=$1
fi

if [ $NET == "local" ]; then
echo "Local networks are not currently supported with this script"
exit 1
elif [ $NET == "test" ]; then
echo "Building Full Service with testnet settings"
NAMESPACE="test"
export SGX_MODE=SW
export IAS_MODE=DEV
elif [ $NET == "main" ]; then
NAMESPACE="prod"
export SGX_MODE=HW
export IAS_MODE=PROD
else
echo "Network specified is not valid. Try 'test' or 'main'. Default is test"
exit 1
fi

WORK_DIR="$HOME/.mobilecoin/${NET}"
mkdir -p ${WORK_DIR}

export CONSENSUS_ENCLAVE_CSS="$WORK_DIR/consensus-enclave.css"

(cd ${WORK_DIR} && CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${CONSENSUS_SIGSTRUCT_URI})

echo "building full service..."
cargo build --release -p mc-full-service

